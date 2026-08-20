//! Applying the core's cross-pair delta (split from batch.rs at the
//! 300-line dogfood wall): reclassify returned lines out of their
//! plain class into moved, and unit-attribute the accepted blocks.
//! Every wire index and line is checked against what THIS side sent
//! — the reply is an answer, not an authority.

use super::super::{Classification, MovedLine, units};
use super::{PairInput, Relocation, Side};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Apply the delta ONTO A COPY. A returned line MUST be one of the
/// sent leftovers AND not yet consumed — the leftover check alone
/// accepted the same line twice, and the second decrement underflowed
/// the class count (usize wrap: ~18e18 "deleted" lines) instead of
/// failing loudly (review 2026-08-19, codex lane). Consuming from a
/// per-side set makes both violations the same named error. The copy
/// is the all-or-nothing seam: in-place mutation leaked a HALF-merged
/// result through the error path as the "pure L1 fallback", breaking
/// batch.rs's header promise (review 2026-08-20 #5).
pub(super) fn merge(
    reply: &Value,
    inputs: &[PairInput],
    sent: &[(Side, Side)],
    pairs: &[Classification],
) -> Result<(Vec<Classification>, Vec<Relocation>), String> {
    let mut merged = pairs.to_vec();
    let mut unconsumed: BTreeMap<usize, [BTreeSet<usize>; 2]> = BTreeMap::new();
    for entry in reply["moved"].as_array().ok_or("reply: moved missing")? {
        let i = entry[0].as_u64().ok_or("moved: pair index")? as usize;
        let (rem_sent, add_sent) = sent.get(i).ok_or("moved: pair out of range")?;
        let sets = unconsumed.entry(i).or_insert_with(|| {
            [rem_sent, add_sent].map(|s| s.iter().flatten().map(|&(l, _, _)| l).collect())
        });
        let (input, c) = (&inputs[i], &mut merged[i]);
        let before_units = units::segments(input.before, input.lang);
        let after_units = units::segments(input.after, input.lang);
        let [rem_left, add_left] = sets;
        let pair = |e: String| format!("{e} (pair {i})");
        apply_side(&entry[1], rem_left, &before_units, true, c).map_err(pair)?;
        apply_side(&entry[2], add_left, &after_units, false, c).map_err(pair)?;
    }
    Ok((merged, relocations_of(reply, inputs)?))
}

/// One side of a pair's delta: reclassify each returned line out of
/// its plain class into moved, with unit attribution. Each line is
/// consumed from the side's leftover set exactly once.
fn apply_side(
    lines: &Value,
    unconsumed: &mut BTreeSet<usize>,
    side_units: &[units::Unit],
    removed: bool,
    c: &mut Classification,
) -> Result<(), String> {
    for l in lines_of(lines)? {
        if !unconsumed.remove(&l) {
            return Err(format!("delta line {l} was not an unconsumed leftover"));
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

/// Unit-attribute each block LINE BY LINE — from/to lines correspond
/// positionally, and one block can span several units (a bulk
/// extraction moves many helpers in one contiguous region), so
/// head-line attribution would name only the first unit and silently
/// unname the rest (the relocation-register gate caught exactly
/// that: 7 of 35 registered units unnamed).
fn relocations_of(reply: &Value, inputs: &[PairInput]) -> Result<Vec<Relocation>, String> {
    let mut out: Vec<Relocation> = Vec::new();
    for b in reply["blocks"].as_array().ok_or("reply: blocks missing")? {
        let from_pair = b[0].as_u64().ok_or("block: from pair")? as usize;
        let to_pair = b[2].as_u64().ok_or("block: to pair")? as usize;
        // .get, like merge(): a wire index as a slice subscript panics
        let from = inputs.get(from_pair).ok_or("block: from out of range")?;
        let to = inputs.get(to_pair).ok_or("block: to out of range")?;
        let from_units = units::segments(from.before, from.lang);
        let to_units = units::segments(to.after, to.lang);
        for (fl, tl) in lines_of(&b[1])?.into_iter().zip(lines_of(&b[3])?) {
            let from_unit = units::owner(&from_units, fl).map(|u| u.key.clone());
            let to_unit = units::owner(&to_units, tl).map(|u| u.key.clone());
            match out.iter_mut().find(|r| {
                r.from_pair == from_pair
                    && r.to_pair == to_pair
                    && r.from_unit == from_unit
                    && r.to_unit == to_unit
            }) {
                Some(r) => r.lines += 1,
                None => out.push(Relocation {
                    from_pair,
                    from_unit,
                    to_pair,
                    to_unit,
                    lines: 1,
                }),
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::PairInput;
    use super::*;
    use crate::fourclass::{ChangedLines, FourClass};
    use crate::scan::lang::Lang;
    use serde_json::json;

    fn one_leftover() -> (
        Vec<PairInput<'static>>,
        Vec<(Side, Side)>,
        Vec<Classification>,
    ) {
        let inputs = vec![PairInput {
            before: "let x = compute();\n",
            after: "",
            lang: Lang::Rust,
        }];
        let sent = vec![(vec![vec![(1usize, 7u64, 10usize)]], Vec::new())];
        let pairs = vec![Classification {
            counts: FourClass {
                removed_deleted: 1,
                ..FourClass::default()
            },
            moved: Vec::new(),
            relocated_units: Vec::new(),
            changed: ChangedLines {
                removed: vec![1],
                added: Vec::new(),
            },
            degraded: false,
        }];
        (inputs, sent, pairs)
    }

    /// The same leftover line listed twice in the delta is a named
    /// error — under the old any()-check it decremented the class
    /// count twice and UNDERFLOWED the usize (panic in debug, ~18e18
    /// "deleted" lines in release). AND the caller's pairs stay the
    /// untouched pure L1: the first (valid) application happened on
    /// merge's copy, so the error path leaks none of it (#5).
    #[test]
    fn a_double_listed_delta_line_is_refused_not_underflowed() {
        let (inputs, sent, pairs) = one_leftover();
        let reply = json!({"moved": [[0, [1, 1], []]], "blocks": []});
        let err = merge(&reply, &inputs, &sent, &pairs).expect_err("double apply");
        assert!(err.contains("unconsumed"), "{err}");
        assert_eq!(pairs[0].counts.removed_deleted, 1, "pure L1 intact");
        assert!(pairs[0].moved.is_empty(), "no half-merged moved lines");
    }

    /// The single-application baseline that keeps the refusal above
    /// from passing vacuously: one line, applied once, reclassifies —
    /// on the RETURNED copy, never the input.
    #[test]
    fn a_single_delta_line_reclassifies_once() {
        let (inputs, sent, pairs) = one_leftover();
        let reply = json!({"moved": [[0, [1], []]], "blocks": []});
        let (merged, _) = merge(&reply, &inputs, &sent, &pairs).expect("single apply");
        assert_eq!(merged[0].counts.removed_deleted, 0);
        assert_eq!(merged[0].counts.removed_moved, 1);
        assert_eq!(pairs[0].counts.removed_deleted, 1, "input untouched");
    }
}
