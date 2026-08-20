//! M5-3f precision instrument: the frozen 100-pair sample, judged by
//! the SHIPPED T3 judge against the materialized pinned trees, scored
//! against the frozen independent audit (design vol.3 §9.2). The
//! candidate universe froze in 3c before the judge existed and the
//! audit froze before any scoring ran (dedup_provenance.rs holds the
//! git legs), so neither the denominator nor the truth could bend to
//! the judge. A degraded core reply refuses at the wire layer
//! (wire::parse_result — T-G12); all row/summary containers are
//! BTreeMaps, so file order cannot leak into bytes (T-G11).
//!
//! Generate (all five corpora, one invocation; corpora repos under
//! .ce-eval/corpora/; needs CE_CORE_BIN; re-blessing a grown wrong
//! ledger needs CE_ACCEPT_T3=1):
//!   cargo test --test eval_t3_precision -- --ignored --nocapture

mod eval_support;
mod eval_t3_precision_parts;

use eval_support::{PrecisionAgg, str_pairs, t3f};
use eval_t3_precision_parts as parts;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One row's stored facts re-derived (T-G4/G6 + the θ pin): identity
/// echo, truth echo against the frozen audit, judged_clone recomputed
/// from raw ted at the product threshold binding, verdict recomputed
/// from its own row.
fn check_row(corpus: &str, rank: &str, row: &Value, s: &Value, truths: &BTreeMap<&str, &str>) {
    for key in t3f::T3_IDENTITY {
        assert_eq!(row[key], s[key], "{corpus}/{rank}: {key} echo drifted");
    }
    let judged_clone = row["judged_clone"].as_bool().expect("judged_clone");
    if let (Some(ted), Some(n1), Some(n2)) =
        (row["ted"].as_i64(), row["n1"].as_i64(), row["n2"].as_i64())
    {
        assert_eq!(
            judged_clone,
            codeeraser::dedup::t3::is_clone(ted, n1, n2),
            "{corpus}/{rank}: judged_clone contradicts its raw ted (θ pin)"
        );
    } else {
        assert!(
            !judged_clone,
            "{corpus}/{rank}: clone verdict without a score"
        );
    }
    eval_support::assert_row_verdict(corpus, rank, row, truths, |truth| {
        parts::verdict_of(
            row["judgment"].as_str().expect("judgment"),
            judged_clone,
            truth,
        )
    });
}

/// One corpus's precision doc checked end to end. The doc's corpus
/// header is the {name, tip} envelope (not the audit tables' bare
/// string), so the frame's pieces run directly: sample filter,
/// bijection walk, per-row re-derivation with the tip pinned.
fn check_precision_doc(corpus: &str, doc: &Value, sample: &Value) {
    let name = doc["corpus"]["name"].as_str();
    assert_eq!(name, (corpus != "self").then_some(corpus), "{corpus}: name");
    eval_support::assert_summary_rederived(corpus, doc, parts::rescore);
    let rows = doc["rows"].as_array().expect("rows");
    let review = t3f::t3_review_doc(corpus);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let sampled = t3f::corpus_mains(sample, corpus);
    eval_support::each_audited_row(corpus, rows, &sampled, |rank, row, s| {
        assert_eq!(doc["corpus"]["tip"], s["tip"], "{corpus}: tip pin");
        check_row(corpus, rank, row, s, &truths);
    });
    eval_support::assert_verdict_conservation(corpus, doc, &parts::VERDICTS);
}

/// Aggregate one doc and apply the per-corpus gate on the answered
/// denominator (the family's T-G2 semantics: answered = judged
/// clone), plus the frozen output floor (T-G14).
fn corpus_gate(path: &str, sample: &Value, agg: &mut PrecisionAgg) {
    let (key, doc, c, w) =
        eval_support::open_scored_doc(path, "t3-precision", sample, agg, check_precision_doc);
    let rows = doc["rows"].as_array().expect("rows");
    agg.rows += rows.len() as u64;
    eval_support::tally_field(rows, "lang", &mut agg.by_lang);
    eval_support::assert_corpus_precision(&key, (c, w), c + w, 0.85, "T-G2");
    let floor = doc["constants"]["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        doc["universe"]["clones"].as_u64().expect("n") >= floor,
        "{key}: frozen doc below its output floor (T-G14)"
    );
}

/// The 3f contract and its refusals in one battery: every frozen
/// corpus doc present (T-G10), 100 judged rows (T-G3), per-language
/// floors (T-G5), precision >= 0.85 overall and per corpus (T-G2),
/// output floors (T-G14), then T-G9 — the shared mutation table
/// (row-0-robust: a value equal to the stored one would make "must
/// refuse" vacuous) plus the θ-pin and cooked-summary cases below.
const SPEC: eval_support::FamilySpec = eval_support::FamilySpec {
    family: "t3-precision",
    sample: "t3-sample",
    gate: 0.85,
    tag: "T-G2 contract",
    total: 100,
    slice_floor: 15,
    mutations: &[
        ("rank", "not-a-sampled-rank", "phantom rank"),
        ("truth", "not-a-truth", "cooked truth echo"),
        ("a_key", "phantom-key", "identity echo drift"),
    ],
};

#[test]
fn precision_contract_and_refusals() {
    eval_support::assert_precision_family(
        &SPEC,
        corpus_gate,
        &check_precision_doc,
        |pristine, refused| {
            let scored = pristine["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .position(|r| r["ted"].is_i64())
                .expect("a scored row");
            let mut stubbed = pristine.clone();
            let flip = !stubbed["rows"][scored]["judged_clone"]
                .as_bool()
                .expect("b");
            stubbed["rows"][scored]["judged_clone"] = json!(flip);
            assert!(
                refused(&stubbed),
                "stubbed threshold (flipped judged_clone) must refuse"
            );
            let mut cooked = pristine.clone();
            cooked["summary"]["verdicts"]["wrong"] = json!(0);
            assert!(refused(&cooked), "cooked summary must refuse");
        },
    );
}

// ——— The frozen sample's OWN integrity (M5-3c) ———
// Restored from eval_t3_sample.rs (review 2026-08-20 #10): the v0.5.0
// test consolidation retired that file as a one-shot generator, but
// its verification leg was a LIVE corpora-free CI gate — without it a
// mutated t3-sample-v1.json still scores. Same family, same doc, so
// the battery lives here now; the generator stays retired.

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
/// derivation the gate repeats from the frozen fields (G4).
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

/// Rank re-derivation + uniqueness of one frozen row — main and
/// backup rows pass the same throat.
fn check_sample_row(scope: &str, row: &Value, seen: &mut std::collections::BTreeSet<String>) {
    let rank = row["rank"].as_str().expect("rank");
    assert_eq!(rank, row_hash(RANK_DOMAIN, row), "{scope}: rank forged");
    assert!(seen.insert(rank.to_string()), "{scope}: duplicate row");
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
    let doc = eval_support::load(&eval_support::eval_doc("t3-sample"));
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
        check_sample_row("main", row, &mut seen);
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
            check_sample_row(lang, row, &mut seen);
        }
    }
    for name in eval_support::FROZEN_CORPORA {
        let name = name.map(str::to_string);
        let cand = eval_support::load(&eval_support::eval_doc(&eval_support::doc_stem(
            "t3-candidates",
            &name,
        )));
        let corpus = name.as_deref().unwrap_or("self");
        assert_eq!(
            doc["pool_digests"][corpus], cand["pairs_sha256"],
            "{corpus}: pool anchor drifted from the frozen candidates"
        );
    }
}
