//! The scan/1 chunk split — moved out of wire.rs when the call table
//! (6.5.0) pushed that file past the 300-line edict; the codec and
//! the split are two jobs, and this is the one that answers a single
//! question: which rows travel together.

use super::wire::ScanRequest;
use anyhow::{Result, ensure};

/// Greedy chunk split whose budget counts EVERY request dimension
/// the core's cap counts (the C15 lesson made structural — the old
/// rows-only `chunks(SCAN_ROW_CAP)` left no room for the grade
/// table, so the first chunk of a cap-sized tree degraded): a row
/// pays 1, or 2 on a classed run because the class column is one
/// entry per row and `CE.Scan.overCap` sums it as its own dimension;
/// a code-6 row pays 1 more (its aligned naming fact travels with
/// it); and the caller reserves the grade and override tables' rows.
/// The class column is priced HERE, per row, not by the caller's
/// reservation: `overrides` is at most a few rows per declared class
/// while `rowClasses` is as long as the chunk, so reserving the one
/// never paid for the other and a large classed tree degraded. The
/// walk that prices code-6 rows is the walk that slices the facts —
/// alignment by construction; the chunk's row SPAN is what the
/// caller slices the class column by.
pub(super) struct Chunk<'a> {
    pub(super) rows: &'a [[u64; 2]],
    pub(super) naming: &'a [[i64; 5]],
    /// The chunk's arcs, rebased onto its own rows array.
    pub(super) calls: Vec<[u64; 2]>,
    pub(super) span: std::ops::Range<usize>,
}

/// Where a chunk starts in each of the three aligned streams, and
/// how much of each it holds. Four cursors that only ever move
/// together, so they travel as one.
struct Cut {
    span: std::ops::Range<usize>,
    fact0: usize,
    facts: usize,
    arc0: usize,
    arcs: usize,
}

fn cut_out<'a>(r: &ScanRequest<'a>, c: Cut) -> Chunk<'a> {
    let base = c.span.start as u64;
    Chunk {
        rows: &r.rows[c.span.clone()],
        naming: &r.naming[c.fact0..c.fact0 + c.facts],
        calls: r.calls[c.arc0..c.arc0 + c.arcs]
            .iter()
            .map(|a| [a[0] - base, a[1] - base])
            .collect(),
        span: c.span,
    }
}

/// The split walks FILES, not rows (6.5.0): a call arc is stated in
/// row indices and would be cut in half by a boundary inside the
/// file that minted it, so the chunk argument the C5 review made —
/// rows grade independently — stops holding the moment a judgment
/// spans two rows. A file whose own block cannot fit the budget is
/// refused by name rather than split, because splitting it would
/// silently drop the arcs that cross the cut.
pub(super) fn plan<'a>(r: &ScanRequest<'a>, budget: usize) -> Result<Vec<Chunk<'a>>> {
    let mut out = Vec::new();
    // 2 while a class column rides: it travels one entry per row
    let per_row = 1 + usize::from(r.row_classes.is_some());
    let (mut start, mut fact0, mut arc0) = (0usize, 0usize, 0usize);
    let (mut weight, mut facts, mut arcs, mut row) = (0usize, 0usize, 0usize, 0usize);
    for &block in r.blocks {
        let end = row + block;
        let named = r.rows[row..end].iter().filter(|x| x[0] == 6).count();
        let held = r.calls[arc0 + arcs..]
            .iter()
            .take_while(|a| (a[0] as usize) < end)
            .count();
        let load = block * per_row + named + held;
        ensure!(
            load <= budget,
            "one file weighs {load} rows and arcs against a chunk budget of {budget} — a file's rows must not straddle a chunk (scan/wire.rs vs Scan/Cost.hs)"
        );
        if weight + load > budget && weight > 0 {
            let cut = Cut {
                span: start..row,
                fact0,
                facts,
                arc0,
                arcs,
            };
            out.push(cut_out(r, cut));
            (start, fact0, arc0) = (row, fact0 + facts, arc0 + arcs);
            (weight, facts, arcs) = (0, 0, 0);
        }
        weight += load;
        facts += named;
        arcs += held;
        row = end;
    }
    let cut = Cut {
        span: start..row,
        fact0,
        facts,
        arc0,
        arcs,
    };
    out.push(cut_out(r, cut));
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/unit/scan/chunk.rs"]
mod tests;
