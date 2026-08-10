//! Classification, deterministic stratified selection, and manifest
//! emission for the M4 pre-registered evaluation set (plan §6 M4,
//! D2-1 purity). Selection is hash-ranked — no RNG, no clock — so the
//! same inputs always freeze the same set.

use crate::scan::Candidate;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const MANIFEST_SCHEMA: &str = "ce.eval-set/1.0.0";

/// D2-1 class of one candidate, or the reason it is out of scope.
pub enum Class {
    Observe,
    PreGuard,
    GuardEraUnlinked,
    DenyTest,
}

/// Classify per D2-1: feed-linked observe sessions are machine-provably
/// unshaped; sessions that ended before the first guard install predate
/// shaping by definition; everything else is reported, not sampled.
pub fn classify(c: &Candidate, observe_ids: &BTreeSet<String>, install_cutoff: &str) -> Class {
    if c.project_slug.contains("t1-demo") {
        return Class::DenyTest;
    }
    if observe_ids.contains(&c.session_id) {
        return Class::Observe;
    }
    // ISO-8601 timestamps compare lexicographically within the same zone
    // (both sides are UTC "Z" stamps)
    if !c.ts.is_empty() && c.ts.as_str() < install_cutoff {
        return Class::PreGuard;
    }
    Class::GuardEraUnlinked
}

fn rank_key(c: &Candidate) -> String {
    let mut h = Sha256::new();
    h.update(b"ce-eval-v1|");
    h.update(c.session_id.as_bytes());
    h.update(b"|");
    h.update(c.ts.as_bytes());
    h.update(b"|");
    h.update(c.file_path.as_bytes());
    h.update(b"|");
    h.update(c.tool.as_bytes());
    hex(&h.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Largest-remainder allocation of `quota` across (project, lang)
/// strata, proportional to stratum size, then hash-rank inside each
/// stratum. Deterministic and coverage-proportional.
pub fn select(candidates: Vec<Candidate>, quota: usize) -> Vec<Candidate> {
    let mut strata: BTreeMap<(String, String), Vec<Candidate>> = BTreeMap::new();
    for c in candidates {
        let key = (c.project_slug.clone(), c.lang.to_string());
        strata.entry(key).or_default().push(c);
    }
    let total: usize = strata.values().map(Vec::len).sum();
    assert!(
        total >= quota,
        "class has {total} candidates < quota {quota}"
    );
    let mut shares: Vec<(&(String, String), usize, f64)> = strata
        .iter()
        .map(|(k, v)| {
            let exact = quota as f64 * v.len() as f64 / total as f64;
            (k, exact as usize, exact - (exact as usize) as f64)
        })
        .collect();
    let allocated: usize = shares.iter().map(|s| s.1).sum();
    shares.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| a.0.cmp(b.0)));
    let mut bump: BTreeSet<(String, String)> = BTreeSet::new();
    for s in shares.iter().take(quota - allocated) {
        bump.insert(s.0.clone());
    }
    let mut picked = Vec::with_capacity(quota);
    let quotas: Vec<((String, String), usize)> = strata
        .keys()
        .map(|k| {
            let base = quota * strata[k].len() / total;
            (k.clone(), base + usize::from(bump.contains(k)))
        })
        .collect();
    for (key, n) in quotas {
        let mut group = strata.remove(&key).unwrap_or_default();
        group.sort_by_key(rank_key);
        picked.extend(group.into_iter().take(n));
    }
    picked
}

/// Stable sample id + content hash over the full payload.
pub fn sample_id_and_hash(c: &Candidate, class: &str) -> (String, String, String) {
    let payload = json!({
        "schema": "ce.eval-sample/1.0.0",
        "class": class,
        "session_id": c.session_id,
        "project_slug": c.project_slug,
        "ts": c.ts,
        "tool": c.tool,
        "file_path": c.file_path,
        "lang": c.lang,
        "before": c.before,
        "after": c.after,
    })
    .to_string();
    let digest = hex(&Sha256::digest(payload.as_bytes()));
    (digest[..16].to_string(), digest, payload)
}

/// Write sample payloads locally (git-ignored) and return the manifest
/// rows. The manifest carries NO file paths and NO content — the
/// samples embed other private repositories, and this repo goes public
/// at M7 (D2-7); integrity is pinned by the per-sample sha256 instead.
pub fn emit_samples(
    out_dir: &Path,
    picked: &[Candidate],
    class: &str,
    labeled: &BTreeSet<String>,
) -> Vec<Value> {
    std::fs::create_dir_all(out_dir.join("samples")).expect("create sample dir");
    let mut rows = Vec::with_capacity(picked.len());
    for c in picked {
        let (id, digest, payload) = sample_id_and_hash(c, class);
        let path = out_dir.join("samples").join(format!("{id}.json"));
        std::fs::write(&path, payload).expect("write sample");
        rows.push(json!({
            "id": id,
            "sha256": digest,
            "class": class,
            "project_slug": c.project_slug,
            "lang": c.lang,
            "tool": c.tool,
            "ts": c.ts,
            "session_id": c.session_id,
            "labeling": labeled.contains(&id),
        }));
    }
    rows
}

/// Deterministic labeling subset: hash-rank the frozen ids and take
/// `n` — same discipline as selection, auditable from the manifest.
pub fn labeling_subset(picked: &[(String, String)], n: usize) -> BTreeSet<String> {
    let mut ranked: Vec<&(String, String)> = picked.iter().collect();
    ranked.sort_by_key(|(id, _)| hex(&Sha256::digest(format!("ce-eval-label-v1|{id}").as_bytes())));
    ranked
        .into_iter()
        .take(n)
        .map(|(id, _)| id.clone())
        .collect()
}

/// Re-read every local sample and verify it against the manifest —
/// the freeze is only as good as its re-checkable hashes. Ids must be
/// unique: the v1 dry run froze 600 rows over 317 files because
/// replayed transcript history duplicated events, and a count-only
/// check verified the same file repeatedly without noticing.
pub fn verify(out_dir: &Path, manifest: &Value) -> Result<usize, String> {
    let rows = manifest["samples"].as_array().ok_or("no samples array")?;
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().ok_or("row without id")?;
        let want = row["sha256"].as_str().ok_or("row without sha256")?;
        if !seen.insert(id) {
            return Err(format!("sample {id}: duplicate id in manifest"));
        }
        let path = out_dir.join("samples").join(format!("{id}.json"));
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let got = hex(&Sha256::digest(&bytes));
        if got != want {
            return Err(format!("sample {id}: sha256 mismatch"));
        }
    }
    Ok(seen.len())
}
