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

/// One pool row, already carrying everything an auditor needs.
struct PoolRow {
    row: Value,
    rank: String,
    lang: String,
}

/// The domain-separated hash of one row under `domain` — the ONE
/// derivation verify() repeats from the frozen fields (G4).
fn row_hash(domain: &str, row: &Value) -> String {
    let s = |k: &str| row[k].as_str().unwrap_or_else(|| panic!("{k}"));
    let n = |k: &str| row[k].as_i64().unwrap_or_else(|| panic!("{k}"));
    content_sha(&format!(
        "{domain}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        s("corpus"),
        s("tip"),
        s("a_path"),
        s("a_key"),
        n("a_nth"),
        s("b_path"),
        s("b_key"),
        n("b_nth"),
        s("source")
    ))
}

/// Walk one corpus's frozen candidate doc + live pass into pool rows,
/// asserting the digest anchor before a single row enters the pool.
fn corpus_pool(name: &Option<String>) -> (String, Vec<PoolRow>) {
    let a = eval_support::anchored_candidates(name);
    let (corpus, tip, c) = (a.corpus, a.tip, a.candidates);
    let rows = c
        .pairs
        .iter()
        .map(|p| {
            let row = sample_row(&c, p, &corpus, &tip);
            PoolRow {
                rank: row_hash(RANK_DOMAIN, &row),
                lang: row["lang"].as_str().expect("lang").to_string(),
                row,
            }
        })
        .collect();
    (corpus, rows)
}

/// The per-language pick: `take` rows into main, the next BACKUP
/// rows into that language's backup — global rank order, never
/// crossing languages (denominator top-up, floors protected).
fn pick(pool: &[PoolRow], lang: &str, take: usize) -> (Vec<Value>, Vec<Value>) {
    let ranked: Vec<&PoolRow> = pool.iter().filter(|r| r.lang == lang).collect();
    assert!(
        ranked.len() >= take + BACKUP as usize,
        "{lang}: pool too small for quota + backup"
    );
    let with_rank = |r: &&PoolRow| {
        let mut row = r.row.clone();
        row["rank"] = json!(r.rank);
        row
    };
    (
        ranked[..take].iter().map(with_rank).collect(),
        ranked[take..take + BACKUP as usize]
            .iter()
            .map(with_rank)
            .collect(),
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

#[test]
#[ignore] // needs all five corpus repositories
fn generate_t3_sample() {
    let mut pool: Vec<PoolRow> = Vec::new();
    let mut digests: BTreeMap<String, Value> = BTreeMap::new();
    for name in FROZEN_CORPORA {
        let name = name.map(str::to_string);
        let (corpus, rows) = corpus_pool(&name);
        let doc = load(&eval_doc(&doc_stem("t3-candidates", &name)));
        digests.insert(corpus, doc["pairs_sha256"].clone());
        pool.extend(rows);
    }
    let mut pool_by: BTreeMap<String, u64> = BTreeMap::new();
    for r in &pool {
        *pool_by.entry(r.lang.clone()).or_insert(0) += 1;
    }
    pool.sort_by(|x, y| x.rank.cmp(&y.rank));
    let q = quotas(&pool_by);
    let (mut main, mut backup): (Vec<Value>, BTreeMap<&str, Vec<Value>>) =
        (Vec::new(), BTreeMap::new());
    for lang in LANGS {
        let (m, b) = pick(&pool, lang, q[lang] as usize);
        main.extend(m);
        backup.insert(lang, b);
    }
    // auditors see audit order, never rank order (independent domain)
    main.sort_by_key(|r| row_hash(AUDIT_DOMAIN, r));
    let doc = json!({
        "schema": "ce.eval-t3-sample/1.0.0",
        "domains": {"rank": RANK_DOMAIN, "audit": AUDIT_DOMAIN},
        "generated_from": generated_from(),
        "method": "hash-ranked sample over the five frozen candidate universes: \
                   rank = sha256(rank domain | corpus | tip | pair identity | \
                   source), no RNG, no clock; 100 main rows = floor 15 per unit \
                   language (four — markdown has no units by design) + 40 seats \
                   by largest remainder over pool shares; backup 20 per language \
                   continues each language's rank order and never crosses \
                   languages (denominator top-up, floors protected). Pool \
                   membership is digest-bound to the t3-candidates docs; rows \
                   are listed in the independent audit-domain order.",
        "pool_by_lang": pool_by,
        "pool_digests": digests,
        "quotas": q,
        "main": main,
        "backup": backup,
    });
    let path = eval_doc("t3-sample");
    write_doc(&path, &doc, &format!("{path} written"));
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
