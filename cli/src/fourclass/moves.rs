//! The L1 judgment's lowering (plan v2.33 W3): `moves/1`. This side
//! keeps the text and the line diff (diff.rs — the probe path's
//! throat, which never needs a verdict); per changed file pair it
//! sends each side's lines as `[content code, fnv1a(trim), alphanumeric
//! width]` (content codes rank the distinct trimmed texts of the
//! request, so equal code = equal text — the move rule's HashSet<&str>
//! membership, exactly), the
//! 0-based lines the diff changed, and the side's units as
//! `[key code, kind, start, end, stack]` — key codes rank the key texts
//! (so code order is text order) and `keys` maps each code to its
//! fnv1a64. The core (CE.FourClass.Moves) decides which changed lines
//! moved, attributes them, names the units relocated intact, the
//! one-sided declarations, the duplicated top-level spans and the
//! leftover runs; this side maps its integers back onto unit keys and
//! checks every index against what it sent — the reply is an answer,
//! not an authority.

use super::batch::{PairInput, Side};
use super::decls::Decl;
use super::diff;
use super::model::{ChangedLines, Classification, FourClass, MovedLine, alnum_width, line_hashes};
use super::units::{self, Unit};
use crate::corelink::judged::{json_len, line_len, line_max};
use crate::dedup::tokens::fnv1a;
use serde::Deserialize;
use serde_json::{Value, json};

pub const CAP: &str = "moves/1";
pub const KIND: &str = "moves";

/// One pair as this side measured it: both sides' units and the line
/// diff (0-based changed indices).
struct Measured {
    units: [Vec<Unit>; 2],
    changed: [Vec<usize>; 2],
    degraded: bool,
}

/// What reading the reply needs: the measurements and the key codes.
pub struct Request {
    measured: Vec<Measured>,
    codes: Vec<String>,
}

