//! Graph-instrument shared surface: the frozen scope classifier,
//! corpus selection and corpus set — ONE binding for the slice
//! instrument (eval_graph.rs) and the precision instrument
//! (eval_graph_precision.rs), so the two can never drift apart on
//! what "in scope" means (the throat-drift shape this support tree
//! exists to prevent).

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::git_run;

/// Two string fields of every row under `key`, as a map — the shape
/// every graph doc keys its rows by (rank→truth, path→sha256).
pub fn str_pairs<'a>(doc: &'a Value, key: &str, k1: &str, k2: &str) -> BTreeMap<&'a str, &'a str> {
    doc[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|r| (r[k1].as_str().expect(k1), r[k2].as_str().expect(k2)))
        .collect()
}

/// The frozen-corpus-set anchor (G10): the docs of one family must
/// cover exactly FROZEN_CORPORA — a deleted or renamed doc reddens
/// CI instead of blinding it. Shared by the slice and precision
/// gates.
pub fn assert_frozen_corpus_set(family: &str) -> Vec<String> {
    let docs = super::frozen_docs(family);
    let mut names: Vec<Option<String>> =
        docs.iter().map(|p| super::doc_suffix(p, family)).collect();
    names.sort();
    let expected: Vec<Option<String>> = FROZEN_CORPORA
        .iter()
        .map(|n| n.map(str::to_string))
        .collect();
    assert_eq!(names, expected, "frozen {family} corpus set drifted (G10)");
    docs
}

/// Verbatim identity echo of one derived row against its sampled row
/// — the G4 discipline both the audit and precision gates enforce.
pub fn assert_identity_echo(corpus: &str, rank: &str, row: &Value, sampled: &Value) {
    for key in ["path", "line", "nth", "kind", "spec"] {
        assert_eq!(
            row[key], sampled[key],
            "{corpus}/{rank}: {key} echo drifted"
        );
    }
}

/// The rank-bijection lookup (G4/G7): a duplicate row and a phantom
/// row are equally loud — shared by the audit and precision gates.
pub fn bijective_row<'a>(
    corpus: &str,
    row: &'a Value,
    sampled: &BTreeMap<&'a str, &'a Value>,
    seen: &mut BTreeSet<&'a str>,
) -> (&'a str, &'a Value) {
    let rank = row["rank"].as_str().expect("rank");
    assert!(seen.insert(rank), "{corpus}: duplicate row {rank} (G7)");
    let s = sampled
        .get(rank)
        .unwrap_or_else(|| panic!("{corpus}: phantom row {rank} (G4)"));
    (rank, s)
}

/// Frozen graph-universe scope: canonical extensions only (variant
/// suffixes stay out on every corpus — the COMMIT_SCOPE argument:
/// one frozen scope keeps corpora comparable), minus machine-local
/// memory/. The crosscheck fixture islands are deliberately IN scope
/// even though ce.toml excludes them from the product walk: their
/// imports have no in-corpus target, so they are the designed-in
/// negative control (design §5, judge defect D2).
pub const SCOPE_EXTS: [&str; 5] = ["go", "md", "py", "rs", "ts"];
pub const SCOPE_EXCLUDES: [&str; 2] = ["memory/", "cli/memory/"];

/// The frozen corpus set — a deleted or renamed doc reddens CI
/// instead of blinding it (design G10; Opus review: the first gate
/// held no per-corpus anchor at all). Sorted; None = the self doc.
pub const FROZEN_CORPORA: [Option<&str>; 5] = [
    None,
    Some("cobra"),
    Some("requests"),
    Some("ripgrep"),
    Some("zod"),
];

/// The self universe: pinned at the commit holding the CURRENT
/// detector, so regeneration is reproducible regardless of later
/// history. A detector change bumps this pin and re-freezes the
/// slice (design RG3 — first fired by the M5-2b-iii hardening batch,
/// which replaced the 2b-i pin eb5fe24).
pub const GRAPH_SELF_TIP: &str = "60f73e3bea7681721a2f572e64788948a17830f6";

/// Scope test for one tree path: Ok(lang code) or the itemized
/// exclusion category the slice doc reports.
pub fn classify_path(path: &str) -> Result<&'static str, &'static str> {
    if SCOPE_EXCLUDES.iter().any(|p| path.starts_with(p)) {
        return Err("excluded_prefix");
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return Err("other_extension");
    };
    if stem.is_empty() {
        return Err("other_extension"); // dotfiles (.gitignore, …)
    }
    match ext {
        "go" => Ok("go"),
        "md" => Ok("md"),
        "py" => Ok("py"),
        "rs" => Ok("rs"),
        "ts" => Ok("ts"),
        "tsx" | "mts" | "cts" | "markdown" => Err("variant_extension"),
        _ => Err("other_extension"),
    }
}

/// Non-path truth verdicts (design §5 vocabulary). Everything else
/// must be a repo-relative "path" or "path#unit".
pub const TRUTH_KEYWORDS: [&str; 4] = ["external", "dynamic", "ambiguous", "none"];

/// The graph family's mount into the ONE review registry
/// (auditgen::REVIEWS — the M5-1d C3 lesson: a gate that resolves
/// via the active corpus reads the wrong book and stays green).
pub fn review_doc(corpus: &str) -> serde_json::Value {
    super::review_of("graph", corpus)
}

