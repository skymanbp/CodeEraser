//! Ground-truth integrity gate: labels-v1.json must equal
//! prelabels-v1.json on every row EXCEPT the declared corrections, and
//! every row (corrected or not) must preserve the prelabel numstat
//! sums. Runs in CI on the two committed files — the local review
//! notes are not needed to verify the published ground truth.

mod eval_support;

use eval_support::{CLASSES, by_id, load};
use serde_json::Value;
use std::collections::HashMap;

/// Uncorrected rows must match the prelabel exactly; every row's class
/// split must add back up to the prelabel numstat totals.
fn check_row(id: &str, row: &Value, p: &Value, corrected: bool) {
    for class in CLASSES {
        if !corrected {
            assert_eq!(row[class], p[class], "{id}: {class} drifted");
        }
    }
    assert_eq!(
        row["added_novel"].as_u64().unwrap() + row["added_moved"].as_u64().unwrap(),
        p["numstat_added"].as_u64().unwrap(),
        "{id}: added sum"
    );
    assert_eq!(
        row["removed_deleted"].as_u64().unwrap() + row["removed_moved"].as_u64().unwrap(),
        p["numstat_deleted"].as_u64().unwrap(),
        "{id}: removed sum"
    );
}

/// Every declared correction must match both the published label row
/// and the prelabel it claims to have corrected.
fn check_corrections(
    corrections: &HashMap<&str, &Value>,
    labels: &HashMap<&str, &Value>,
    pres: &HashMap<&str, &Value>,
) {
    for (id, c) in corrections {
        let l = labels
            .get(id)
            .unwrap_or_else(|| panic!("{id}: correction for unknown row"));
        for class in CLASSES {
            assert_eq!(
                c["final"][class], l[class],
                "{id}: correction/label mismatch"
            );
            assert_eq!(
                c["prelabel"][class], pres[id][class],
                "{id}: recorded prelabel"
            );
        }
    }
}

fn check_summary(s: &Value, n_corrections: u64, totals: &HashMap<&str, u64>) {
    assert_eq!(s["samples"].as_u64(), Some(200));
    assert_eq!(s["corrections"].as_u64(), Some(n_corrections));
    assert_eq!(s["prelabel_exact"].as_u64(), Some(200 - n_corrections));
    for class in CLASSES {
        assert_eq!(
            s["totals"][class].as_u64(),
            Some(totals[class]),
            "summary total {class}"
        );
    }
}

#[test]
fn labels_consistent_with_prelabels_modulo_corrections() {
    let pre = load("../contracts/eval/prelabels-v1.json");
    let lab = load("../contracts/eval/labels-v1.json");
    assert_eq!(
        lab["source_manifest"], pre["source_manifest"],
        "same freeze"
    );

    let pres = by_id(&pre, "prelabels");
    let labels = by_id(&lab, "labels");
    let corrections = by_id(&lab, "corrections");
    assert_eq!(labels.len(), 200, "label rows");

    let mut totals: HashMap<&str, u64> = HashMap::new();
    for (id, row) in &labels {
        let p = pres.get(id).unwrap_or_else(|| panic!("{id}: no prelabel"));
        let corrected = row["prelabel_ok"] == Value::Bool(false);
        assert_eq!(
            corrected,
            corrections.contains_key(id),
            "{id}: flag vs list"
        );
        for class in CLASSES {
            *totals.entry(class).or_insert(0) += row[class].as_u64().expect(class);
        }
        check_row(id, row, p, corrected);
    }
    check_corrections(&corrections, &labels, &pres);
    check_summary(&lab["summary"], corrections.len() as u64, &totals);
}
