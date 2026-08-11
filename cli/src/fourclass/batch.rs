//! L2 batch classification: run L1 per pair unchanged, ship only the
//! leftovers (significant lines L1 called novel/deleted) to ce-core
//! as `[line, fnv1a(trim)]`, and apply the returned monotone delta.
//! L1 is the IR producer, not a modified engine — single-pair batches
//! with no link are bitwise L1, which is what keeps the saturated
//! 200-sample scores structural rather than re-proven.
//!
//! Every fallback (no link, missing capability, link error, degraded
//! core) returns the pure-L1 result with a visible reason (A9f):
//! plan R8's "退回 L1 而非退回无" is a code path, not a promise.

use super::{Classification, MovedLine, classify, significant, units};
use crate::corelink::Link;
use crate::dedup::tokens::fnv1a;
use crate::scan::lang::Lang;
use serde_json::{Value, json};

pub struct PairInput<'a> {
    pub before: &'a str,
    pub after: &'a str,
    pub lang: Lang,
}

/// One accepted cross-pair correspondence, unit-attributed on both
/// ends (judgment — which lines correspond — stayed in Haskell;
/// symbol lookup stays here, where ADR-002 puts symbols).
#[derive(Debug)]
pub struct Relocation {
    pub from_pair: usize,
    pub from_unit: Option<String>,
    pub to_pair: usize,
    pub to_unit: Option<String>,
    pub lines: usize,
}

pub struct BatchClassification {
    pub pairs: Vec<Classification>,
    pub relocations: Vec<Relocation>,
    /// None = the cross-file pass ran (or nothing needed asking).
    pub degraded: Option<String>,
}

pub fn classify_batch(inputs: &[PairInput], link: Option<&mut Link>) -> BatchClassification {
    let mut pairs: Vec<Classification> = inputs
        .iter()
        .map(|p| classify(p.before, p.after, p.lang))
        .collect();
    let done = |pairs, degraded| BatchClassification {
        pairs,
        relocations: Vec::new(),
        degraded,
    };
    let Some(link) = link else {
        return done(pairs, Some("no_link".into()));
    };
    if !link.has("fourclass/1") {
        return done(pairs, Some("no_capability".into()));
    }
    let sent = leftovers(inputs, &pairs);
    if sent
        .iter()
        .all(|(rem, add)| rem.is_empty() && add.is_empty())
    {
        return done(pairs, None);
    }
    match link.request("fourclass", request_body(&sent)) {
        Err(e) => done(pairs, Some(e)),
        Ok(reply) => match merge(&reply, inputs, &sent, &mut pairs) {
            Err(e) => done(pairs, Some(e)),
            Ok(relocations) => BatchClassification {
                pairs,
                relocations,
                degraded: reply["reason"].as_str().map(String::from),
            },
        },
    }
}

/// One contiguous run of significant leftover lines. Run structure is
/// ALIGNMENT data and therefore produced here, by the aligner: two
/// leftovers are adjacent iff every line between them is also changed
/// and none of those in-between changed lines is significant (blanks
/// and bare punctuation bridge a run — git's moved blocks span them
/// too; a significant in-between line, moved or unchanged, breaks it).
pub type Run = Vec<(usize, u64)>;
pub type Side = Vec<Run>;

/// Per pair: the significant changed lines L1 left novel/deleted,
/// grouped into runs, as (1-based line, fnv1a of the trimmed
/// content). Public so eval and contract tests can assert the exact
/// request content.
pub fn leftovers(inputs: &[PairInput], pairs: &[Classification]) -> Vec<(Side, Side)> {
    inputs
        .iter()
        .zip(pairs)
        .map(|(input, c)| {
            let moved = |removed: bool| -> Vec<usize> {
                c.moved
                    .iter()
                    .filter(|m| m.removed == removed)
                    .map(|m| m.line)
                    .collect()
            };
            (
                side_runs(input.before, &c.changed.removed, &moved(true)),
                side_runs(input.after, &c.changed.added, &moved(false)),
            )
        })
        .collect()
}

fn side_runs(text: &str, changed: &[usize], moved: &[usize]) -> Side {
    let lines: Vec<&str> = text.lines().collect();
    let mut runs: Side = Vec::new();
    let mut open = false;
    let mut prev = 0usize;
    for &l in changed {
        if l != prev + 1 {
            open = false; // an unchanged gap breaks the run
        }
        prev = l;
        let t = lines[l - 1];
        if !significant(t) {
            continue; // blank/punctuation changed line bridges
        }
        if moved.contains(&l) {
            open = false; // a within-moved line breaks the run
            continue;
        }
        let entry = (l, fnv1a(t.trim().as_bytes()));
        match runs.last_mut() {
            Some(run) if open => run.push(entry),
            _ => runs.push(vec![entry]),
        }
        open = true;
    }
    runs
}

