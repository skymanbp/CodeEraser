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

use eval_support::{TRUTH_KEYWORDS, eval_doc, frozen_docs, load, review_doc};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// "mismatch" is NOT truth vocabulary (TRUTH_KEYWORDS): an auditor
// reporting a frozen site it could not find must redden the gate,
// not pass as a path.
const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// The corpus's frozen file universe, from its in-repo slice doc —
/// path-shaped truths bind to real universe members with zero git
/// (review F4: any single-line string used to pass as "path-shaped";
/// the truth column, 2d's entire deliverable, had no CI validation.
/// The review verified all 39 path truths are in-universe, so the
/// strong binding is safe to freeze).
fn universe_paths(corpus: &str) -> BTreeSet<String> {
    let stem = match corpus {
        "self" => "graph-slice".into(),
        other => format!("graph-slice-{other}"),
    };
    let path = eval_doc(&stem);
    assert!(
        frozen_docs("graph-slice").contains(&path),
        "{corpus}: no slice doc"
    );
    load(&path)["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().expect("path").to_string())
        .collect()
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
    let universe = universe_paths(corpus);
    let audited = doc["rows"].as_array().expect("rows");
    let sampled: BTreeMap<&str, &Value> = sample_rows
        .iter()
        .filter(|r| r["corpus"].as_str() == Some(corpus))
        .map(|r| (r["rank"].as_str().expect("rank"), r))
        .collect();
    assert_eq!(audited.len(), sampled.len(), "{corpus}: audited row count");
    let mut seen = BTreeSet::new();
    for row in audited {
        let (rank, s) = eval_support::bijective_row(corpus, row, &sampled, &mut seen);
        eval_support::assert_identity_echo(corpus, rank, row, s);
        assert_eq!(doc["tip"], s["commit"], "{corpus}: tip vs sampled commit");
        check_verdict(corpus, row, &universe);
    }
}

/// truth ∈ keywords, or an in-corpus target BOUND to the frozen file
/// universe: the base (before any #unit) must be ".", a universe
/// file, or a directory prefix of one. why must be substantive (the
/// mechanism-naming requirement is enforced at review time — the
/// gate holds the floor: non-empty, not a stub).
fn check_verdict(corpus: &str, row: &Value, universe: &BTreeSet<String>) {
    let truth = row["truth"].as_str().expect("truth");
    assert_ne!(
        truth, "mismatch",
        "{corpus}: frozen site not found by auditor"
    );
    if !TRUTH_KEYWORDS.contains(&truth) {
        let base = truth.split('#').next().expect("base");
        let bound = base == "."
            || universe.contains(base)
            || universe.iter().any(|p| p.starts_with(&format!("{base}/")));
        assert!(
            bound,
            "{corpus}:{} truth {truth:?} names no frozen-universe member",
            row["line"]
        );
    }
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

/// Counterfactual (the G9 discipline): every advertised refusal is
/// exercised, not assumed — table-driven after the first three
/// copy-pasted mutations tripped the repo's own dedup ratchet
/// (reviews F10/F4 added the last three cases).
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
    let field_mutations: [(&str, &str, &str); 5] = [
        ("rank", "not-a-sampled-rank", "phantom rank"),
        ("why", "yes", "stub why"),
        ("truth", "mismatch", "auditor mismatch"),
        ("spec", "phantom-spec", "identity echo drift"),
        ("truth", "probably/exceptions.py", "non-universe truth"),
    ];
    for (field, value, label) in field_mutations {
        let mut doc = pristine.clone();
        doc["rows"][0][field] = Value::from(value);
        assert!(refused(&doc), "{label} must refuse");
    }
    let mut missing = pristine.clone();
    missing["rows"].as_array_mut().expect("rows").remove(0);
    assert!(refused(&missing), "missing row must refuse");
}
