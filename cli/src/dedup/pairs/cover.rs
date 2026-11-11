//! The cover/1 leg (v2.35 imports lane, R2): which verified token runs
//! the T1/T2 report keeps is the core's (CE.Dedup.Cover — the coverage
//! rule C_t: per file pair, diverse runs before low-diversity ones,
//! longest first, each kept only with `t` tokens of its own on some
//! side). This side sends the runs it verified — the two files by path
//! rank, the two token offsets, the length, the distinct count — with
//! the run's two filters, and reads every run's class back. The line-
//! containment pass it replaces (`dominant`) is gone; there is no copy of
//! the rule here to fall back on. The requests ride the process's core
//! session (corelink::judged::session_ask) like resolve/1 and bags/1, a
//! degraded or skewed reply refused by name.

use crate::corelink::judged;
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Range;

/// The capability the core must offer — and the rule the result cache
/// keys its slot on (rescache.rs): blocks selected under another rule are
/// never served for this one.
pub const CAP: &str = "cover/1";
const KIND: &str = "cover";

/// One verified run: the two files, the two token offsets, the length
/// and the run's distinct token count.
#[derive(Debug, Clone, Copy)]
pub struct Run<'s> {
    pub a: &'s str,
    pub a0: usize,
    pub b: &'s str,
    pub b0: usize,
    pub len: usize,
    pub distinct: usize,
}

/// A run's class: reported, suppressed (kept, below the diversity
/// floor, counted, never reported) or dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Reported,
    Suppressed,
    Dropped,
}

thread_local! {
    /// cover.requests this thread has sent: the leg that proves the
    /// write-time probe sends none reads it (a thread-local, so no other
    /// thread's dedup moves it).
    static SENT: Cell<usize> = const { Cell::new(0) };
}

/// The cover.requests sent from this thread so far.
pub fn sent_on_this_thread() -> usize {
    SENT.with(Cell::get)
}

/// Every run's class, in the order given. The run set goes to the core
/// split BETWEEN file pairs at the core's row cap (`limits.caps.
/// cover_rows`): pairs are judged independently, so the answers
/// concatenate exactly; a pair is never split, and one pair past the cap
/// alone is refused by name, never dropped.
pub fn classify(runs: &[Run], (t, d): (usize, usize)) -> Result<Vec<Class>> {
    if runs.is_empty() {
        return Ok(Vec::new());
    }
    let rows = wire_rows(runs);
    let cap = crate::tables::get().limits.caps.cover_rows;
    let mut classes = vec![Class::Dropped; runs.len()];
    for part in chunks(&rows, cap)? {
        let slice = &rows[part];
        let body = json!({
            "minTokens": t,
            "minDistinct": d,
            "runs": slice.iter().map(|(_, row)| row).collect::<Vec<_>>(),
        });
        SENT.with(|n| n.set(n.get() + 1));
        let reply = judged::session_ask((CAP, KIND), body).map_err(anyhow::Error::msg)?;
        for ((i, _), class) in slice.iter().zip(consume(&reply, slice.len(), (t, d))?) {
            classes[*i] = class;
        }
    }
    Ok(classes)
}

/// Each run as its wire row with its index, in the contract's order:
/// file codes by path rank, rows ascending on (fileA, fileB, a0, b0).
fn wire_rows(runs: &[Run]) -> Vec<(usize, [u64; 6])> {
    let mut rank: BTreeMap<&str, u64> = runs.iter().flat_map(|r| [(r.a, 0), (r.b, 0)]).collect();
    for (n, code) in rank.values_mut().enumerate() {
        *code = n as u64;
    }
    let mut rows: Vec<(usize, [u64; 6])> = runs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let w = |x: usize| x as u64;
            (
                i,
                [
                    rank[r.a],
                    rank[r.b],
                    w(r.a0),
                    w(r.b0),
                    w(r.len),
                    w(r.distinct),
                ],
            )
        })
        .collect();
    rows.sort_by_key(|(_, row)| [row[0], row[1], row[2], row[3]]);
    rows
}

/// The request split: greedy runs of whole file pairs, each at most `cap`
/// rows; a pair longer than a request holds is refused by name.
fn chunks(rows: &[(usize, [u64; 6])], cap: usize) -> Result<Vec<Range<usize>>> {
    judged::whole_groups(rows, cap, |(_, x), (_, y)| x[..2] == y[..2]).map_err(|(_, n)| {
        anyhow::anyhow!(
            "{CAP}: one file pair has {n} verified runs, over the core's cover_rows cap {cap}; a \
             pair's runs are judged in one request — exclude one of the two files in ce.toml"
        )
    })
}

/// The reply read strictly: three ascending index tables, in range,
/// together every row once; counts that add up; the two filters echoed.
/// Anything else is wire skew, named.
fn consume(reply: &Value, n: usize, (t, d): (usize, usize)) -> Result<Vec<Class>> {
    let skew = |what: &str| format!("{CAP}: wire skew: {what}");
    let echo = (
        judged::count(reply, "rows")?,
        field(reply, "minTokens")?,
        field(reply, "minDistinct")?,
    );
    ensure!(echo == (n, t, d), skew("rows or filters not echoed"));
    let mut classes: Vec<Option<Class>> = vec![None; n];
    for (key, class) in [
        ("reported", Class::Reported),
        ("suppressed", Class::Suppressed),
        ("dropped", Class::Dropped),
    ] {
        let ix: Vec<usize> = judged::table(reply, key)?;
        ensure!(
            ix.windows(2).all(|w| w[0] < w[1]),
            skew(&format!("{key} not ascending"))
        );
        ensure!(
            judged::count(reply, key)? == ix.len(),
            skew(&format!("counts.{key}"))
        );
        for i in ix {
            match classes.get_mut(i) {
                Some(slot @ None) => *slot = Some(class),
                Some(Some(_)) => bail!(skew(&format!("row {i} in two classes"))),
                None => bail!(skew(&format!("{key} index {i} out of range"))),
            }
        }
    }
    classes
        .into_iter()
        .enumerate()
        .map(|(i, c)| c.with_context(|| skew(&format!("row {i} in no class"))))
        .collect()
}

/// A required unsigned field of the reply.
fn field(reply: &Value, key: &str) -> Result<usize> {
    let v = judged::field(reply, key)?;
    v.as_u64()
        .map(|x| x as usize)
        .with_context(|| format!("{CAP}: {key} not a count"))
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/pairs/cover.rs"]
mod tests;