fn request_body(sent: &[(Side, Side)]) -> Value {
    let pairs: Vec<Value> = sent
        .iter()
        .enumerate()
        .filter(|(_, (rem, add))| !rem.is_empty() || !add.is_empty())
        .map(|(i, (rem, add))| json!({"i": i, "rem": rem, "add": add}))
        .collect();
    json!({"pairs": pairs})
}

/// Apply the delta. A returned line MUST currently sit in the
/// novel/deleted class (i.e. be among the sent leftovers) — a
/// violation is a protocol bug and fails loudly, never miscounts.
fn merge(
    reply: &Value,
    inputs: &[PairInput],
    sent: &[(Side, Side)],
    pairs: &mut [Classification],
) -> Result<Vec<Relocation>, String> {
    for entry in reply["moved"].as_array().ok_or("reply: moved missing")? {
        let i = entry[0].as_u64().ok_or("moved: pair index")? as usize;
        let (rem_sent, add_sent) = sent.get(i).ok_or("moved: pair out of range")?;
        let (input, c) = (&inputs[i], &mut pairs[i]);
        let before_units = units::segments(input.before, input.lang);
        let after_units = units::segments(input.after, input.lang);
        let pair = |e: String| format!("{e} (pair {i})");
        apply_side(&entry[1], rem_sent, &before_units, true, c).map_err(pair)?;
        apply_side(&entry[2], add_sent, &after_units, false, c).map_err(pair)?;
    }
    relocations_of(reply, inputs)
}

/// One side of a pair's delta: reclassify each returned line out of
/// its plain class into moved, with unit attribution.
fn apply_side(
    lines: &Value,
    sent: &Side,
    side_units: &[units::Unit],
    removed: bool,
    c: &mut Classification,
) -> Result<(), String> {
    for l in lines_of(lines)? {
        if !sent.iter().flatten().any(|&(sl, _)| sl == l) {
            return Err(format!("delta line {l} was not a leftover"));
        }
        if removed {
            c.counts.removed_deleted -= 1;
            c.counts.removed_moved += 1;
        } else {
            c.counts.added_novel -= 1;
            c.counts.added_moved += 1;
        }
        c.moved.push(moved_line(l, removed, side_units));
    }
    Ok(())
}

fn moved_line(line: usize, removed: bool, side_units: &[units::Unit]) -> MovedLine {
    MovedLine {
        line,
        removed,
        unit: units::owner(side_units, line).map(|u| u.key.clone()),
    }
}

fn lines_of(v: &Value) -> Result<Vec<usize>, String> {
    v.as_array()
        .ok_or("delta: line array")?
        .iter()
        .map(|l| l.as_u64().map(|n| n as usize).ok_or("delta: line".into()))
        .collect()
}

/// Unit-attribute each block by its head lines and aggregate.
fn relocations_of(reply: &Value, inputs: &[PairInput]) -> Result<Vec<Relocation>, String> {
    let mut out: Vec<Relocation> = Vec::new();
    for b in reply["blocks"].as_array().ok_or("reply: blocks missing")? {
        let from_pair = b[0].as_u64().ok_or("block: from pair")? as usize;
        let to_pair = b[2].as_u64().ok_or("block: to pair")? as usize;
        let from_lines = lines_of(&b[1])?;
        let to_lines = lines_of(&b[3])?;
        let owner = |input: &PairInput, text_is_before: bool, line: usize| {
            let text = if text_is_before {
                input.before
            } else {
                input.after
            };
            units::owner(&units::segments(text, input.lang), line).map(|u| u.key.clone())
        };
        let from_unit = owner(&inputs[from_pair], true, from_lines[0]);
        let to_unit = owner(&inputs[to_pair], false, to_lines[0]);
        match out.iter_mut().find(|r| {
            r.from_pair == from_pair
                && r.to_pair == to_pair
                && r.from_unit == from_unit
                && r.to_unit == to_unit
        }) {
            Some(r) => r.lines += from_lines.len(),
            None => out.push(Relocation {
                from_pair,
                from_unit,
                to_pair,
                to_unit,
                lines: from_lines.len(),
            }),
        }
    }
    Ok(out)
}
