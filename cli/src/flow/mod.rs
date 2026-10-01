//! The flow family's measuring side (plan v2.31 step 4; ADR-008
//! seventh instalment, design booklet docs/reference/analysis-track.md
//! §5): the fourteenth judgment family `flow/1`, the dead code inside a
//! function. The division: this side lowers each unit's body into the
//! four integer tables the core reads — statements, variables, accesses
//! — through one table per language (spec.rs and its tables, the rows
//! and positions they are written in); the control-flow graph, the
//! reachability and liveness walks and every finding are the core's
//! (CE.Flow.*). The lowering and the wire land in step 4 A2 / A3, the
//! three faces in step 5; this step holds the tables and their proof
//! against the grammars.

mod access;
mod access_write;
mod at;
mod build;
mod capture;
mod finish;
mod format;
pub mod lower;
mod params;
pub mod pos;
mod reads;
pub mod rows;
mod scope;
pub mod spec;
mod tree;
mod tree_arms;
mod tree_jumps;
mod tree_loop;
mod tree_try;
pub mod wire;
mod wire_batch;

/// The languages whose flow findings are judged, not advised, as a
/// bitmask at their wire codes: a language enters when its precision
/// doc passes the §5.5 gate (every kind 0 / 1 / 2 pass or vacuous). All
/// ten code languages since step 4 commit G. The set is the package's
/// `flow_judged` column since plan v2.32 step 2 (CE.Lang.Common's
/// language rows, crate::tables), the one owner of which languages'
/// flow findings are verdicts. Pinned by the mask leg of
/// it/eval_flow_precision.rs.
pub fn judged_mask() -> i64 {
    crate::tables::get().languages.mask(|row| row.flow_judged)
}
