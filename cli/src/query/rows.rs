//! The query document request's judged rows (plan v2.32 step 3): the
//! core's tables sent back as `document/1` rows, with the goals as
//! spelled, a value past the JSON integer range as its decimal string
//! (program::num).

use super::columns::GoalHead;
use super::program::num;
use super::wire;
use crate::document::Request;
use anyhow::{Result, bail};
use serde_json::Value;

/// The core's tables sent back, with the goals as spelled.
pub(super) fn judged(req: Request, heads: &[GoalHead], j: &wire::Judged) -> Result<Request> {
    if j.errors.is_empty() && heads.len() != j.goals.len() {
        bail!(
            "wire skew: {} goals sent, {} answered",
            heads.len(),
            j.goals.len()
        );
    }
    let head_rows: Vec<[usize; 3]> = heads
        .iter()
        .enumerate()
        .map(|(g, h)| [g, usize::from(h.name.is_some()), h.columns.len()])
        .collect();
    let goals: Vec<Vec<i64>> = j
        .goals
        .iter()
        .enumerate()
        .map(|(g, (kind, sorts))| [&[g as i64, *kind][..], sorts].concat())
        .collect();
    let preds: Vec<Vec<i64>> = j
        .preds
        .iter()
        .map(|(code, sorts)| [&[*code][..], sorts].concat())
        .collect();
    let errors: Vec<[i64; 2]> = j
        .errors
        .iter()
        .map(|(at, code)| [*at as i64, *code])
        .collect();
    Ok(req
        .rows("heads", head_rows)
        .rows("goals", goals)
        .rows("preds", preds)
        .rows("answers", answer_rows(j))
        .rows("proof", proof_rows(j))
        .rows("errors", errors))
}

/// The answers as [goal, value...].
fn answer_rows(j: &wire::Judged) -> Vec<Vec<Value>> {
    j.answers
        .iter()
        .map(|(g, vs)| valued(&[*g as i64], vs))
        .collect()
}

/// The proof rows as [goal, answer, node, parent, rule, pred, arg...].
fn proof_rows(j: &wire::Judged) -> Vec<Vec<Value>> {
    let row = |p: &wire::ProofRow| {
        let head = [
            p.goal as i64,
            p.answer as i64,
            p.node,
            p.parent,
            p.rule,
            p.pred,
        ];
        valued(&head, &p.args)
    };
    j.proof.iter().map(row).collect()
}

/// A row of integers, then values as numbers (a value past the JSON
/// integer range rides as its decimal string, program::num).
fn valued(head: &[i64], values: &[i128]) -> Vec<Value> {
    let head = head.iter().map(|x| Value::from(*x));
    head.chain(values.iter().map(|v| num(*v))).collect()
}
