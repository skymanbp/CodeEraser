//! M4 pre-registered evaluation-set freeze (plan §6 M4, D2-1 purity;
//! composition per user rulings 2026-08-10: 600 samples, 100% real
//! transcripts — 400 observe-feed-linked + 200 pre-guard — manifest
//! in-repo, payloads local, 200-sample labeling subset).
//!
//! Machine-specific by design (like fpr_replay): reads this machine's
//! Claude Code transcripts and observe feeds, so it is #[ignore]d and
//! driven by env vars — no user-home literals in code (portability,
//! and the transcripts root is genuinely configurable):
//!   CE_EVAL_TRANSCRIPTS  dir of per-project transcript folders
//!   CE_EVAL_FEEDS        dir whose children hold .ce/observe.ndjson
//!   CE_EVAL_FROZEN_AT    UTC ISO instant; events at/after are excluded
//!   CE_EVAL_OUT          local payload dir (default ../.ce-eval)
//!
//! Freeze:  cargo test --test eval_extract -- --ignored --nocapture
//! Verify:  re-run with the manifest's recorded frozen_at — selection
//!          is hash-ranked (no RNG, no clock), so identical inputs
//!          reproduce identical ids and hashes.

mod freeze;
mod scan;

use freeze::Class;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// First guard install on the dogfood machine (ce 98d2af3, 2026-08-07
/// 18:20 +08:00) expressed in UTC as a lexicographic comparison prefix.
const INSTALL_CUTOFF_UTC: &str = "2026-08-07T10:20:00";

const QUOTA_OBSERVE: usize = 400;
const QUOTA_PRE_GUARD: usize = 200;
const QUOTA_LABELING: usize = 200;

fn env_dir(name: &str) -> PathBuf {
    let v = std::env::var(name)
        .unwrap_or_else(|_| panic!("{name} must point at the corpus (see file header)"));
    PathBuf::from(v)
}

fn observe_session_ids(feeds_root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Ok(projects) = std::fs::read_dir(feeds_root) else {
        return ids;
    };
    for project in projects.flatten() {
        let feed = project.path().join(".ce").join("observe.ndjson");
        let Ok(text) = std::fs::read_to_string(&feed) else {
            continue;
        };
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if v["schema"].as_str() == Some("ce.observe/0.2.0")
                && v["mode"].as_str() == Some("observe")
                && let Some(sid) = v["session_id"].as_str()
            {
                ids.insert(sid.to_string());
            }
        }
    }
    ids
}

#[test]
#[ignore] // machine-specific corpus; run explicitly to freeze or verify
fn freeze_eval_set_v1() {
    let transcripts = env_dir("CE_EVAL_TRANSCRIPTS");
    let feeds = env_dir("CE_EVAL_FEEDS");
    let frozen_at = std::env::var("CE_EVAL_FROZEN_AT")
        .expect("CE_EVAL_FROZEN_AT (UTC ISO) pins the candidate horizon");
    let out_dir = std::env::var("CE_EVAL_OUT").unwrap_or_else(|_| "../.ce-eval".into());
    let out_dir = PathBuf::from(out_dir);

    let observe_ids = observe_session_ids(&feeds);
    assert!(
        !observe_ids.is_empty(),
        "no observe sessions found in feeds"
    );

    let (candidates, drops) = scan_all(&transcripts, &frozen_at);
    debug_assert!(
        candidates
            .iter()
            .all(|c| c.ts.as_str() < frozen_at.as_str())
    );

    let (observe, pre_guard, n_guard_era, n_deny) = partition(candidates, &observe_ids);
    let pool = (observe.len(), pre_guard.len());

    let observe = freeze::select(observe, QUOTA_OBSERVE);
    let pre_guard = freeze::select(pre_guard, QUOTA_PRE_GUARD);
    let manifest = build_manifest(
        &out_dir,
        &frozen_at,
        (&observe, &pre_guard),
        pool,
        (n_guard_era, n_deny, &drops),
    );

    let manifest_path = Path::new("../contracts/eval/manifest-v1.json");
    std::fs::create_dir_all(manifest_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("ser"),
    )
    .expect("write manifest");

    let verified = freeze::verify(&out_dir, &manifest).expect("verify frozen samples");
    assert_eq!(verified, QUOTA_OBSERVE + QUOTA_PRE_GUARD);
    println!(
        "frozen {} samples (observe {} of {}, pre_guard {} of {}) — \
         guard_era_unlinked {}, deny_test {}, verify OK",
        verified, QUOTA_OBSERVE, pool.0, QUOTA_PRE_GUARD, pool.1, n_guard_era, n_deny,
    );
}

