//! `ce erase` (M9 batch 3) — the deterministic two-phase eraser.
//! Contract: docs/reference/erase.md; wire face erase/1 (2.16.0).
//! This module owns the two phase entries: `plan` measures candidate
//! facts from the three source families (same caches, same core,
//! same knobs), the core's erase/1 predicate says which rows are
//! safe (ADR-008: bytes are measurement, safety is judgment), and
//! the plan is the complete closed statement of intent; `apply`
//! executes one plan behind its preconditions and then re-plans to
//! PROVE the erased verdicts are gone. Never an LLM rewrite; every
//! hunk reproduces byte-for-byte from the same tree. Types live in
//! model.rs so the children never import upward (axis 6 charged the
//! first draft's `super::` web as the import cycle it was).

mod apply;
pub mod gather;
pub mod log;
mod model;
pub mod render;
mod wire;

use anyhow::{Result, bail};
pub use model::{
    CLASS_NAMES, Candidate, Counts, LOG_SCHEMA, Plan, REASON_NAMES, Row, SCHEMA_ID,
    T1T2_NO_WHOLE_UNIT, family_command,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The whole plan: measure, judge, label, close over targets.
pub fn plan(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Plan> {
    let g = gather::candidates(root, db, core)?;
    let mut rows: Vec<Row> = g
        .candidates
        .iter()
        .zip(wire::judge(core, &g.candidates)?)
        .map(|(c, (eraseable, reason))| Row {
            class: CLASS_NAMES[c.class],
            eraseable,
            reason: REASON_NAMES[reason as usize],
            hash: g.hashes.get(&c.path).copied().unwrap_or(0),
            path: c.path.clone(),
            span: c.span,
            provenance: c.provenance.clone(),
            sites: c.sites,
        })
        .collect();
    rows.sort_by(|a, b| (&a.path, a.span, a.class).cmp(&(&b.path, b.span, b.class)));
    let rows = close_targets(rows);
    let counts = Counts {
        candidates: rows.len(),
        eraseable: rows.iter().filter(|r| r.eraseable).count(),
        advisory: rows.iter().filter(|r| !r.eraseable).count(),
        out_of_class: g.out_of_class,
    };
    Ok(Plan { rows, counts })
}

/// The destructive phase: preconditions + writes + audit log
/// (apply.rs), then the convergence re-run HERE — the re-plan calls
/// back into this module, so the executor never imports upward.
/// Returns the applied row count.
pub fn apply_plan(root: &Path, db: Option<PathBuf>, core: &str, p: &Plan) -> Result<usize> {
    let n = apply::execute(root, p)?;
    if n > 0 {
        converge(root, db, core, p)?;
    }
    Ok(n)
}

/// An applied verdict that survives the re-plan fails the command
/// loudly (convergence is part of apply, not a suggestion).
fn converge(root: &Path, db: Option<PathBuf>, core: &str, applied: &Plan) -> Result<()> {
    let after = plan(root, db, core)?;
    let survivors: Vec<String> = after
        .eraseable()
        .filter(|r| {
            applied
                .eraseable()
                .any(|a| a.class == r.class && a.path == r.path)
        })
        .map(|r| format!("{} {}", r.class, r.path))
        .collect();
    if !survivors.is_empty() {
        bail!(
            "erase did not converge — surviving verdicts: {}",
            survivors.join(", ")
        );
    }
    Ok(())
}

/// Close the target set: one row per (path, span), and a whole-file
/// deletion subsumes every span row on the same path — an apply that
/// deleted a file and then tried to splice lines out of it would
/// refuse on the hash it can no longer read. Within one key the
/// ERASEABLE row with the richest licence wins (7.0.0, O51): a dead
/// file that is also a byte-identical twin of a live unit is proposed
/// as `t1_twin`, whose provenance names the survivor it duplicates —
/// before 7.0.0 the dead-file row won by class-name order and the twin
/// class could never reach apply. With no eraseable row the first
/// advisory row (class order) stands, so the categorical refusal
/// (`public_surface`) is the one the reader sees. Rows arrive sorted,
/// so the winner is deterministic.
fn close_targets(rows: Vec<Row>) -> Vec<Row> {
    let mut whole: BTreeMap<String, bool> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.span.is_none()) {
        let slot = whole.entry(r.path.clone()).or_insert(false);
        *slot = *slot || r.eraseable;
    }
    let mut best: BTreeMap<(String, Option<(i64, i64)>), Row> = BTreeMap::new();
    for r in rows {
        // an eraseable whole-file deletion owns the path
        if r.span.is_some() && whole.get(&r.path) == Some(&true) {
            continue;
        }
        let key = (r.path.clone(), r.span);
        let richer =
            |cur: &Row| r.eraseable && (!cur.eraseable || licence(r.class) > licence(cur.class));
        if best.get(&key).is_none_or(richer) {
            best.insert(key, r);
        }
    }
    best.into_values().collect()
}

/// How much a class's eraseable row tells the reader: a twin names
/// the live unit it duplicates, a dead file names only its death.
fn licence(class: &str) -> u8 {
    match class {
        "t1_twin" => 2,
        "dead_file" => 1,
        _ => 0,
    }
}
