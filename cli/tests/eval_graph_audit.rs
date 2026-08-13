//! M5-2d: the frozen audit — precision ground truth over the sampled
//! sites (design §5 人工审计流; executor = independent Opus subagents
//! per user delegation 2026-08-12, mirroring the M4-2c precedent).
//! The audit tables are DATA under eval_graph_review/, one file per
//! corpus, resolved BY NAME on day one (the M5-1d C3 lesson: a gate
//! that resolves via the active corpus reads the wrong book and stays
//! green). include_str! makes a missing corpus a compile error — a
//! whole corpus can never go silently blind (G10).
//!
//! The ancestry half of G13 (sampled → audited → resolver) lives in
//! graph_provenance.rs.

mod eval_support;

use eval_support::{eval_doc, load};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// Non-path truth verdicts (design §5 vocabulary). Everything else
/// must be a repo-relative "path" or "path#unit". "mismatch" is NOT
/// vocabulary: an auditor reporting a frozen site it could not find
/// must redden the gate, not pass as a path.
const TRUTH_KEYWORDS: [&str; 4] = ["external", "dynamic", "ambiguous", "none"];

fn review_text(corpus: &str) -> &'static str {
    match corpus {
        "cobra" => include_str!("eval_graph_review/cobra.json"),
        "requests" => include_str!("eval_graph_review/requests.json"),
        "ripgrep" => include_str!("eval_graph_review/ripgrep.json"),
        "self" => include_str!("eval_graph_review/self.json"),
        "zod" => include_str!("eval_graph_review/zod.json"),
        other => panic!("no review table for {other}"),
    }
}

fn review_doc(corpus: &str) -> Value {
    serde_json::from_str(review_text(corpus)).expect(corpus)
}

/// One corpus checked against the frozen sample: embedded name and
/// tip, a rank bijection (phantom and missing rows equally loud),
/// verbatim echo of every identity field, and a verdict per row.
fn check_corpus(corpus: &str, doc: &Value, sample_rows: &[Value]) {
    assert_eq!(
        doc["corpus"].as_str(),
        Some(corpus),
        "{corpus}: embedded name"
    );
    let audited = doc["rows"].as_array().expect("rows");
    let sampled: BTreeMap<&str, &Value> = sample_rows
        .iter()
        .filter(|r| r["corpus"].as_str() == Some(corpus))
        .map(|r| (r["rank"].as_str().expect("rank"), r))
        .collect();
    assert_eq!(audited.len(), sampled.len(), "{corpus}: audited row count");
    let mut seen = BTreeSet::new();
    for row in audited {
        let rank = row["rank"].as_str().expect("rank");
        assert!(seen.insert(rank), "{corpus}: duplicate audit row {rank}");
        let s = sampled
            .get(rank)
            .unwrap_or_else(|| panic!("{corpus}: phantom audit row {rank}"));
        for key in ["path", "line", "nth", "kind", "spec"] {
            assert_eq!(row[key], s[key], "{corpus}/{rank}: {key} echo drifted");
        }
        assert_eq!(doc["tip"], s["commit"], "{corpus}: tip vs sampled commit");
        check_verdict(corpus, row);
    }
}

/// truth ∈ keywords, or a path-shaped in-corpus target; why must be
/// substantive (the mechanism-naming requirement is enforced at
/// review time — the gate holds the floor: non-empty, not a stub).
fn check_verdict(corpus: &str, row: &Value) {
    let truth = row["truth"].as_str().expect("truth");
    assert_ne!(
        truth, "mismatch",
        "{corpus}: frozen site not found by auditor"
    );
    let path_shaped = !truth.is_empty()
        && truth == truth.trim()
        && !truth.contains('\n')
        && !truth.contains('\\');
    assert!(
        TRUTH_KEYWORDS.contains(&truth) || path_shaped,
        "{corpus}:{} bad truth {truth:?}",
        row["line"]
    );
    let why = row["why"].as_str().expect("why").trim();
    assert!(
        why.len() >= 15,
        "{corpus}:{} why is not a mechanism: {why:?}",
        row["line"]
    );
}

/// The 2d exit criterion: every sampled row audited, per corpus,
/// with the identity echo intact — 100/100 with non-empty whys.
#[test]
fn audit_covers_sample_bijectively() {
    let sample = load(&eval_doc("graph-sample"));
    let rows = sample["rows"].as_array().expect("rows");
    let mut audited_total = 0;
    for corpus in CORPORA {
        let doc = review_doc(corpus);
        check_corpus(corpus, &doc, rows);
        audited_total += doc["rows"].as_array().expect("rows").len();
    }
    assert_eq!(audited_total, rows.len(), "audited != sampled");
}

/// site_gaps is a required field on every table — an ABSENT sweep is
/// indistinguishable from a sweep that found nothing, so the shape
/// itself is the "有交代" (empty array = looked, found none).
#[test]
fn audit_site_gaps_accounted() {
    for corpus in CORPORA {
        let doc = review_doc(corpus);
        let gaps = doc["site_gaps"]
            .as_array()
            .unwrap_or_else(|| panic!("{corpus}: site_gaps missing (empty allowed, absent not)"));
        for gap in gaps {
            let well_formed = gap["path"].as_str().is_some_and(|p| !p.is_empty())
                && gap["note"].as_str().is_some_and(|n| n.len() >= 10);
            assert!(well_formed, "{corpus}: malformed site gap {gap}");
        }
    }
}

/// Counterfactual (the G9 discipline): a phantom rank, a stub why,
/// and an auditor mismatch must each actually redden — asserted via
/// catch_unwind, not assumed.
#[test]
fn audit_refuses_tampering() {
    let sample = load(&eval_doc("graph-sample"));
    let rows = sample["rows"].as_array().expect("rows").clone();
    let refused = |doc: &Value| {
        let doc = doc.clone();
        let rows = rows.clone();
        std::panic::catch_unwind(move || check_corpus("cobra", &doc, &rows)).is_err()
    };
    let pristine = review_doc("cobra");
    assert!(!refused(&pristine), "pristine table must pass");
    let mut phantom = pristine.clone();
    phantom["rows"][0]["rank"] = Value::from("not-a-sampled-rank");
    assert!(refused(&phantom), "phantom rank must refuse");
    let mut stub = pristine.clone();
    stub["rows"][0]["why"] = Value::from("yes");
    assert!(refused(&stub), "stub why must refuse");
    let mut lost = pristine.clone();
    lost["rows"][0]["truth"] = Value::from("mismatch");
    assert!(refused(&lost), "auditor mismatch must refuse");
}
