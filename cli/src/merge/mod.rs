//! The merge family's measuring side (plan v2.31 step 7; ADR-008
//! seventh instalment, design booklet docs/reference/analysis-track.md
//! §6): the fifteenth judgment family `merge/1`, how a clone group would
//! merge into one function. The division: this side gathers the groups
//! (`ce dedup`'s T1/T2 families, `ce clone`'s T3 pairs), builds each
//! member's tree in clone/1's encoding with two more columns — each
//! leaf's source-text hash and each node's position class, read off the
//! slot tables of this module — and labels what comes back; the
//! alignment, the holes, the parameters, the feasibility, the kept
//! member and the savings are the core's (CE.Merge.*). Advisory: the
//! document is a report, no gate reads it.
//!
//! The slot tables live here (slot*.rs), not in flow/: they read
//! FlowSpec's statement forms and never restate them, and flow/ minus
//! mod.rs is the lowering the flow precision docs answer for.

pub mod console;
pub mod face;
pub mod groups;
pub mod groups_trim;
pub mod slot;
mod slot_c;
mod slot_flow;
mod slot_hs;
mod slot_java;
mod slot_launch;
mod slot_lua;
mod slot_r;
mod slot_ts;
pub mod wire;
