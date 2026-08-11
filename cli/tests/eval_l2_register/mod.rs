//! Relocation-register coverage for the L2 bar: the reviewed 35-unit
//! register (eval_commit_review) checked against what the pipeline
//! actually names — block-derived relocations plus the delta lines'
//! own unit attributions (extension and source-attribution lines
//! carry unit ownership in MovedLine but never appear in blocks; the
//! gate caught mark_ready, whose whole body arrived via extension,
//! unnamed under blocks-only naming).
#![allow(dead_code)]

use crate::eval_commit_review as review;
use crate::eval_l2_parts::delta_moved;
use codeeraser::fourclass::batch::BatchClassification;
use codeeraser::fourclass::session;
use serde_json::Value;

/// The batch relocations in the session report shape, named by the
/// gt pair paths.
pub fn relocations_json(batch: &BatchClassification, gt_rows: &[Value]) -> Value {
    let pairs: Vec<session::PathPair> = gt_rows
        .iter()
        .map(|gp| {
            (
                gp["before"].as_str().map(String::from),
                gp["after"].as_str().map(String::from),
            )
        })
        .collect();
    session::report_json(batch, &pairs)["relocations"].clone()
}

/// Unit keys attributed to the L2-over-L1 delta lines themselves.
pub fn moved_units(l1: &BatchClassification, l2: &BatchClassification) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (_, m) in delta_moved(l1, l2) {
        if let Some(u) = &m.unit
            && !out.contains(u)
        {
            out.push(u.clone());
        }
    }
    out.sort();
    out
}

/// Every unit key a committed row names, either through a block
/// relocation end or a delta line's own attribution.
fn row_keys(r: &Value) -> Vec<String> {
    let mut keys: Vec<String> = r["relocations"]
        .as_array()
        .map(|rels| {
            rels.iter()
                .flat_map(|rel| [rel["from_unit"].clone(), rel["to_unit"].clone()])
                .filter_map(|u| u.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if let Some(units) = r["moved_units"].as_array() {
        keys.extend(units.iter().filter_map(|u| u.as_str().map(String::from)));
    }
    keys
}

/// Register entries not named by their commit's row. Pure const data
/// on the register side, so the CI gate re-runs this without git.
pub fn register_misses(rows: &[Value]) -> Vec<String> {
    let mut misses = Vec::new();
    for r in rows {
        let Some(sha) = r["sha"].as_str() else {
            continue;
        };
        let keys = row_keys(r);
        for entry in review::units_for(sha) {
            for unit in entry["units"].as_array().expect("units") {
                let name = unit.as_str().expect("unit name");
                // "~name" = reviewed as adapted-in-flight: zero
                // line-identical body lines survive (verified against
                // the raw diff), so line-level naming is structurally
                // impossible and the entry documents exactly that.
                if name.starts_with('~') {
                    continue;
                }
                let hit = keys
                    .iter()
                    .any(|k| k == name || k.starts_with(&format!("{name}/")));
                if !hit {
                    misses.push(format!("{} {name}", &sha[..9]));
                }
            }
        }
    }
    misses
}
