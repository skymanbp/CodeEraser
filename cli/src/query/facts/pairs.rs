//! The judged-pair tables: `dup` off the T1/T2 blocks the snapshot
//! found, `clone` as the whole-unit twins inside those blocks (both
//! sides cover the same number of units, paired in order) and the T3
//! pairs the core judged, `docdup` as the file pairs behind the
//! segment pairs the core judged. Every pair is stored once, lower
//! id first; a file duplicated within itself is a pair of one node.

use super::units::Units;
use super::{Ctx, Sink};
use crate::query::legend::{self, CLONE_KINDS};
use anyhow::Result;

pub fn fill(ctx: &Ctx<'_>, units: &Units, sink: &mut Sink) -> Result<()> {
    if sink.wants("dup") {
        for b in &ctx.blocks.blocks {
            let (Some(f), Some(g)) = (ctx.nodes.of_path(&b.a_file), ctx.nodes.of_path(&b.b_file))
            else {
                continue;
            };
            sink.row("dup", vec![f.min(g), f.max(g), b.tokens as u64]);
        }
    }
    if sink.wants("clone") {
        twins(ctx, units, sink);
        near_misses(ctx, units, sink)?;
    }
    if sink.wants("docdup") {
        let (segs, pairs, _) = crate::docdup::judge::rows_of(ctx.root, ctx.idx, ctx.core)?;
        for (i, j, _) in pairs {
            let (Some(f), Some(g)) = (
                ctx.nodes.of_path(&segs[i].path),
                ctx.nodes.of_path(&segs[j].path),
            ) else {
                continue;
            };
            sink.row("docdup", vec![f.min(g), f.max(g)]);
        }
    }
    Ok(())
}

/// A T1/T2 block covering whole units on both sides pairs them in
/// order; a block covering a different count on each side names no
/// twin (a partial cover is not a unit clone).
fn twins(ctx: &Ctx<'_>, units: &Units, sink: &mut Sink) {
    let kind = legend::sym(CLONE_KINDS[0]);
    for b in &ctx.blocks.blocks {
        let left = units.covered(&b.a_file, b.a_start as i64, b.a_end as i64);
        let right = units.covered(&b.b_file, b.b_start as i64, b.b_end as i64);
        if left.is_empty() || left.len() != right.len() {
            continue;
        }
        for (u, v) in left.into_iter().zip(right) {
            if u != v {
                sink.row("clone", vec![u.min(v), u.max(v), kind]);
            }
        }
    }
}

/// The T3 pairs the core judged clones, on the units' identities.
fn near_misses(ctx: &Ctx<'_>, units: &Units, sink: &mut Sink) -> Result<()> {
    let kind = legend::sym(CLONE_KINDS[1]);
    let judged = crate::dedup::t3::judge_index(ctx.root, ctx.idx, ctx.core)?;
    for (i, j, _) in &judged.clones {
        let seat = |u: &crate::dedup::candidates::Unit| {
            units
                .ids
                .get(&(u.path.clone(), u.key.clone(), u.nth))
                .copied()
        };
        if let (Some(u), Some(v)) = (seat(&judged.units[*i]), seat(&judged.units[*j])) {
            sink.row("clone", vec![u.min(v), u.max(v), kind]);
        }
    }
    Ok(())
}
