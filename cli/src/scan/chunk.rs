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
    /// The chunk's events (7.2.0), their row column rebased the same
    /// way; every other column is unit-local and travels as is.
    pub(super) events: Vec<Vec<i64>>,
    pub(super) span: std::ops::Range<usize>,
}

/// Where a chunk starts in each of the four aligned streams, how much
/// of each it holds, and what that weighs against the budget. Cursors
/// that only ever move together, so they travel as one.
#[derive(Default)]
struct Cut {
    span: std::ops::Range<usize>,
    fact0: usize,
    facts: usize,
    arc0: usize,
    arcs: usize,
    ev0: usize,
    evs: usize,
    weight: usize,
}

impl Cut {
    /// The empty cut that begins where this one ends.
    fn after(&self) -> Cut {
        Cut {
            span: self.span.end..self.span.end,
            fact0: self.fact0 + self.facts,
            arc0: self.arc0 + self.arcs,
            ev0: self.ev0 + self.evs,
            ..Cut::default()
        }
    }

    /// Take the file that ends at row `end` into this cut.
    fn take(&mut self, end: usize, load: &Load) {
        self.span.end = end;
        self.facts += load.named;
        self.arcs += load.held;
        self.evs += load.carried;
        self.weight += load.total();
    }
}

/// What one file adds to a chunk — the seats `CE.Scan.overCap` counts
/// for it: its rows (1 each, 2 while a class column rides, because
/// that column travels one entry per row), its code-6 rows' naming
/// facts, its arcs and its events, one seat each.
struct Load {
    rows: usize,
    named: usize,
    held: usize,
    carried: usize,
}

impl Load {
    /// The load of the file running from the cut's end to `end`, read
    /// from where the cut's arc and event cursors stand.
    fn of(r: &ScanRequest<'_>, c: &Cut, end: usize, per_row: usize) -> Load {
        let row = c.span.end;
        Load {
            rows: (end - row) * per_row,
            named: r.rows[row..end].iter().filter(|x| x[0] == 6).count(),
            held: r.calls[c.arc0 + c.arcs..]
                .iter()
                .take_while(|a| (a[0] as usize) < end)
                .count(),
            carried: r.events[c.ev0 + c.evs..]
                .iter()
                .take_while(|e| (e[0] as usize) < end)
                .count(),
        }
    }

    fn total(&self) -> usize {
        self.rows + self.named + self.held + self.carried
    }
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
        events: r.events[c.ev0..c.ev0 + c.evs]
            .iter()
            .map(|e| {
                let mut e = e.clone();
                e[0] -= base as i64;
                e
            })
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
/// silently drop the arcs that cross the cut. The events (7.2.0) are
/// keyed by row the same way and priced one each, so a file travels
/// with every event of every unit it holds.
pub(super) fn plan<'a>(r: &ScanRequest<'a>, budget: usize) -> Result<Vec<Chunk<'a>>> {
    let mut out = Vec::new();
    let per_row = 1 + usize::from(r.row_classes.is_some());
    let mut cut = Cut::default();
    for &block in r.blocks {
        let end = cut.span.end + block;
        let load = Load::of(r, &cut, end, per_row);
        ensure!(
            load.total() <= budget,
            "one file weighs {} rows, arcs and events against a chunk budget of {budget} — a file's rows must not straddle a chunk (scan/wire.rs vs Scan/Cost.hs)",
            load.total()
        );
        if cut.weight + load.total() > budget && cut.weight > 0 {
            let next = cut.after();
            out.push(cut_out(r, cut));
            cut = next;
        }
        cut.take(end, &load);
    }
    out.push(cut_out(r, cut));
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/unit/scan/chunk.rs"]
mod tests;
