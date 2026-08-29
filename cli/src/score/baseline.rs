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
const BASELINE_FILE: &str = "ce-baseline.json";

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
fn fn_entity(path: &str, key: &str, nth: i64) -> u64 {
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
/// `key` is the entity's path as the baseline spells it — the
/// PROJECT-root-relative one (6.4.0, O40; score::provenance::Keys),
/// which is the scan's own `f.path` exactly when the scope is the
/// project.
pub fn continuous_rows(f: &FileMetrics, key: &str) -> Vec<[u64; 3]> {
    let mut rows = vec![[file_entity(key), 0, f.total_lines as u64]];
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
            // these Units exist only to run the with_nth throat over
            // the scan's spans; neither word enters a baseline entity
            // key, so reading them here would be dead work
            vis: 0,
            conv: 0,
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
        rows.push([fn_entity(key, &u.key, nth), 1, u64::from(m.cognitive)]);
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

/// The committed baseline as a verbatim JSON value: None = NO FILE
/// (the core judges in establish mode — a road `ce baseline` refuses
/// to take without the named act, main_score). A file that is
/// present but not a baseline document — `null`, an array, an object
/// without the two tables — is a named error, never None: a missing
/// file and a broken one used to read the same, and the broken one
/// then re-established the floor wholesale with nobody naming it
/// (plan v2.18 step #14, O31). The bytes under "continuous"/
/// "discrete" go on the wire untouched.
pub fn read(root: &Path) -> Result<Option<Value>> {
    let path = path_for(root);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    let doc: Value = serde_json::from_str(&text).with_context(|| path.display().to_string())?;
    anyhow::ensure!(
        doc["continuous"].is_array() && doc["discrete"].is_array(),
        "{}: not a baseline document (an object carrying the continuous and discrete tables)",
        path.display()
    );
    Ok(Some(doc))
}

/// The fence as the non-verdict roads read it (6.4.0, O33). No
/// committed baseline = unfenced: the fence arms with the file and
/// nowhere else, so every road judges with the declared config.
/// Otherwise the digest this config declares beside the one the
/// baseline recorded — absent key = None, the verdict road's own
/// Maybe-equality (a repo at the shipped defaults declares None and
/// recorded None). A present file that is not a baseline document is
/// the O31 error, never a silent state.
pub(crate) enum Fence {
    Unfenced,
    Fenced {
        current: Option<u64>,
        recorded: Option<u64>,
    },
}

impl Fence {
    pub(crate) fn drifted(&self) -> bool {
        matches!(self, Fence::Fenced { current, recorded } if current != recorded)
    }

    /// The scan/1 `knobsFence` value: null unfenced, else the pair.
    pub(crate) fn wire(&self) -> Value {
        match self {
            Fence::Unfenced => Value::Null,
            Fence::Fenced { current, recorded } => json!([current, recorded]),
        }
    }
}

pub(crate) fn fence_status(root: &Path, cfg: &crate::config::Config) -> Result<Fence> {
    Ok(match read(root)? {
        None => Fence::Unfenced,
        Some(doc) => Fence::Fenced {
            current: cfg.knobs_digest(),
            recorded: doc.get("knobsDigest").and_then(Value::as_u64),
        },
    })
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
    // a floor is persisted from the project root ALONE (O30): the CLI
    // refuses a scoped `ce baseline pkg` before measuring, and this
    // is the library's own refusal for every caller after it — a
    // scoped measurement keys its rows below the scope and would
    // overwrite the project's floor with a partial one
    let anchor = crate::root::project_root(root);
    anyhow::ensure!(
        crate::root::same_dir(root, &anchor),
        "baseline: {} is inside project {} — a baseline is a per-project fact, persisted from its root",
        root.display(),
        anchor.display()
    );
    let mut doc = json!({
        "schema": SCHEMA_ID,
        "continuous": new_baseline["continuous"],
        "discrete": new_baseline["discrete"],
        "softLine": new_baseline["softLine"],
        // 2.21.0 (batch-7 slice 5): the zone tier cut points, the
        // hook's core-authored map — exactly the key class this
        // writer's own comment warns about dropping
        "zoneTiers": new_baseline["zoneTiers"],
    });
    // 5.1.0: the rulepack fingerprint these ceilings were established
    // under, written exactly when the core sent one. ABSENT, not
    // null — a repo that declares no class must keep a
    // byte-identical baseline file (K11), and a key holding null is
    // not an absent key.
    if let Some(d) = new_baseline.get("knobsDigest").filter(|v| !v.is_null()) {
        doc["knobsDigest"] = d.clone();
    }
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
#[path = "../../tests/unit/score/baseline.rs"]
mod tests;
