//! The query document read back (plan v2.32 step 3): the core lays
//! the document out (document/1, CE.Query.Document), face.rs binds it,
//! and the console and the exit code read these structs off the bound
//! value — the same value the machine faces print.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProgramFace {
    pub rules_file: Option<String>,
    pub query: Option<String>,
    pub why: bool,
    pub tokens: usize,
    pub clauses: usize,
    pub prelude: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GoalFace {
    pub goal: usize,
    pub kind: String,
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub sorts: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AnswerFace {
    pub goal: usize,
    pub values: Vec<String>,
    pub raw: Vec<Value>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProofFace {
    pub goal: usize,
    pub answer: usize,
    pub node: i64,
    pub parent: i64,
    pub rule: i64,
    pub pred: String,
    pub args: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorFace {
    pub at: String,
    pub token: Option<usize>,
    pub code: i64,
    pub what: String,
}

/// The bound document, read.
#[derive(Deserialize)]
pub struct Report {
    pub program: ProgramFace,
    pub goals: Vec<GoalFace>,
    pub answers: Vec<AnswerFace>,
    pub proof: Vec<ProofFace>,
    pub errors: Vec<ErrorFace>,
    pub counts: BTreeMap<String, u64>,
    pub degraded: Option<String>,
}

impl Report {
    pub fn violations(&self) -> u64 {
        self.counts.get("violations").copied().unwrap_or(0)
    }

    /// Whether the document carries a judgment: no lexical or program
    /// error, and a core that answered.
    pub fn judged(&self) -> bool {
        self.errors.is_empty() && self.degraded.is_none()
    }
}
