//! The universe-binding half of the sample gates: everything that
//! reads the frozen graph-slice docs (split from the crate root when
//! the instrument crossed the file budget). The sample must stay
//! bound to the live frozen universes — a re-frozen slice reddens the
//! sample instead of silently orphaning it.

use crate::eval_support::{by_field, doc_suffix, eval_doc, frozen_docs, load};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// (corpus tag, loaded slice doc) for every frozen universe, in
/// frozen file-name order (the generator writes sources in this
/// order and the gate zips against it). The self doc gets the
/// explicit "self" tag — hash payloads need a non-empty corpus field.
pub fn universes() -> Vec<(String, Value)> {
    frozen_docs("graph-slice")
        .iter()
        .map(|p| {
            let tag = doc_suffix(p, "graph-slice").unwrap_or_else(|| "self".into());
            (tag, load(p))
        })
        .collect()
}

/// Pool cell counts re-derived from the frozen slice summaries — the
/// gate-side twin of the generator's live pool count.
fn universe_cells(slices: &[(String, Value)]) -> BTreeMap<String, u64> {
    let mut cells = BTreeMap::new();
    for (_, doc) in slices {
        for (cell, n) in doc["summary"]["sites_by"].as_object().expect("sites_by") {
            *cells.entry(cell.clone()).or_insert(0) += n.as_u64().expect("count");
        }
    }
    cells
}

/// One row bound to its frozen universe: the file exists at the
/// pinned tip with the same lang, actually holds the sampled kind,
/// line/nth stay in range, spec is non-empty, and the running
/// per-(corpus,path,kind) multiplicity never exceeds the frozen
/// count.
fn bind_row(
    row: &Value,
    tips: &BTreeMap<&str, &str>,
    files: &BTreeMap<&str, HashMap<&str, &Value>>,
    seen: &mut BTreeMap<String, u64>,
) {
    let tag = row["corpus"].as_str().expect("corpus");
    let path = row["path"].as_str().expect("path");
    assert_eq!(
        row["commit"].as_str(),
        tips.get(tag).copied(),
        "{tag}: commit is not the tip"
    );
    let file = files[tag]
        .get(path)
        .unwrap_or_else(|| panic!("{tag}/{path}: not in the frozen universe"));
    assert_eq!(row["lang"], file["lang"], "{tag}/{path}: lang mismatch");
    let kind = row["kind"].as_str().expect("kind");
    let cap = file["sites"][kind].as_u64().unwrap_or(0);
    assert!(cap >= 1, "{tag}/{path}: no frozen {kind} site");
    let total: u64 = file["sites"]
        .as_object()
        .expect("sites")
        .values()
        .map(|v| v.as_u64().expect("n"))
        .sum();
    // honest bound (review F8): the slice docs carry no per-file line
    // counts, so `line` has no checkable upper bound here and `nth`
    // (per-line ordinal across kinds) is bounded only by the file
    // total — tampering with either is caught by the rank hash in
    // verify_row, not by this range check
    assert!(
        row["line"].as_u64().expect("line") >= 1 && row["nth"].as_u64().expect("nth") < total,
        "{tag}/{path}: line/nth out of range"
    );
    assert!(
        !row["spec"].as_str().expect("spec").is_empty(),
        "{tag}/{path}: empty spec"
    );
    let n = seen.entry(format!("{tag}|{path}|{kind}")).or_insert(0);
    *n += 1;
    assert!(
        *n <= cap,
        "{tag}/{path}: {kind} sampled beyond the frozen count {cap}"
    );
}

/// Universe gate: sources pin tips and totals, the allocation
/// re-derives from the slice summaries through the same
/// apportionment code, and every row (primary and backup) binds to a
/// frozen file.
#[test]
fn graph_sample_matches_universe() {
    let doc = load(&eval_doc("graph-sample"));
    let slices = universes();
    let sources = doc["sources"].as_array().expect("sources");
    assert_eq!(sources.len(), slices.len(), "corpus set drifted");
    for (src, (tag, slice)) in sources.iter().zip(&slices) {
        assert_eq!(src["corpus"].as_str(), Some(tag.as_str()), "source order");
        assert_eq!(
            src["tip"], slice["corpus"]["tip"],
            "{tag}: universe re-frozen — re-sample"
        );
        assert_eq!(
            src["total_sites"], slice["summary"]["total_sites"],
            "{tag}: universe totals drifted"
        );
    }
    assert_eq!(
        doc["allocation"],
        json!(super::quotas_from_counts(&universe_cells(&slices))),
        "allocation does not re-derive from the frozen universes"
    );
    let tips: BTreeMap<&str, &str> = slices
        .iter()
        .map(|(tag, doc)| (tag.as_str(), doc["corpus"]["tip"].as_str().expect("tip")))
        .collect();
    let files: BTreeMap<&str, HashMap<&str, &Value>> = slices
        .iter()
        .map(|(tag, doc)| (tag.as_str(), by_field(doc, "files", "path")))
        .collect();
    let mut seen = BTreeMap::new();
    let rows = doc["rows"].as_array().expect("rows");
    let backups = doc["backups"].as_array().expect("backups");
    for row in rows.iter().chain(backups.iter()) {
        bind_row(row, &tips, &files, &mut seen);
    }
}