/// The rows of one corpus, in frozen order — the ONE sample filter
/// every audit/precision family selects rows with (three families
/// re-grew it independently before the ratchet paired them).
pub fn of_corpus<'a>(rows: &'a [Value], corpus: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|r| r["corpus"].as_str() == Some(corpus))
        .collect()
}

/// Every promised seat must hold a nonzero count — a vocabulary
/// class or coverage lane that silently empties is how a gate goes
/// blind while staying green. Shared by the slice coverage gate and
/// the audit vocabulary-seat gate.
pub fn assert_nonzero_seats(counts: &BTreeMap<String, u64>, seats: &[&str], what: &str) {
    for seat in seats {
        assert!(
            counts.get(*seat).copied().unwrap_or(0) > 0,
            "{seat}: {what}"
        );
    }
}

/// site_gaps/unit_gaps: a REQUIRED field on every audit table — an
/// absent sweep is indistinguishable from a sweep that found nothing,
/// so the shape itself carries the "looked, found none" claim. One
/// checker for every audit family.
pub fn assert_gaps_accounted(corpus: &str, doc: &Value, key: &str) {
    let gaps = doc[key]
        .as_array()
        .unwrap_or_else(|| panic!("{corpus}: {key} missing (empty allowed, absent not)"));
    for gap in gaps {
        let well_formed = gap["path"].as_str().is_some_and(|p| !p.is_empty())
            && gap["note"].as_str().is_some_and(|n| n.len() >= 10);
        assert!(well_formed, "{corpus}: malformed {key} row {gap}");
    }
}

/// Does `check` panic on this doc? The G9 counterfactual primitive
/// every tamper battery runs on.
pub fn doc_refused(doc: &Value, check: &dyn Fn(&Value)) -> bool {
    let doc = doc.clone();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || check(&doc))).is_err()
}

/// The table-driven half of every audit tamper battery: pristine
/// passes, each single-field mutation of row 0 refuses, a dropped
/// row refuses. Families add their bespoke cases via doc_refused.
pub fn assert_tampering_refused(
    pristine: &Value,
    mutations: &[(&str, &str, &str)],
    check: &dyn Fn(&Value),
) {
    assert!(!doc_refused(pristine, check), "pristine table must pass");
    for (field, value, label) in mutations {
        let mut doc = pristine.clone();
        doc["rows"][0][*field] = Value::from(*value);
        assert!(doc_refused(&doc, check), "{label} must refuse");
    }
    let mut missing = pristine.clone();
    missing["rows"].as_array_mut().expect("rows").remove(0);
    assert!(doc_refused(&missing, check), "missing row must refuse");
}

/// The per-corpus audited↔sampled bijection walk (G3/G4/G7): count
/// equality, then every audited row resolved against its sampled row
/// and deduped; row CONTENT stays with the calling family.
pub fn each_audited_row<'a>(
    corpus: &str,
    audited: &'a [Value],
    sample_rows: &[&'a Value],
    mut f: impl FnMut(&'a str, &'a Value, &'a Value),
) {
    let sampled: BTreeMap<&str, &Value> = sample_rows
        .iter()
        .map(|r| (r["rank"].as_str().expect("rank"), *r))
        .collect();
    assert_eq!(audited.len(), sampled.len(), "{corpus}: audited row count");
    let mut seen = BTreeSet::new();
    for row in audited {
        let (rank, s) = bijective_row(corpus, row, &sampled, &mut seen);
        f(rank, row, s);
    }
}

/// One audit table's whole frame: embedded corpus name, rows
/// extracted, this corpus's sample selected, bijection walked — the
/// calling family's closure checks row content only (the frame
/// itself was the last shape the ratchet caught the two audit
/// families sharing).
pub fn check_audit_frame<'a>(
    corpus: &str,
    doc: &'a Value,
    sample_rows: &'a [Value],
    f: impl FnMut(&'a str, &'a Value, &'a Value),
) {
    assert_eq!(
        doc["corpus"].as_str(),
        Some(corpus),
        "{corpus}: embedded name"
    );
    let audited = doc["rows"].as_array().expect("rows");
    let sampled = of_corpus(sample_rows, corpus);
    each_audited_row(corpus, audited, &sampled, f);
}

/// (corpus name, pinned tree OID). Self unless CE_SLICE_REPO points
/// elsewhere; external corpora must name themselves and pin their
/// tip (rev-parsed to a full OID — a movable rev would make the doc
/// unreproducible).
pub fn graph_corpus() -> (Option<String>, String) {
    let get = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    if std::env::var("CE_SLICE_REPO").is_err() {
        return (None, GRAPH_SELF_TIP.into());
    }
    let name = get("CE_GRAPH_NAME").expect("CE_SLICE_REPO needs CE_GRAPH_NAME");
    let tip = get("CE_GRAPH_TIP").expect("CE_SLICE_REPO needs CE_GRAPH_TIP (pinned universe)");
    let full = git_run(
        &["rev-parse", "--verify", &format!("{tip}^{{commit}}")],
        false,
    );
    (Some(name), full.trim().to_string())
}
