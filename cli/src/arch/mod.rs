//! The architecture family (plan v2.31 step 9; design booklet
//! docs/reference/analysis-track.md §7): the directory graph's layers
//! and the arcs to cut out of its cycles, the file graph's clusters and
//! the files outside their cluster's directory, the impact of changing
//! a file, and each directory's fan-in, fan-out and instability — as
//! advice. The boundary: this side measures (the measured files, their
//! arcs and line counts, structure::rows) and puts the paths and the
//! directory names the core answers back into the document and the
//! console lines the core lays out (face.rs); the tree, the five tables
//! (CE.Arch.Tables) and every answer are the core's arch/1 (CE.Arch), the
//! document and its lines its document/1 (CE.Arch.Document,
//! CE.Arch.Lines). No gate reads it, no baseline holds it, `ce check`
//! never sees it.

pub mod face;
pub mod wire;
