//! F4 update-supervision four-classification (plan §4.3): the L1 rung
//! — Myers line diff + tree-sitter function-boundary alignment.
//!
//! Move semantics match the ground-truth convention (labels-v1.json):
//! a changed line is *moved* when its content — leading/trailing
//! whitespace ignored, matching git's allow-indentation-change — also
//! appears on the opposite side of the diff, and the line is
//! *significant* (carries at least one alphanumeric character). Sides
//! are marked independently, so counts may be unbalanced, exactly as
//! git marks them. Blank and pure-punctuation lines belong to no
//! symbol, so they can never "move" — the artifact class the ground
//! truth review corrected out.
//!
//! The function-boundary half attributes every moved line to the unit
//! it left and the unit it joined (the guard's 指回位置), and
//! summarizes intact unit relocations.

pub mod batch;
pub mod diff;
pub mod kinds;
mod model;
pub mod session;
pub mod stacking;
pub mod units;

// the L1 judgment lives in model.rs since the headroom sprint (the
// batch/delta children importing it THROUGH this hub made the
// family a module cycle the graph axis itself billed); these
// re-exports keep every outside path where it always was
pub use model::{
    ChangedLines, Classification, FourClass, MovedLine, alnum_width, classify, significant,
};
