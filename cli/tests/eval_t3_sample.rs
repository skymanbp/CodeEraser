//! M5-3c t3 audit sample: 100 main rows + 20 per-language backup,
//! drawn by hash rank over the frozen five-corpus candidate pool
//! (design vol.3 §9.2) — no RNG, no clock, committed BEFORE any TED
//! judge or `dedup/t3` code exists (the T-G13 ancestry leg 3f
//! asserts by git log). Pool membership is bound by digest: the
//! sample doc records each corpus's pairs_sha256 and the CI gate
//! holds them equal to the frozen candidate docs forever.
//!
//! Generate (needs all five corpus repositories):
//!   cargo test --release --test eval_t3_sample -- --ignored --nocapture

mod eval_support;

use eval_support::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// T3 sample languages: the four with units — markdown is docdup's
/// domain and carries zero units by design (3b), so the design's
/// "floor 15 per language" spans four languages here, leaving
/// 100 − 60 = 40 largest-remainder seats.
const LANGS: [&str; 4] = ["go", "py", "rs", "ts"];
const MAIN: u64 = 100;
const FLOOR: u64 = 15;
const BACKUP: u64 = 20;
const RANK_DOMAIN: &str = "ce-t3-pair-v1";
const AUDIT_DOMAIN: &str = "ce-t3-audit-v1";

/// The domain-separated hash of one row under `domain` — the shared
/// identity_hash throat over this family's field order; the ONE
/// derivation verify() repeats from the frozen fields (G4).
fn row_hash(domain: &str, row: &Value) -> String {
    eval_support::identity_hash(
        domain,
        row,
        &[
            "corpus", "tip", "a_path", "a_key", "a_nth", "b_path", "b_key", "b_nth", "source",
        ],
    )
}

/// Main quotas: floor 15 per language, remaining seats by largest
/// remainder over pool shares (pure integer arithmetic, ties broken
/// by language order) — the ONE apportion the gate re-runs.
fn quotas(pool_by: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let seats = MAIN - FLOOR * LANGS.len() as u64;
    let total: u64 = pool_by.values().sum();
    let mut q: BTreeMap<String, u64> = BTreeMap::new();
    let mut rems: Vec<(u64, &str)> = Vec::new();
    let mut used = 0;
    for lang in LANGS {
        let share = seats * pool_by[lang];
        q.insert(lang.into(), FLOOR + share / total);
        used += share / total;
        rems.push((share % total, lang));
    }
    rems.sort_by(|x, y| (y.0, x.1).cmp(&(x.0, y.1)));
    for (_, lang) in rems.iter().take((seats - used) as usize) {
        *q.get_mut(*lang).expect("lang") += 1;
    }
    q
}

/// CI gate, no git: every stored rank re-derives from its own row
/// fields (a tampered row or a smuggled rank reddens), ids are
/// unique across main and backup, quotas re-derive from the recorded
/// pool via the same apportion code, floors hold, main rows sit in
/// audit order, and the pool digests still equal the frozen
/// candidate docs (the CI-persistent pool anchor; rank-order
/// continuation of the backups is a generation-time fact — it needs
/// the pool, which lives in the corpora, not the repo).
#[test]
fn t3_sample_verifies() {
    let doc = load(&eval_doc("t3-sample"));
    let pool_by: BTreeMap<String, u64> = doc["pool_by_lang"]
        .as_object()
        .expect("pool")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_u64().expect("n")))
        .collect();
    let q = quotas(&pool_by);
    assert_eq!(json!(q), doc["quotas"], "quotas drifted from the apportion");
    let main = doc["main"].as_array().expect("main");
    assert_eq!(main.len(), MAIN as usize, "main size");
    let mut seen = std::collections::BTreeSet::new();
    let mut by_lang: BTreeMap<&str, u64> = BTreeMap::new();
    let mut audits: Vec<String> = Vec::new();
    for row in main {
        check_row("main", row, &mut seen);
        *by_lang
            .entry(row["lang"].as_str().expect("lang"))
            .or_insert(0) += 1;
        audits.push(row_hash(AUDIT_DOMAIN, row));
    }
    assert!(audits.is_sorted(), "main rows not in audit order");
    for lang in LANGS {
        assert_eq!(by_lang[lang], q[lang], "{lang}: quota broken");
        assert!(by_lang[lang] >= FLOOR, "{lang}: floor broken");
        let bk = doc["backup"][lang].as_array().expect("backup");
        assert_eq!(bk.len(), BACKUP as usize, "{lang}: backup size");
        for row in bk {
            check_row(lang, row, &mut seen);
        }
    }
    for name in FROZEN_CORPORA {
        let name = name.map(str::to_string);
        let cand = load(&eval_doc(&doc_stem("t3-candidates", &name)));
        let corpus = name.as_deref().unwrap_or("self");
        assert_eq!(
            doc["pool_digests"][corpus], cand["pairs_sha256"],
            "{corpus}: pool anchor drifted from the frozen candidates"
        );
    }
}

/// Rank re-derivation + uniqueness of one frozen row — main and
/// backup rows pass the same throat.
fn check_row(scope: &str, row: &Value, seen: &mut std::collections::BTreeSet<String>) {
    let rank = row["rank"].as_str().expect("rank");
    assert_eq!(rank, row_hash(RANK_DOMAIN, row), "{scope}: rank forged");
    assert!(seen.insert(rank.to_string()), "{scope}: duplicate row");
}
