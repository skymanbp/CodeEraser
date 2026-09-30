//! The query/1 leg (design booklet §3, §4.5): the request — the
//! token stream, the fact tables the program named, the prelude
//! count, the two flags — and the reply consumed onto typed tables.
//! Nothing here judges: the parse, the checks, the strata, the
//! evaluation and the proofs are the core's (ADR-008 seventh
//! instalment); this side re-labels the ids it sent. A degraded
//! reply is a NAMED non-judgment, and a reply whose tables disagree
//! with what was sent is wire skew, never a healthy answer (A9f).

use crate::corelink::{Link, judged};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "query/1";
pub const KIND: &str = "query";
const SINCE: &str = "7.3.0";

/// The counts object's keys, in the wire's order.
pub const COUNTS: [&str; 10] = [
    "rules",
    "queries",
    "asserts",
    "strata",
    "facts",
    "derived",
    "answers",
    "violations",
    "proofNodes",
    "proofTruncated",
];

/// One proof row: the goal and answer it belongs to, the node and
/// its parent (−1 at the root), the clause that derived it (−1 = a
/// fact sent), the predicate (−1 = a query's root), the arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofRow {
    pub goal: usize,
    pub answer: usize,
    pub node: i64,
    pub parent: i64,
    pub rule: i64,
    pub pred: i64,
    pub args: Vec<i128>,
}

/// The reply, typed.
#[derive(Debug, Default)]
pub struct Judged {
    /// Per goal: its kind (0 query / 1 assert) and column sorts.
    pub goals: Vec<(i64, Vec<i64>)>,
    /// Per program predicate: its resolved position sorts.
    pub preds: BTreeMap<i64, Vec<i64>>,
    /// (goal, values) in the core's order.
    pub answers: Vec<(usize, Vec<i128>)>,
    pub proof: Vec<ProofRow>,
    /// (token index, error code).
    pub errors: Vec<(usize, i64)>,
    pub counts: BTreeMap<&'static str, u64>,
    pub schema: Option<Vec<Vec<i64>>>,
    /// The core's named reason it did not judge.
    pub degraded: Option<String>,
}

/// The request body.
pub fn body(
    program: &[Value],
    facts: &BTreeMap<u32, Vec<Vec<u64>>>,
    prelude: usize,
    why: bool,
    schema: bool,
) -> Value {
    let tables: serde_json::Map<String, Value> = facts
        .iter()
        .map(|(code, rows)| (code.to_string(), json!(rows)))
        .collect();
    json!({
        "program": program,
        "facts": tables,
        "prelude": prelude,
        "why": why,
        "schema": schema,
    })
}

/// One request over a link past its handshake, behind the
/// capability gate (a pre-7.3.0 core is healthy and answers nothing).
pub fn ask(link: &mut Link, body: Value) -> Result<Value, String> {
    judged::ask(link, CAP, SINCE, KIND, body)
}

/// A wire number: the unsigned word a hash is, or the signed integer
/// a literal may be.
fn number(v: &Value) -> Result<i128, String> {
    v.as_u64()
        .map(i128::from)
        .or_else(|| v.as_i64().map(i128::from))
        .ok_or_else(|| format!("wire skew: not a number: {v}"))
}

/// The reply consumed; `tokens` is the stream length sent, the bound
/// every error index must respect.
pub fn consume(reply: &Value, tokens: usize) -> Result<Judged, String> {
    let mut j = Judged {
        counts: counts_of(reply)?,
        ..Judged::default()
    };
    if let Err(reason) = judged::degraded(reply) {
        j.degraded = Some(reason);
        return Ok(j);
    }
    j.goals = goals_of(reply)?;
    for row in judged::table::<Vec<Vec<i64>>>(reply, "preds")? {
        match row.as_slice() {
            [code, sorts @ ..] if *code >= 1000 => {
                j.preds.insert(*code, sorts.to_vec());
            }
            _ => return Err("wire skew: a preds row is [code >= 1000, sorts…]".into()),
        }
    }
    for row in judged::table::<Vec<Vec<Value>>>(reply, "answers")? {
        let (goal, values) = split_goal(&row, &j.goals)?;
        if values.len() != j.goals[goal].1.len() {
            return Err("wire skew: an answer's arity is its goal's".into());
        }
        j.answers.push((goal, values));
    }
    j.proof = proof_of(reply, &j.goals)?;
    for row in judged::table::<Vec<Vec<i64>>>(reply, "errors")? {
        match row.as_slice() {
            [at, code] if *at >= 0 && (*at as usize) <= tokens && (1..=9).contains(code) => {
                j.errors.push((*at as usize, *code));
            }
            _ => return Err("wire skew: an error is [token <= sent, code 1..9]".into()),
        }
    }
    if reply.get("schema").is_some() {
        j.schema = Some(judged::table(reply, "schema")?);
    }
    Ok(j)
}

fn counts_of(reply: &Value) -> Result<BTreeMap<&'static str, u64>, String> {
    COUNTS
        .iter()
        .map(|k| Ok((*k, judged::count(reply, k)? as u64)))
        .collect()
}

fn goals_of(reply: &Value) -> Result<Vec<(i64, Vec<i64>)>, String> {
    judged::table::<Vec<Vec<i64>>>(reply, "goals")?
        .into_iter()
        .enumerate()
        .map(|(g, row)| match row.as_slice() {
            [goal, kind, sorts @ ..] if *goal as usize == g && (0..=1).contains(kind) => {
                Ok((*kind, sorts.to_vec()))
            }
            _ => Err("wire skew: a goal row is [goal in order, kind 0|1, sorts…]".into()),
        })
        .collect()
}

/// A row's leading goal index against the goals sent, and the rest
/// as numbers.
fn split_goal(row: &[Value], goals: &[(i64, Vec<i64>)]) -> Result<(usize, Vec<i128>), String> {
    let Some((head, rest)) = row.split_first() else {
        return Err("wire skew: an empty answer row".into());
    };
    let goal = head
        .as_u64()
        .map(|g| g as usize)
        .filter(|g| *g < goals.len())
        .ok_or("wire skew: an answer names a goal that was not sent")?;
    let values = rest.iter().map(number).collect::<Result<Vec<_>, _>>()?;
    Ok((goal, values))
}

fn proof_of(reply: &Value, goals: &[(i64, Vec<i64>)]) -> Result<Vec<ProofRow>, String> {
    judged::table::<Vec<Vec<Value>>>(reply, "proof")?
        .into_iter()
        .map(|row| {
            let (goal, v) = split_goal(&row, goals)?;
            match v.as_slice() {
                [answer, node, parent, rule, pred, args @ ..] if *answer >= 0 => Ok(ProofRow {
                    goal,
                    answer: *answer as usize,
                    node: *node as i64,
                    parent: *parent as i64,
                    rule: *rule as i64,
                    pred: *pred as i64,
                    args: args.to_vec(),
                }),
                _ => Err(
                    "wire skew: a proof row is [goal, answer, node, parent, rule, pred, args…]"
                        .into(),
                ),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/query/wire.rs"]
mod tests;
