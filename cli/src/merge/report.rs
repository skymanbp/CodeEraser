//! The merge document read back (plan v2.32 step 3): the core lays
//! the document out (document/1, CE.Merge.Document), face.rs binds it,
//! and the console and the exit code read these structs off the bound
//! value — the same value the machine faces print.

use super::groups::Unsendable;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A member: its file, its unit (None for a fragment), `lines` its
/// clone-family span (its identity) and `run` the lines of the run it
/// sent, which the core priced (a whole unit's are its span).
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MemberFace {
    pub path: String,
    pub unit: Option<String>,
    pub lines: [usize; 2],
    pub run: [usize; 2],
}

/// One member's text at a parameter: the source from the hole's first
/// root to its last on that member (step-7 ruling 6) — a leaf or
/// relabel hole's one node, a gap hole's whole forest, "" on an empty
/// side.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ValueFace {
    pub member: usize,
    pub text: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ParamFace {
    pub param: usize,
    pub values: Vec<ValueFace>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GroupFace {
    pub group: usize,
    pub family: String,
    pub fragment: bool,
    pub members: Vec<MemberFace>,
    pub params: u64,
    pub kept: usize,
    pub savings: i64,
    pub feasible: bool,
    pub reason: String,
    pub holes: Vec<ParamFace>,
}

/// The bound document, read.
#[derive(Deserialize)]
pub struct Report {
    pub counts: BTreeMap<String, u64>,
    pub unsendable: Unsendable,
    pub groups: Vec<GroupFace>,
    pub degraded: Option<String>,
}
