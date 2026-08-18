//! Materialization + scoring engine for the precision instrument
//! (design §5). The generator and every CI gate score through the
//! SAME functions here (the G1 discipline), so a summary can never
//! drift from its rows without reddening.

use crate::eval_support::TRUTH_KEYWORDS;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The five-way verdict (design D4 + G3 conservation vocabulary).
/// An in-corpus answer matches its truth exactly, or at file level
/// when the resolver made no unit claim (the ladder is file-level by
/// design at 2f; GT records definition units as extra precision —
/// the R5 symbol-binding rung is future work with its own stance).
/// External is an ANSWER: claiming external for an in-corpus truth
/// is wrong, not missed.
pub fn verdict_of(truth: &str, answered: Option<&str>, external: bool) -> &'static str {
    if let Some(a) = answered {
        let file_level = !a.contains('#') && truth.split('#').next() == Some(a);
        return if truth == a || file_level {
            "correct"
        } else {
            "wrong"
        };
    }
    if external {
        return if truth == "external" {
            "external_ok"
        } else {
            "wrong"
        };
    }
    if TRUTH_KEYWORDS.contains(&truth) {
        "unresolved_ok"
    } else {
        "missed"
    }
}

pub const VERDICTS: [&str; 5] = ["correct", "wrong", "missed", "external_ok", "unresolved_ok"];

/// Summary re-derived from the judged rows — generator and gate call
/// this one scorer (G1). The cut table publishes precision per rung
/// ceiling: at ceiling k, answers above k count as refusals.
pub fn rescore(rows: &[Value]) -> Value {
    let mut by_lang: BTreeMap<&str, BTreeMap<&str, u64>> = BTreeMap::new();
    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    let mut in_truth = 0u64;
    for r in rows {
        let v = r["verdict"].as_str().expect("verdict");
        let lang = r["lang"].as_str().expect("lang");
        *totals.entry(v).or_insert(0) += 1;
        *by_lang.entry(lang).or_default().entry(v).or_insert(0) += 1;
        let truth = r["truth"].as_str().expect("truth");
        if !TRUTH_KEYWORDS.contains(&truth) {
            in_truth += 1;
        }
    }
    let n = |m: &BTreeMap<&str, u64>, k: &str| m.get(k).copied().unwrap_or(0);
    let cut: Vec<Value> = (1..=5u8)
        .map(|k| {
            let (mut c, mut w) = (0u64, 0u64);
            for r in rows {
                let within = r["rung"].as_u64().is_some_and(|g| g <= u64::from(k));
                match (
                    r["verdict"].as_str(),
                    within && !r["external"].as_bool().unwrap_or(false),
                ) {
                    (Some("correct"), true) => c += 1,
                    (Some("wrong"), true) if r["answered"].is_string() => w += 1,
                    _ => {}
                }
            }
            json!([k, c, w, ratio(c, w)])
        })
        .collect();
    let (c, w) = (n(&totals, "correct"), n(&totals, "wrong"));
    json!({
        "verdicts": totals,
        "precision": ratio(c, w),
        "recall": if in_truth == 0 { Value::Null } else { json!(c as f64 / in_truth as f64) },
        "in_corpus_truths": in_truth,
        "by_lang": by_lang,
        "cut": cut,
    })
}

fn ratio(c: u64, w: u64) -> Value {
    if c + w == 0 {
        Value::Null
    } else {
        json!(c as f64 / (c + w) as f64)
    }
}
