//! The arch document read back (plan v2.32 step 3): the core lays
//! the document out (document/1, CE.Arch.Document), face.rs binds it,
//! and the console and the exit code read these structs off the bound
//! value — the same value the machine faces print.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Layer {
    pub dir: String,
    pub level: i64,
}

#[derive(Deserialize)]
pub struct Reference {
    pub from: String,
    pub to: String,
    pub refs: i64,
}

#[derive(Deserialize)]
pub struct Cut {
    pub from: String,
    pub to: String,
    pub refs: i64,
    pub exact: bool,
    /// The file references folded into this arc; a package target is
    /// its directory with a trailing slash.
    pub files: Vec<Reference>,
}

#[derive(Deserialize)]
pub struct Cluster {
    pub cluster: i64,
    pub majority: String,
    pub files: Vec<String>,
}

#[derive(Deserialize)]
pub struct Misplaced {
    pub path: String,
    pub dir: String,
    pub majority: String,
}

#[derive(Deserialize)]
pub struct Impact {
    pub path: String,
    pub depth: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub dir: String,
    pub fan_in: i64,
    pub fan_out: i64,
    /// Per mille; None when no arc touches the directory.
    pub instability: Option<i64>,
}

/// The bound document, read.
#[derive(Deserialize)]
pub struct Report {
    pub schema: String,
    pub counts: BTreeMap<String, i64>,
    pub layers: Vec<Layer>,
    pub cuts: Vec<Cut>,
    pub clusters: Vec<Cluster>,
    pub misplaced: Vec<Misplaced>,
    pub impact: Vec<Impact>,
    pub metrics: Vec<Metric>,
    pub degraded: Option<String>,
}
