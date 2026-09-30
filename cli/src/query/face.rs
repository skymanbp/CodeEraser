//! The one document of `ce query`, `ce rules`, the MCP tools and the
//! GUI screen (design booklet §4.6): the program as lexed, every
//! goal with its columns and sorts, every answer labelled through the
//! request's own tables, every proof row named, every error at its
//! `where line:column`, the core's counts, and — when the core did
//! not judge — the named reason. Labelling only: the answers, the
//! proofs and the errors are the core's (wire.rs), and a document
//! with a `degraded` reason carries no verdict this side reached.

use super::columns::{self, GoalHead};
use super::facts::{self, Labels};
use super::program::{Fault, Program, num};
use super::{PRELUDE, legend, wire};
use crate::corelink::Link;
use anyhow::{Result, anyhow};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The two documents (booklet §4.6): a question answered, a rules
/// file judged.
pub const SCHEMA_ID: &str = "ce.query-report/0.1.0";
pub const RULES_SCHEMA_ID: &str = "ce.rules-report/0.1.0";

/// Error names by code: 0 is this side's lexical fault, 1..9 the
/// core's (CE.Query.Cost).
const ERROR_NAMES: [&str; 10] = [
    "lexical",
    "syntax error",
    "unknown predicate",
    "arity mismatch",
    "sort mismatch",
    "unsafe variable",
    "unstratifiable negation",
    "prelude predicate redefined",
    "aggregate shape",
    "anonymous head",
];