type Partitioned = (Vec<scan::Candidate>, Vec<scan::Candidate>, usize, usize);

fn partition(candidates: Vec<scan::Candidate>, observe_ids: &BTreeSet<String>) -> Partitioned {
    let mut observe = Vec::new();
    let mut pre_guard = Vec::new();
    let (mut n_guard_era, mut n_deny) = (0usize, 0usize);
    for c in candidates {
        match freeze::classify(&c, observe_ids, INSTALL_CUTOFF_UTC) {
            Class::Observe => observe.push(c),
            Class::PreGuard => pre_guard.push(c),
            Class::GuardEraUnlinked => n_guard_era += 1,
            Class::DenyTest => n_deny += 1,
        }
    }
    (observe, pre_guard, n_guard_era, n_deny)
}

fn scan_all(transcripts: &Path, frozen_at: &str) -> (Vec<scan::Candidate>, scan::DropCounts) {
    let mut candidates = Vec::new();
    let mut drops = scan::DropCounts::default();
    let projects = std::fs::read_dir(transcripts).expect("read transcripts root");
    for project in projects.flatten() {
        if project.path().is_dir() {
            scan::scan_project(&project.path(), frozen_at, &mut candidates, &mut drops);
        }
    }
    (candidates, drops)
}

fn build_manifest(
    out_dir: &Path,
    frozen_at: &str,
    picked: (&[scan::Candidate], &[scan::Candidate]),
    pool: (usize, usize),
    dropped: (usize, usize, &scan::DropCounts),
) -> Value {
    let ids: Vec<(String, String)> = picked
        .0
        .iter()
        .map(|c| freeze::sample_id_and_hash(c, "observe"))
        .chain(
            picked
                .1
                .iter()
                .map(|c| freeze::sample_id_and_hash(c, "pre_guard")),
        )
        .map(|(id, digest, _)| (id, digest))
        .collect();
    let labeled = freeze::labeling_subset(&ids, QUOTA_LABELING);
    let mut rows = freeze::emit_samples(out_dir, picked.0, "observe", &labeled);
    rows.extend(freeze::emit_samples(
        out_dir,
        picked.1,
        "pre_guard",
        &labeled,
    ));
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    manifest_json(frozen_at, pool, dropped, rows)
}

fn manifest_json(
    frozen_at: &str,
    pool: (usize, usize),
    dropped: (usize, usize, &scan::DropCounts),
    rows: Vec<Value>,
) -> Value {
    let (n_guard_era, n_deny, drops) = dropped;
    json!({
        "schema": freeze::MANIFEST_SCHEMA,
        "frozen_at": frozen_at,
        "install_cutoff_utc": INSTALL_CUTOFF_UTC,
        "composition": {
            "observe": QUOTA_OBSERVE,
            "pre_guard": QUOTA_PRE_GUARD,
            "labeling_subset": QUOTA_LABELING,
            "pool_observe": pool.0,
            "pool_pre_guard": pool.1,
        },
        "excluded": {
            "guard_era_unlinked_edits": n_guard_era,
            "deny_test_edits": n_deny,
            "error_results": drops.error_result,
            "unsupported_tool": drops.unsupported_tool,
            "unsupported_lang": drops.unsupported_lang,
            "unreconstructible": drops.unreconstructible,
            "oversize": drops.oversize,
            "replayed_history": drops.replayed_history,
        },
        "method": "hash-ranked (sha256, seedless) largest-remainder allocation \
                   over (project, lang) strata; labeling subset hash-ranked over ids",
        "ground_truth": "git -M -C cross-check + heuristic pre-labels, then \
                         per-item review by the agent (user delegation 2026-08-10); \
                         pre-labels and corrections stored separately",
        "samples": rows,
    })
}
