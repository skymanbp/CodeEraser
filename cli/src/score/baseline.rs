//! Baseline identities and the ce-baseline.json file (ADR-006 /
//! design §7.2). Identities are FNV-1a 64 fingerprints so the wire
//! stays index-free where stability matters: continuous rows join
//! current-vs-baseline on (fingerprint, metricCode) across runs, and
//! discrete members are the violation set itself. The baseline file
//! crosses the wire VERBATIM (its "continuous"/"discrete" keys ARE
//! the wire shape); Rust never computes tolerance or membership —
//! that is the core's job (ADR-008 anti-preemption).
//!
//! Known degradation (§7.2, recorded): deleting an earlier same-key
//! sibling shifts nth, so that member id reads as one removal plus
//! one addition.

use crate::fourclass::units::{self, Unit};
use crate::scan::metrics::FileMetrics;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Where the committed baseline lives (betterer convention).
pub const BASELINE_FILE: &str = "ce-baseline.json";

pub const SCHEMA_ID: &str = "ce.baseline/1";

/// FNV-1a 64 over the field bytes, NUL-separated — the §7.2 member
/// identity primitive.
pub fn fnv1a(fields: &[&[u8]]) -> u64 {
    let mut h: u64 = 14695981039346656037;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(1099511628211);
        }
    };
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            eat(&[0]);
        }
        eat(f);
    }
    h
}

/// One side of a pair member: the cached unit identity.
pub type Side = (String, String, i64);

/// §7.2: member id = fnv1a(kind ‖0‖ a_path ‖0‖ a_key ‖0‖ a_nth ‖0‖
/// b_path ‖0‖ b_key ‖0‖ b_nth), sides normalized by (path,key,nth)
/// lex order. Line numbers and block order are deliberately absent:
/// moving a clone must not redden; a NEW clone must.
pub fn member_id(kind: &str, a: &Side, b: &Side) -> u64 {
    let (x, y) = if a <= b { (a, b) } else { (b, a) };
    let (an, bn) = (x.2.to_string(), y.2.to_string());
    fnv1a(&[
        kind.as_bytes(),
        x.0.as_bytes(),
        x.1.as_bytes(),
        an.as_bytes(),
        y.0.as_bytes(),
        y.1.as_bytes(),
        bn.as_bytes(),
    ])
}

/// Continuous entity for a file's line count (metricCode 0).
pub fn file_entity(path: &str) -> u64 {
    fnv1a(&[b"file", path.as_bytes()])
}

/// Continuous entity for a function's cognitive complexity
/// (metricCode 1): (path, key, nth) through the SAME with_nth
/// ordering the unit caches persist — the fn identity a rename or
/// move keeps honest.
pub fn fn_entity(path: &str, key: &str, nth: i64) -> u64 {
    fnv1a(&[
        b"fn",
        path.as_bytes(),
        key.as_bytes(),
        nth.to_string().as_bytes(),
    ])
}

/// Continuous rows [entity, code, value] for one scanned file: its
/// line count plus every function's cognitive complexity, nth
/// assigned by the units::with_nth throat over the scan's own spans.
pub fn continuous_rows(f: &FileMetrics) -> Vec<[u64; 3]> {
    let mut rows = vec![[file_entity(&f.path), 0, f.total_lines as u64]];
    // m.name already carries the Go receiver qualification from the
    // extraction root (functions::name_of), so this composition and
    // the unit-cache keys agree by construction (M5-close review D4)
    let fn_units: Vec<Unit> = f
        .functions
        .iter()
        .map(|m| Unit {
            key: format!("{}/{}", m.name, m.params),
            start_line: m.start_line,
            end_line: m.end_line,
        })
        .collect();
    for (u, nth) in units::with_nth(&fn_units) {
        // recover the metrics row by POINTER identity — same-line
        // nested closures share (start,end), so a span lookup could
        // pair the wrong measurement (the churn unit_id lesson)
        let idx = fn_units
            .iter()
            .position(|x| std::ptr::eq(x, u))
            .expect("with_nth walks the same slice");
        let m = &f.functions[idx];
        rows.push([fn_entity(&f.path, &u.key, nth), 1, u64::from(m.cognitive)]);
    }
    rows
}

/// The committed baseline's path for `root`: the project ANCHOR's
/// copy, never a fresh one beside a subdirectory. A ratchet is a
/// per-project fact, and `ce check cli` reading no baseline made the
/// gate pass by having nothing to compare (the empty-ratchet green).
/// `ce baseline cli` writing one would have been worse: a second
/// floor no gate reads and no eject removes.
pub fn path_for(root: &Path) -> PathBuf {
    crate::root::project_root(root).join(BASELINE_FILE)
}

/// The baseline file as a verbatim JSON value: None = no baseline
/// yet (the core judges in establish mode). The bytes under
/// "continuous"/"discrete" go on the wire untouched.
pub fn read(root: &Path) -> Result<Option<Value>> {
    let path = path_for(root);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    Ok(Some(
        serde_json::from_str(&text).context("ce-baseline.json")?,
    ))
}

/// Write the core's newBaseline back as the committed file, wrapped
/// in the schema envelope (extra keys are ignored by the core's
/// reader, so the envelope never desyncs the wire shape). softLine
/// (2.14.0) is copied EXPLICITLY: this writer rebuilds the document
/// from named keys, so a key it does not name would be silently
/// dropped on every re-establish — the v0.6 map called this the
/// single easiest thing to miss.
/// Returns the path written, so the caller can NAME it: the success
/// line used to print the bare constant, which said nothing about
/// which directory just gained a floor.
pub fn write(root: &Path, new_baseline: &Value) -> Result<PathBuf> {
    let doc = json!({
        "schema": SCHEMA_ID,
        "continuous": new_baseline["continuous"],
        "discrete": new_baseline["discrete"],
        "softLine": new_baseline["softLine"],
    });
    let path = path_for(root);
    // temp + rename, not a truncating write: a `ce baseline` killed
    // mid-write (Ctrl-C, CI timeout) left a torn ce-baseline.json that
    // failed every later `ce check` — and the PreToolUse budget rule
    // reads the same file, where a parse error silently substitutes a
    // different soft line with no trace in the feed.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(&doc)?))
        .with_context(|| tmp.display().to_string())?;
    std::fs::rename(&tmp, &path).with_context(|| path.display().to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The §7.2 counterfactual pair: side order must not matter, and
    /// every field participates (a moved LINE changes nothing here
    /// because lines are not fields).
    #[test]
    fn member_identity_is_order_free_and_field_sensitive() {
        let a: Side = ("a.rs".into(), "work/1".into(), 0);
        let b: Side = ("b.rs".into(), "work/1".into(), 1);
        assert_eq!(member_id("clone", &a, &b), member_id("clone", &b, &a));
        let c: Side = ("b.rs".into(), "work/1".into(), 2);
        assert_ne!(member_id("clone", &a, &b), member_id("clone", &a, &c));
        assert_ne!(member_id("clone", &a, &b), member_id("t3", &a, &b));
    }
}
