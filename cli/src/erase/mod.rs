//! `ce erase` (M9 batch 3) — the deterministic two-phase eraser.
//! Contract: docs/reference/erase.md; wire face erase/1 (2.16.0; the
//! target closure on the wire since 7.2.0, plan v2.30 step 7b).
//! This module owns the two phase entries: `plan` measures candidate
//! facts from the three source families (same caches, same core,
//! same knobs), the core's erase/1 predicate says which rows are
//! safe and which row STANDS for each target (ADR-008: bytes are
//! measurement, safety and the closure are judgment), and the plan is
//! the complete closed statement of intent; `apply` executes one plan
//! behind its preconditions and then re-plans to PROVE the erased
//! verdicts are gone. Never an LLM rewrite; every hunk reproduces
//! byte-for-byte from the same tree. Types live in model.rs so the
//! children never import upward (axis 6 charged the first draft's
//! `super::` web as the import cycle it was).

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
use std::path::{Path, PathBuf};

/// The whole plan: measure, judge, label. The closure over targets —
/// one row per (path, span), a whole-file deletion owning its path —
/// is the core's answer (`kept`), and the rows go out in the order
/// the plan renders them (path, span, class name) because the
/// closure's tie inside one class breaks to the EARLIEST row: two
/// producers sending the same rows in the same order close the same
/// way, and this producer's order is the sorted one.
pub fn plan(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Plan> {
    let mut g = gather::candidates(root, db, core)?;
    g.candidates.sort_by(|a, b| {
        (&a.path, a.span, CLASS_NAMES[a.class]).cmp(&(&b.path, b.span, CLASS_NAMES[b.class]))
    });
    let rows: Vec<Row> = g
        .candidates
        .iter()
        .zip(wire::judge(core, &g.candidates)?)
        .filter(|(_, v)| v.kept)
        .map(|(c, v)| Row {
            class: CLASS_NAMES[c.class],
            eraseable: v.eraseable,
            reason: REASON_NAMES[v.reason as usize],
            hash: g.hashes.get(&c.path).copied().unwrap_or(0),
            path: c.path.clone(),
            span: c.span,
            provenance: c.provenance.clone(),
            sites: c.sites,
        })
        .collect();
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
