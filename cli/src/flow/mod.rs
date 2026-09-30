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
pub mod spec_c;
pub mod spec_go;
pub mod spec_java;
pub mod spec_lua;
pub mod spec_py;
pub mod spec_r;
pub mod spec_rs;
pub mod spec_ts;
mod tree;
mod tree_arms;
mod tree_jumps;
mod tree_loop;
mod tree_try;
pub mod wire;
mod wire_batch;

use crate::scan::lang::Lang;

/// The languages whose flow findings are judged, not advised: a
/// language enters when its precision doc passes the §5.5 gate (every
/// kind 0 / 1 / 2 pass or vacuous), in the exam table's order. Empty
/// since commit E retired the first-generation docs (the lowering they
/// answered by moved); each language comes back with its doc on the
/// fixed lowering. Pinned by the mask leg of it/eval_flow_precision.rs.
const JUDGED: &[Lang] = &[];

pub fn judged_mask() -> i64 {
    JUDGED.iter().fold(0, |m, &l| m | (1 << l as i64))
}
