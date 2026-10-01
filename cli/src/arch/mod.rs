//! The architecture family (plan v2.31 step 9; design booklet
//! docs/reference/analysis-track.md §7): the directory graph's layers
//! and the arcs to cut out of its cycles, the file graph's clusters and
//! the files outside their cluster's directory, the impact of changing
//! a file, and each directory's fan-in, fan-out and instability — as
//! advice. The boundary: this side measures (the measured files, the
//! tree, the references as five integer tables, tables.rs) and labels
//! (face.rs, console.rs); every answer is the core's arch/1 (CE.Arch).
//! No gate reads it, no baseline holds it, `ce check` never sees it.

pub mod console;
pub mod face;
pub mod tables;
pub mod wire;
