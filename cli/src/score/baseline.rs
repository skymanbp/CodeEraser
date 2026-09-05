//! Baseline identities and the ce-baseline.json file (ADR-006 /
//! design §7.2). Identities are FNV-1a 64 fingerprints so the wire
//! stays index-free where stability matters: continuous rows join
//! current-vs-baseline on (fingerprint, metricCode) across runs, and
//! discrete members are the violation set itself. The baseline file
//! crosses the wire VERBATIM (its "continuous"/"discrete" keys ARE
//! the wire shape); Rust never computes tolerance or membership —
//! that is the core's job (ADR-008 anti-preemption).
//!
//! Same-key siblings are told apart by the §7.2 CONTAINER ANCHOR
//! (anchor.rs) since 7.0.0, never by nth: the recorded degradation —
//! deleting an earlier sibling shifted the survivors' nth and read as
//! one removal plus one addition — is retired, and a 6.x baseline is
//! refused by name (`ce.baseline/1`) so the identity change is a
//! visible one-time re-establish, not a silent red.

use super::anchor::{Anchors, Side};
use crate::scan::metrics::FileMetrics;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where the committed baseline lives (betterer convention).
const BASELINE_FILE: &str = "ce-baseline.json";

/// The schema this ce writes and the ONLY one it reads (7.0.0): every
/// 6.x file carries `/1` — identities keyed on nth — and read as-is,
/// every same-key member would register as one removal plus one
/// addition and the ratchet would fail on "added" clones nobody wrote.
/// `read` refuses any other stamped schema by the name the FILE
/// carries, so the family is spelled here once (facts gate).
pub const SCHEMA_ID: &str = "ce.baseline/2";

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

/// §7.2: member id = fnv1a(kind ‖0‖ a_path ‖0‖ a_key ‖0‖ a_anchor ‖0‖
/// b_path ‖0‖ b_key ‖0‖ b_anchor), sides normalized by (path, key,
/// anchor) lex order. Line numbers and block order are deliberately
/// absent: moving a clone must not redden; a NEW clone must. The
/// anchor is the container chain's hash plus the unit's order under it
/// (anchor.rs): a top-level function anchors to one constant, and only
/// a same-container redefinition can ever shift.
pub fn member_id(kind: &str, a: &Side, b: &Side) -> u64 {
    let (x, y) = if a <= b { (a, b) } else { (b, a) };
    fnv1a(&[
        kind.as_bytes(),
        x.0.as_bytes(),
        x.1.as_bytes(),
        x.2.as_bytes(),
        y.0.as_bytes(),
        y.1.as_bytes(),
        y.2.as_bytes(),
    ])
}

/// Continuous entity for a file's line count (metricCode 0).
pub fn file_entity(path: &str) -> u64 {
    fnv1a(&[b"file", path.as_bytes()])
}

/// Continuous entity for a function's cognitive complexity
/// (metricCode 1): (path, key, §7.2 anchor) — the fn identity a
/// rename or move keeps honest, and a deleted sibling leaves alone.
fn fn_entity(path: &str, key: &str, anchor: &str) -> u64 {
    fnv1a(&[b"fn", path.as_bytes(), key.as_bytes(), anchor.as_bytes()])
}

/// Continuous rows [entity, code, value] for one scanned file: its
/// line count plus every function's cognitive complexity, each
/// function anchored through the index's own unit table (anchor.rs).
/// `key` is the entity's path as the baseline spells it — the
/// PROJECT-root-relative one (6.4.0, O40; score::provenance::Keys),
/// which is the scan's own `f.path` exactly when the scope is the
/// project. m.name already carries the Go receiver qualification from
/// the extraction root (functions::name_of), so the key composed here
/// and the unit cache's agree by construction (M5-close review D4) —
/// a function the cache does not know is a named error, never a
/// guessed identity.
pub fn continuous_rows(f: &FileMetrics, key: &str, anchors: &Anchors) -> Result<Vec<[u64; 3]>> {
    let mut rows = vec![[file_entity(key), 0, f.total_lines as u64]];
    // same-span same-key functions (closures sharing a line) in scan order
    let mut seen: BTreeMap<(String, usize, usize), usize> = BTreeMap::new();
    for m in &f.functions {
        let unit = format!("{}/{}", m.name, m.params);
        let nth = seen
            .entry((unit.clone(), m.start_line, m.end_line))
            .or_insert(0);
        let anchor = anchors
            .of(&f.path, &unit, m.start_line, m.end_line, *nth)
            .with_context(|| {
                format!(
                    "{}:{}-{} {unit}: the scanner measured a function the unit cache does not hold",
                    f.path, m.start_line, m.end_line
                )
            })?;
        *nth += 1;
        rows.push([fn_entity(key, &unit, anchor), 1, u64::from(m.cognitive)]);
    }
    Ok(rows)
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
    let Some(doc) = document(root)? else {
        return Ok(None);
    };
    if let Some(stamped) = doc["schema"].as_str().filter(|s| *s != SCHEMA_ID) {
        let path = path_for(root);
        bail!(
            "{}: written under {stamped}, this ce reads {SCHEMA_ID} (7.0.0 keys members on §7.2 container anchors, not nth) — re-establish once: CE_ACCEPT_BASELINE=1 ce baseline .",
            path.display()
        );
    }
    Ok(Some(doc))
}

/// The committed document as an ENVELOPE, whatever schema stamped it:
/// softLine, zoneTiers, knobsDigest and the structure floor mean the
/// same under every schema, so the fence, the hook's budget and the
/// structure advisory read here and keep working across the 7.0.0
/// identity migration — only the ratchet tables (`read`) refuse.
pub fn document(root: &Path) -> Result<Option<Value>> {
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
    Ok(match document(root)? {
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
    // 5.1.0: the knob fingerprint these ceilings were established
    // under (the canonical effective set since O39), written exactly
    // when the core sent one. ABSENT, not null — a repo that judges
    // as the shipped default must keep a byte-identical baseline
    // file (K11), and a key holding null is not an absent key.
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
