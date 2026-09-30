//! The flow/1 batches (plan v2.31 step 4 A3; design booklet §5.1 rule
//! 11, the lowering's totality by refusal): the units split greedily
//! into requests under the core's one cap, a unit never straddling
//! two; and the refusal-driven exclusion — the core names the first
//! offending row (`<table> <i>: <reason>`, CE.Flow.Contract / Tree /
//! Shape), this side takes the unit that row belongs to out of the
//! batch, records it with the core's reason and asks again. The tree
//! contract is never restated here: what a legal unit is stays the
//! core's to say.

use super::lower::{Lowered, Unit};
use super::wire::{self, Finding, KIND, ROW_CAP, Sent, TABLES};
use crate::corelink::Link;

/// A whole judgment over files: each finding with its file and its
/// unit's `nth` (scan's extraction order, the lowering's identity);
/// each unit the core refused, the same way, with the core's reason;
/// the units skipped whole as dynamic.
#[derive(Debug, Default)]
pub struct Verdict {
    pub findings: Vec<(usize, usize, Finding)>,
    pub refused: Vec<(usize, usize, String)>,
    pub dynamic_units: u64,
}

/// A unit's rows against the cap: its units row and its three tables.
fn weight(u: &Unit) -> usize {
    TABLES.iter().map(|(_, rows)| rows(u)).sum()
}

/// A unit this side refused before asking: `(file, nth, reason)`.
pub type Unsent = (usize, usize, String);

/// The units in file order split into requests, each at most ROW_CAP
/// rows; a unit heavier than the cap alone is refused by name beside
/// the batches — its rows are never split, since the core reads a unit
/// whole — and the other units plan as ever. A dynamic unit travels
/// like any other: the core counts it.
pub fn plan(files: &[Lowered]) -> (Vec<Sent<'_>>, Vec<Unsent>) {
    let mut out = Vec::new();
    let mut refused = Vec::new();
    let mut batch = Sent {
        files,
        units: Vec::new(),
    };
    let mut load = 0;
    for (f, file) in files.iter().enumerate() {
        for unit in &file.units {
            let w = weight(unit);
            if w > ROW_CAP {
                let reason = format!(
                    "weighs {w} rows against the cap of {ROW_CAP} — a unit's rows never straddle two requests"
                );
                refused.push((f, unit.nth, reason));
                continue;
            }
            if load + w > ROW_CAP && !batch.units.is_empty() {
                out.push(std::mem::replace(
                    &mut batch,
                    Sent {
                        files,
                        units: Vec::new(),
                    },
                ));
                load = 0;
            }
            batch.units.push((unit, f));
            load += w;
        }
    }
    if !batch.units.is_empty() {
        out.push(batch);
    }
    (out, refused)
}

/// Every batch asked; a refused unit taken out and the rest asked
/// again. Each refusal removes one unit, so a batch is asked at most
/// once more than it has units.
pub fn judge(link: &mut Link, files: &[Lowered]) -> Result<Verdict, String> {
    let (batches, refused) = plan(files);
    let mut verdict = Verdict {
        refused,
        ..Verdict::default()
    };
    for mut sent in batches {
        while !sent.units.is_empty() {
            match wire::ask(link, wire::body(&sent)) {
                Ok(reply) => {
                    settle(&reply, &sent, &mut verdict)?;
                    break;
                }
                Err(e) => {
                    let (at, reason) = refused_unit(&e, &sent).ok_or(e)?;
                    let (unit, f) = sent.units.remove(at);
                    verdict.refused.push((f, unit.nth, reason));
                }
            }
        }
    }
    Ok(verdict)
}

/// One judged reply folded in; a degraded reply to a batch priced
/// under the cap is cap-mirror drift and an error (consume refuses it).
fn settle(reply: &serde_json::Value, sent: &Sent, verdict: &mut Verdict) -> Result<(), String> {
    let judged = wire::consume(reply, sent)?;
    for f in judged.findings {
        let (unit, file) = sent.units[f.u];
        verdict.findings.push((file, unit.nth, f));
    }
    verdict.dynamic_units += judged.counts["dynamicUnits"];
    Ok(())
}

/// A contract refusal's table, row and reason: `<table> <i>: <reason>`,
/// the core's one refusal form for the four tables.
pub(super) fn refusal(message: &str) -> Option<(&str, usize, &str)> {
    let (table, rest) = message.split_once(' ')?;
    let (row, reason) = rest.split_once(": ")?;
    Some((table, row.parse().ok()?, reason))
}

/// The request index of the unit a refused row belongs to, and the
/// reason; None for any other error, which is passed on as it came.
fn refused_unit(error: &str, sent: &Sent) -> Option<(usize, String)> {
    let message = error.strip_prefix(&format!("core refused {KIND}.request: contract: "))?;
    let (table, row, reason) = refusal(message)?;
    let (_, rows) = TABLES.iter().find(|(name, _)| *name == table)?;
    let mut seen = 0;
    let at = sent.units.iter().position(|(u, _)| {
        seen += rows(u);
        row < seen
    })?;
    Some((at, reason.to_owned()))
}