/// What a face asks: a rules file (its label and text), a question,
/// whether queries carry proofs.
pub struct Ask {
    pub rules: Option<(String, String)>,
    pub query: Option<String>,
    pub why: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct GoalFace {
    pub goal: usize,
    pub kind: &'static str,
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub sorts: Vec<&'static str>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AnswerFace {
    pub goal: usize,
    pub values: Vec<String>,
    pub raw: Vec<Value>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ProofFace {
    pub goal: usize,
    pub answer: usize,
    pub node: i64,
    pub parent: i64,
    pub rule: i64,
    pub pred: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorFace {
    pub at: String,
    pub token: Option<usize>,
    pub code: i64,
    pub what: String,
}

pub struct Report {
    pub rules_file: Option<String>,
    pub query: Option<String>,
    pub why: bool,
    pub tokens: usize,
    pub clauses: usize,
    pub prelude: usize,
    pub goals: Vec<GoalFace>,
    pub answers: Vec<AnswerFace>,
    pub proof: Vec<ProofFace>,
    pub errors: Vec<ErrorFace>,
    pub counts: BTreeMap<&'static str, u64>,
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

/// A question as the user typed it, in query form: `?-` in front and
/// `.` behind unless written.
pub fn question(text: &str) -> String {
    let t = text.trim();
    let head = if t.starts_with("?-") { "" } else { "?- " };
    let tail = if t.ends_with('.') { "" } else { "." };
    format!("{head}{t}{tail}")
}

/// The whole leg: lexed, assembled, judged, labelled.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, ask: &Ask) -> Result<Report> {
    let query = ask.query.as_deref().map(question);
    let rules = ask.rules.as_ref().map(|(_, text)| text.as_str());
    let mut r = empty(ask, query.clone());
    let program = match Program::lex(PRELUDE, rules, query.as_deref()) {
        Ok(p) => p,
        Err(f) => {
            r.errors.push(lexical(&f));
            return Ok(r);
        }
    };
    r.tokens = program.tokens.len();
    r.clauses = program.clauses;
    r.prelude = program.prelude_clauses;
    r.errors = glob_faults(root, &program);
    if !r.errors.is_empty() {
        return Ok(r);
    }
    let facts = facts::assemble(root, db, core, &program)?;
    let body = wire::body(&program.wire(), &facts.tables, r.prelude, ask.why, false);
    let reply = match Link::open(core).and_then(|(mut link, _)| wire::ask(&mut link, body)) {
        Ok(reply) => reply,
        Err(why) => {
            r.degraded = Some(why);
            return Ok(r);
        }
    };
    let j = wire::consume(&reply, r.tokens).map_err(|e| anyhow!("{e}"))?;
    label(&mut r, &program, &facts.labels, j)?;
    Ok(r)
}

fn empty(ask: &Ask, query: Option<String>) -> Report {
    Report {
        rules_file: ask.rules.as_ref().map(|(label, _)| label.clone()),
        query,
        why: ask.why,
        tokens: 0,
        clauses: 0,
        prelude: 0,
        goals: Vec::new(),
        answers: Vec::new(),
        proof: Vec::new(),
        errors: Vec::new(),
        counts: wire::COUNTS.iter().map(|k| (*k, 0)).collect(),
        degraded: None,
    }
}

fn lexical(f: &Fault) -> ErrorFace {
    ErrorFace {
        at: format!("{} {}:{}", f.src.word(), f.line, f.col),
        token: None,
        code: 0,
        what: f.what.clone(),
    }
}

/// Every glob the program spelled, compiled through the exclude
/// list's dialect before any table is built: a glob it cannot read
/// is a program error at the glob's token.
fn glob_faults(root: &Path, program: &Program) -> Vec<ErrorFace> {
    let mut out = Vec::new();
    for (id, glob) in program.sets.iter().enumerate() {
        let Err(why) =
            crate::scan::globs::compile_inclusions(root, std::slice::from_ref(glob), "glob")
        else {
            continue;
        };
        let at = program
            .tokens
            .iter()
            .position(|t| t.kind == super::lexer::Kind::Set.code() && t.value == id as i128)
            .map_or_else(|| "?".to_string(), |i| program.locate(i));
        out.push(ErrorFace {
            at,
            token: None,
            code: 0,
            what: why.trim_start_matches("ce.toml ").to_string(),
        });
    }
    out
}

/// The core's tables labelled through the program and the facts.
fn label(r: &mut Report, program: &Program, labels: &Labels, j: wire::Judged) -> Result<()> {
    r.counts = j.counts.clone();
    r.degraded = j.degraded.clone();
    r.errors = j
        .errors
        .iter()
        .map(|(at, code)| ErrorFace {
            at: program.locate(*at),
            token: Some(*at),
            code: *code,
            what: ERROR_NAMES[*code as usize].to_string(),
        })
        .collect();
    // an erroring program answers no goal: nothing to label
    if !r.errors.is_empty() || r.degraded.is_some() {
        return Ok(());
    }
    let heads: Vec<GoalHead> = columns::goals(program);
    if heads.len() != j.goals.len() {
        return Err(anyhow!(
            "wire skew: {} goals sent, {} answered",
            heads.len(),
            j.goals.len()
        ));
    }
    r.goals = heads
        .into_iter()
        .zip(&j.goals)
        .enumerate()
        .map(|(goal, (h, (_, sorts)))| GoalFace {
            goal,
            kind: h.kind,
            name: h.name,
            columns: h.columns,
            sorts: sorts.iter().map(|s| legend::sort_name(*s)).collect(),
        })
        .collect();
    let sorts_of = |goal: usize| j.goals[goal].1.clone();
    r.answers = j
        .answers
        .iter()
        .map(|(goal, values)| AnswerFace {
            goal: *goal,
            values: render(labels, &sorts_of(*goal), values),
            raw: values.iter().map(|v| num(*v)).collect(),
        })
        .collect();
    r.proof = j
        .proof
        .iter()
        .map(|p| ProofFace {
            goal: p.goal,
            answer: p.answer,
            node: p.node,
            parent: p.parent,
            rule: p.rule,
            pred: if p.pred < 0 {
                "?-".into()
            } else {
                program.pred_name(p.pred)
            },
            args: render(labels, &pred_sorts(&j, p, &sorts_of(p.goal)), &p.args),
        })
        .collect();
    Ok(())
}

/// A proof node's argument sorts: the schema's for a fact, the
/// core's resolved ones for a program predicate, the goal's own for
/// a query root.
fn pred_sorts(j: &wire::Judged, p: &wire::ProofRow, goal_sorts: &[i64]) -> Vec<i64> {
    if p.pred < 0 {
        return goal_sorts.to_vec();
    }
    if let Some(sorts) = j.preds.get(&p.pred) {
        return sorts.clone();
    }
    u32::try_from(p.pred)
        .ok()
        .and_then(legend::pred_of)
        .map(|s| s.sorts.clone())
        .unwrap_or_default()
}

fn render(labels: &Labels, sorts: &[i64], values: &[i128]) -> Vec<String> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| labels.render(sorts.get(i).copied().unwrap_or(-1), *v))
        .collect()
}

pub fn report_json(r: &Report, rules: bool) -> Value {
    serde_json::json!({
        "schema": if rules { RULES_SCHEMA_ID } else { SCHEMA_ID },
        "program": {
            "rules_file": r.rules_file, "query": r.query, "why": r.why,
            "tokens": r.tokens, "clauses": r.clauses, "prelude": r.prelude,
        },
        "goals": r.goals,
        "answers": r.answers,
        "proof": r.proof,
        "errors": r.errors,
        "counts": r.counts,
        "degraded": r.degraded,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/query/face.rs"]
mod tests;
