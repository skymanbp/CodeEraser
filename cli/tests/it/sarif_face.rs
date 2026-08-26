//! The `--format sarif` projection (plan v2.14: an ENCODING of the
//! judged facts, not an interpretation — the 2026-08-19 retirement
//! ruling stands). Two faces carry it: scan findings and dedup clone
//! blocks. These gates pin the projection's vocabulary — version,
//! ruleId families, grade mapping, physical locations — so the CI
//! upload leg cannot silently drift off what code scanning ingests.

use crate::common::fixtures::{seed_clone_pair, tmp};
use crate::common::gates::run_expect;
use crate::common::run_ce;
use serde_json::Value;

fn sarif(stdout: &str) -> Value {
    let d: Value = serde_json::from_str(stdout).expect("sarif json");
    assert_eq!(d["version"], "2.1.0");
    assert_eq!(d["runs"][0]["tool"]["driver"]["name"], "CodeEraser");
    d["runs"][0]["results"].clone()
}

/// A 76-line function (past the 75 hard line) and a 55-line one
/// (past the 50 soft line) in one file: the fail grade must respell
/// as "error", the warn grade as "warning", each with the finding's
/// file and line under its ce.scan/ rule.
#[test]
fn scan_grades_respell_as_sarif_levels_with_locations() {
    let dir = tmp("sarif-scan");
    let long = |name: &str, n: usize| {
        let body: String = (0..n).map(|i| format!("    let x{i} = {i};\n")).collect();
        format!("fn {name}() {{\n{body}}}\n")
    };
    std::fs::write(dir.join("a.rs"), long("hard", 75) + &long("soft", 54)).unwrap();
    let out = run_ce(&dir, &["scan", ".", "--format", "sarif"]);
    assert_eq!(out.status.code(), Some(1), "a fail row still exits 1");
    let results = sarif(&String::from_utf8_lossy(&out.stdout));
    let of = |level: &str| -> Vec<&Value> {
        results
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["level"] == level && r["ruleId"] == "ce.scan/fn-lines")
            .collect()
    };
    let (err, warn) = (of("error"), of("warning"));
    assert_eq!((err.len(), warn.len()), (1, 1), "{results}");
    let loc = &err[0]["locations"][0]["physicalLocation"];
    assert_eq!(loc["artifactLocation"]["uri"], "a.rs");
    assert_eq!(loc["region"]["startLine"], 1);
    assert!(
        err[0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("fn-lines = 77 (limit 75) [hard]"),
        "{}",
        err[0]["message"]["text"]
    );
}

/// A clone block is budget-gated debt, not a per-block failure: it
/// rides as "note", with the pair's second span in relatedLocations
/// so the alert shows both ends.
#[test]
fn dedup_blocks_ride_as_notes_with_both_spans() {
    let dir = tmp("sarif-dedup");
    seed_clone_pair(&dir);
    let results = sarif(&run_expect(&dir, &["dedup", ".", "--format", "sarif"]));
    let r = &results[0];
    assert_eq!(r["level"], "note");
    assert_eq!(r["ruleId"], "ce.dedup/clone-block");
    let primary = &r["locations"][0]["physicalLocation"];
    let related = &r["relatedLocations"][0]["physicalLocation"];
    assert_eq!(primary["artifactLocation"]["uri"], "a.rs");
    assert!(primary["region"]["endLine"].as_u64() > primary["region"]["startLine"].as_u64());
    assert_eq!(related["artifactLocation"]["uri"], "b.rs");
}
