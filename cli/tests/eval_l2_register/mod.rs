//! Relocation-register coverage for the L2 bar: the reviewed 35-unit
//! register (eval_commit_review) checked against what the pipeline
//! actually names — block-derived relocations plus the delta lines'
//! own unit attributions (extension and source-attribution lines
//! carry unit ownership in MovedLine but never appear in blocks; the
//! gate caught mark_ready, whose whole body arrived via extension,
//! unnamed under blocks-only naming).

use crate::eval_commit_review as review;
use serde_json::Value;

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

/// One relocation edge's unit bases, both ends (an adapted-in-flight
/// move renames: rebuild_proxies/3 -> resolve_proxies/3 — either end
/// may carry the registered name).
fn rel_bases(rel: &Value) -> Vec<String> {
    ["from_unit", "to_unit"]
        .iter()
        .filter_map(|k| rel[k].as_str())
        .map(|u| u.rsplit_once('/').map_or(u, |(b, _)| b).to_string())
        .collect()
}

/// Whether a review row's units array names `unit` (modulo the ~
/// adaptation marker) — the lookup both registers share.
fn names_unit(e: &Value, unit: &str) -> bool {
    e["units"]
        .as_array()
        .expect("units")
        .iter()
        .any(|u| u.as_str().expect("unit").trim_start_matches('~') == unit)
}

/// Walk committed rows by sha — the frame both register checks share.
fn each_sha<'a>(rows: &'a [Value], mut f: impl FnMut(&'a str, &'a Value)) {
    for r in rows {
        if let Some(sha) = r["sha"].as_str() {
            f(sha, r);
        }
    }
}

/// Edge-level GT (M5-1c-iii): every pipeline relocation whose unit is
/// covered by the reviewed edge table must name a source file the
/// review verified — an unverified source is an INVENTED edge. Units
/// outside the table stay arrival-level (no edge obligation), so the
/// check is reverse-only and cannot false-red on extension paths.
/// `corpus` names the review record: the CI gate iterates every
/// frozen corpus, and the active-corpus resolution it used before
/// silently checked external docs against the SELF register — empty
/// for their shas, so the check never ran (Codex review C3 class,
/// caught by the below-floor anchor's first run).
pub fn edge_violations(corpus: Option<&str>, rows: &[Value]) -> Vec<String> {
    let mut bad = Vec::new();
    each_sha(rows, |sha, r| {
        let edges: Vec<Value> = review::edges_in(corpus, sha);
        let allowed = |unit: &str, from: &str, to: &str| {
            edges
                .iter()
                .any(|e| e["to"] == to && e["from"] == from && names_unit(e, unit))
        };
        let tabled = |unit: &str| edges.iter().any(|e| names_unit(e, unit));
        for rel in r["relocations"].as_array().into_iter().flatten() {
            let (from, to) = (rel["from"].as_str(), rel["to"].as_str());
            let (Some(from), Some(to)) = (from, to) else {
                continue;
            };
            for base in rel_bases(rel) {
                if tabled(&base) && !allowed(&base, from, to) {
                    bad.push(format!("{} {base}: {from} -> {to}", &sha[..9]));
                }
            }
        }
    });
    bad
}

/// Register entries not named by their commit's row. Pure const data
/// on the register side, so the CI gate re-runs this without git.
/// `corpus` names the review record (see edge_violations).
pub fn register_misses(corpus: Option<&str>, rows: &[Value]) -> Vec<String> {
    let mut misses = Vec::new();
    each_sha(rows, |sha, r| {
        let keys = row_keys(r);
        for entry in review::units_in(corpus, sha) {
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
    });
    misses
}
