//! The flow/1 leg (plan v2.31 step 4 A3; design booklet
//! docs/reference/analysis-track.md §3 and §5): every lowered unit's
//! four integer tables on the wire, and the reply consumed strictly
//! onto typed findings. Nothing here judges: the control-flow graph,
//! the reachability and liveness walks and every finding are the
//! core's (CE.Flow.*), and so is every refusal — the batches and the
//! refusal-driven exclusion (wire_batch.rs) only take a unit the core
//! named out and ask again. A degraded reply to a batch priced under
//! the cap is cap-mirror drift and an error, and a reply whose counts or
//! rows disagree with what was sent is wire skew, never a healthy answer
//! (A9f).

use super::lower::{Lowered, Unit};
use crate::corelink::{Link, judged};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub use super::wire_batch::{Verdict, judge, plan};

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "flow/1";
pub const KIND: &str = "flow";
const SINCE: &str = "7.4.0";

/// One request's units and where they came from: a unit's request
/// index `u` is its place in `units`, the `usize` beside it the index
/// of its file in `files`.
pub struct Sent<'a> {
    pub files: &'a [Lowered],
    pub units: Vec<(&'a Unit, usize)>,
}

/// The counts object's keys, in the wire's order: the four tables sent,
/// then the findings and the units skipped as dynamic.
pub const COUNTS: [&str; 6] = ["units", "stmts", "vars", "uses", "findings", "dynamicUnits"];

/// One finding row, `[u, kind, seq, v, seqEnd]`: `u` the request's
/// unit index, kind 0 unreachable run (v −1) / 1 dead store / 2 unused
/// local / 3 unused parameter (seq −1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub u: usize,
    pub kind: u8,
    pub seq: i64,
    pub v: i64,
    pub seq_end: i64,
}

/// The reply, typed.
#[derive(Debug, Default)]
pub struct Judged {
    pub findings: Vec<Finding>,
    pub counts: BTreeMap<&'static str, u64>,
}

/// The request body: the four tables, a unit's rows led by its request
/// index, the tables in unit order and each unit's rows in the order
/// the lowering numbered them.
pub fn body(sent: &Sent) -> Value {
    let mut tables: [Vec<Vec<i64>>; 4] = Default::default();
    for (u, (unit, file)) in sent.units.iter().enumerate() {
        let u = u as i64;
        let lang = sent.files[*file].lang as i64;
        tables[0].push(vec![u, lang, i64::from(unit.params)]);
        tables[1].extend(unit.stmts.iter().map(|r| led(u, r)));
        tables[2].extend(unit.vars.iter().map(|r| led(u, r)));
        tables[3].extend(unit.uses.iter().map(|r| led(u, r)));
    }
    let [units, stmts, vars, uses] = tables;
    json!({ "units": units, "stmts": stmts, "vars": vars, "uses": uses })
}

fn led(u: i64, row: &[i64]) -> Vec<i64> {
    std::iter::once(u).chain(row.iter().copied()).collect()
}

/// The rows one unit sends to one table.
pub(super) type RowsOf = fn(&Unit) -> usize;

/// The four tables by the name a refusal calls them (CE.Flow.Contract,
/// Tree, Shape: `<table> <i>: <reason>`), in COUNTS' order, and the rows
/// one unit sends to each.
pub(super) const TABLES: [(&str, RowsOf); 4] = [
    ("unit", |_| 1),
    ("stmt", |u| u.stmts.len()),
    ("var", |u| u.vars.len()),
    ("use", |u| u.uses.len()),
];

/// The rows each of the four tables carries for `sent`.
pub fn sizes(sent: &Sent) -> [usize; 4] {
    TABLES.map(|(_, rows)| sent.units.iter().map(|(u, _)| rows(u)).sum())
}

/// One request over a link past its handshake, behind the capability
/// gate (a pre-7.4.0 core is healthy and answers nothing here).
pub fn ask(link: &mut Link, body: Value) -> Result<Value, String> {
    judged::ask(link, CAP, SINCE, KIND, body)
}

/// The reply consumed against what was sent: a degraded reply to a
/// batch priced under the cap is cap-mirror drift and an error; the
/// four table counts must be the rows sent, the findings count the
/// rows, the dynamic count the dynamic units sent, the rows strictly
/// ascending and each one naming a statement and a variable its kind
/// allows.
pub fn consume(reply: &Value, sent: &Sent) -> Result<Judged, String> {
    crate::lockstep::refuse_degraded(reply, "flow/wire.rs vs Flow/Cost.hs")
        .map_err(|e| e.to_string())?;
    let counts = COUNTS
        .iter()
        .map(|k| Ok((*k, judged::count(reply, k)? as u64)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    for (key, n) in COUNTS.iter().zip(sizes(sent)) {
        skew(
            counts[key] == n as u64,
            &format!("counts.{key} is not the {n} rows sent"),
        )?;
    }
    let rows: Vec<[i64; 5]> =
        judged::table(reply, "findings").map_err(|e| format!("wire skew: {e}"))?;
    let dynamic = sent.units.iter().filter(|(u, _)| u.dynamic).count();
    skew(
        counts["findings"] == rows.len() as u64,
        "counts.findings is not the rows",
    )?;
    skew(
        counts["dynamicUnits"] == dynamic as u64,
        &format!("counts.dynamicUnits is not the {dynamic} dynamic units sent"),
    )?;
    skew(
        rows.windows(2).all(|w| w[0] < w[1]),
        "findings not strictly ascending",
    )?;
    let findings = rows
        .iter()
        .map(|r| finding(r, sent))
        .collect::<Result<_, _>>()?;
    Ok(Judged { findings, counts })
}

fn skew(holds: bool, what: &str) -> Result<(), String> {
    if holds {
        Ok(())
    } else {
        Err(format!("wire skew: {what}"))
    }
}

/// One row, held to the unit it names.
fn finding(row: &[i64; 5], sent: &Sent) -> Result<Finding, String> {
    let [u, kind, seq, v, seq_end] = *row;
    let unit = usize::try_from(u).ok().and_then(|i| sent.units.get(i));
    let fits = unit.is_some_and(|(unit, _)| fits(unit, kind, seq, v, seq_end));
    skew(
        fits,
        &format!("finding {row:?} does not fit the unit it names"),
    )?;
    Ok(Finding {
        u: u as usize,
        kind: kind as u8,
        seq,
        v,
        seq_end,
    })
}

/// A kind's row against its unit: an unreachable run spans statements
/// of the unit and names no variable; a dead store names one statement
/// and a variable; an unused local its declaring statement; an unused
/// parameter a parameter at no statement.
fn fits(unit: &Unit, kind: i64, seq: i64, v: i64, seq_end: i64) -> bool {
    let stmts = 0..unit.stmts.len() as i64;
    let var = usize::try_from(v).ok().and_then(|v| unit.vars.get(v));
    let one = seq == seq_end;
    match kind {
        0 => v == -1 && stmts.contains(&seq) && stmts.contains(&seq_end) && seq <= seq_end,
        1 => one && stmts.contains(&seq) && var.is_some(),
        2 => one && stmts.contains(&seq) && var.is_some_and(|r| r[1] == seq),
        3 => one && seq == -1 && var.is_some_and(|r| r[2] & 1 == 1),
        _ => false,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/wire.rs"]
mod tests;
