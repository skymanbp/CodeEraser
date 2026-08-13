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

/// The audit ground truth, resolved BY NAME on day one (the M5-1d C3
/// lesson: a gate that resolves via the active corpus reads the
/// wrong book and stays green). include_str! makes a missing corpus
/// a compile error — a whole corpus can never go silently blind
/// (G10). Shared by the audit gate and the precision instrument.
pub fn review_text(corpus: &str) -> &'static str {
    match corpus {
        "cobra" => include_str!("../eval_graph_review/cobra.json"),
        "requests" => include_str!("../eval_graph_review/requests.json"),
        "ripgrep" => include_str!("../eval_graph_review/ripgrep.json"),
        "self" => include_str!("../eval_graph_review/self.json"),
        "zod" => include_str!("../eval_graph_review/zod.json"),
        other => panic!("no review table for {other}"),
    }
}

pub fn review_doc(corpus: &str) -> serde_json::Value {
    serde_json::from_str(review_text(corpus)).expect(corpus)
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