/// The core's answer for every pair, mapped back onto keys.
pub struct Judged {
    pub pairs: Vec<Classification>,
    /// Per pair, the leftover runs of the before and the after side.
    pub runs: Vec<(Side, Side)>,
    /// Per pair, the after side's duplicated top-level spans.
    pub dup_spans: Vec<Vec<[u64; 3]>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Answer {
    counts: [usize; 4],
    moved: Vec<(usize, u8, i64)>,
    relocated: Vec<usize>,
    decl_rem: Vec<usize>,
    decl_add: Vec<usize>,
    dup_spans: Vec<[u64; 3]>,
    runs_rem: Side,
    runs_add: Side,
}

fn measure(p: &PairInput) -> Measured {
    let a: Vec<&str> = p.before.lines().collect();
    let b: Vec<&str> = p.after.lines().collect();
    let d = diff::diff(&line_hashes(&a), &line_hashes(&b));
    Measured {
        units: [p.before, p.after].map(|t| units::segments(t, p.lang)),
        changed: [d.removed, d.added],
        degraded: d.degraded,
    }
}

/// May a key carry stacking identity? Anonymous units have none, and
/// `impl` units are containers (attack review F7).
fn stackable(key: &str) -> bool {
    !key.starts_with("(anonymous)") && !key.starts_with("impl ")
}

/// The `moves.request` bodies over the changeset — consecutive pairs
/// packed under the core's line and unit ceilings (`limits.moves`) and
/// under the protocol's line (`limits.caps.line_bytes`, measured the way
/// corelink::judged::fits_line measures it: every body repeats the whole
/// `keys` table, and the byte line is reached at about a quarter of the
/// row cap — plan v2.33 step 11b item 2), so only a single pair over a
/// ceiling meets a named answer — with what reading the replies needs.
pub fn request(inputs: &[PairInput]) -> (Request, Vec<Value>) {
    let measured: Vec<Measured> = inputs.iter().map(measure).collect();
    let mut codes: Vec<String> = measured
        .iter()
        .flat_map(|m| m.units.iter().flatten().map(|u| u.key.clone()))
        .collect();
    codes.sort_unstable();
    codes.dedup();
    let code = |k: &str| codes.binary_search_by(|c| c.as_str().cmp(k)).unwrap_or(0);
    let mut contents: Vec<&str> = inputs
        .iter()
        .flat_map(|p| p.before.lines().chain(p.after.lines()).map(str::trim))
        .collect();
    contents.sort_unstable();
    contents.dedup();
    let content = |t: &str| contents.binary_search(&t).unwrap_or(0);
    let side = |text: &str, changed: &[usize], us: &[Unit]| {
        let lines: Vec<[u64; 3]> = text
            .lines()
            .map(|l| {
                let t = l.trim();
                [
                    content(t) as u64,
                    fnv1a(t.as_bytes()),
                    alnum_width(l) as u64,
                ]
            })
            .collect();
        let units: Vec<Value> = us
            .iter()
            .map(|u| {
                json!([
                    code(&u.key),
                    u.kind,
                    u.start_line,
                    u.end_line,
                    u8::from(stackable(&u.key))
                ])
            })
            .collect();
        json!({"lines": lines, "changed": changed, "units": units})
    };
    let keys: Vec<u64> = codes.iter().map(|k| fnv1a(k.as_bytes())).collect();
    let pairs: Vec<Value> = inputs
        .iter()
        .zip(&measured)
        .map(|(p, m)| {
            json!({"before": side(p.before, &m.changed[0], &m.units[0]),
                   "after": side(p.after, &m.changed[1], &m.units[1])})
        })
        .collect();
    let bodies = packed(inputs, &measured, &keys, pairs);
    (Request { measured, codes }, bodies)
}

/// The pairs cut into bodies where `packing` puts the starts, each body
/// repeating the whole `keys` table (counted once per body in the room
/// the line leaves).
fn packed(
    inputs: &[PairInput],
    measured: &[Measured],
    keys: &[u64],
    pairs: Vec<Value>,
) -> Vec<Value> {
    let sizes: Vec<[usize; 3]> = inputs
        .iter()
        .zip(measured)
        .zip(&pairs)
        .map(|((p, m), v)| {
            let n_lines = p.before.lines().count() + p.after.lines().count();
            [
                n_lines,
                m.units[0].len() + m.units[1].len(),
                json_len(v) + 1,
            ]
        })
        .collect();
    let limits = &crate::tables::get().limits.moves;
    let empty = line_len(KIND, &json!({"keys": keys, "pairs": []}));
    let room = line_max().saturating_sub(empty);
    let starts = packing(&sizes, [limits.line_cap, limits.unit_cap, room]);
    let mut pairs = pairs.into_iter();
    starts
        .iter()
        .enumerate()
        .map(|(c, &from)| {
            let to = starts.get(c + 1).copied().unwrap_or(inputs.len());
            let chunk: Vec<Value> = pairs.by_ref().take(to - from).collect();
            json!({"keys": keys, "pairs": chunk})
        })
        .collect()
}

/// Where the bodies start, over each pair's [lines, units, bytes] (its
/// serialized bytes plus the separator that joins it to the array): a
/// body is cut when the next pair would pass EITHER row cap or the room
/// the line leaves beside the body's `keys` table; a pair that alone
/// passes one still gets a body of its own — over a row cap the core
/// answers it degraded by name, over the line the batch names it before
/// sending (batch.rs).
pub(crate) fn packing(sizes: &[[usize; 3]], caps: [usize; 3]) -> Vec<usize> {
    let (mut starts, mut run) = (Vec::new(), [0usize; 3]);
    for (i, size) in sizes.iter().enumerate() {
        if i == 0 || (0..3).any(|k| run[k] + size[k] > caps[k]) {
            starts.push(i);
            run = [0; 3];
        }
        (0..3).for_each(|k| run[k] += size[k]);
    }
    starts
}

/// The replies read against the request, in body order: a degraded
/// reply is its named reason, every index is checked against what was
/// sent.
pub fn read(req: Request, replies: &[Value]) -> Result<Judged, String> {
    let mut answers: Vec<Answer> = Vec::new();
    for reply in replies {
        crate::corelink::judged::degraded(reply)?;
        answers.extend(crate::corelink::judged::table::<Vec<Answer>>(
            reply, "pairs",
        )?);
    }
    if answers.len() != req.measured.len() {
        return Err("moves: one answer per pair".into());
    }
    let mut out = Judged {
        pairs: Vec::new(),
        runs: Vec::new(),
        dup_spans: Vec::new(),
    };
    for (i, (m, a)) in req.measured.into_iter().zip(answers).enumerate() {
        let c = classification(m, &a, &req.codes).map_err(|e| format!("moves pair {i}: {e}"))?;
        out.pairs.push(c);
        out.runs.push((a.runs_rem, a.runs_add));
        out.dup_spans.push(a.dup_spans);
    }
    Ok(out)
}

fn classification(m: Measured, a: &Answer, codes: &[String]) -> Result<Classification, String> {
    let [novel, moved_in, deleted, moved_out] = a.counts;
    if novel + moved_in != m.changed[1].len() || deleted + moved_out != m.changed[0].len() {
        return Err("counts do not cover the changed lines".into());
    }
    let unit_at = |side: usize, idx: usize| m.units[side].get(idx).ok_or("unit index out of range");
    let mut moved = Vec::with_capacity(a.moved.len());
    for &(line, removed, unit) in &a.moved {
        let side = usize::from(removed == 0);
        let unit = match usize::try_from(unit) {
            Ok(u) => Some(unit_at(side, u)?.key.clone()),
            Err(_) => None,
        };
        moved.push(MovedLine {
            line,
            removed: removed == 1,
            unit,
        });
    }
    let relocated_units = a
        .relocated
        .iter()
        .map(|&k| {
            codes
                .get(k)
                .cloned()
                .ok_or("relocated key outside the key table")
        })
        .collect::<Result<_, _>>()?;
    let decl = |side: usize, idxs: &[usize]| -> Result<Vec<Decl>, &'static str> {
        idxs.iter()
            .map(|&i| {
                unit_at(side, i).map(|u| Decl {
                    key: u.key.clone(),
                    kind: u.kind,
                    start: u.start_line,
                    end: u.end_line,
                })
            })
            .collect()
    };
    let decls = (decl(0, &a.decl_rem)?, decl(1, &a.decl_add)?);
    let [removed, added] = m.changed.map(|c| c.into_iter().map(|i| i + 1).collect());
    Ok(Classification {
        counts: FourClass {
            added_novel: novel,
            added_moved: moved_in,
            removed_deleted: deleted,
            removed_moved: moved_out,
        },
        moved,
        relocated_units,
        decls,
        changed: ChangedLines { removed, added },
        degraded: m.degraded,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/fourclass/moves.rs"]
mod tests;
