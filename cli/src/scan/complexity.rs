//! The complexity road's measuring half (ADR-008 fourth instalment,
//! plan v2.23; widened by plan v2.30 step 7b ③): the call arcs and
//! each unit's structural events projected onto row indices on the
//! way out, and the values the core judged with written back on the
//! way in.
//!
//! Neither a rule nor a constant lives here. `scan::calls` states who
//! calls whom inside one parse unit, `scan::metrics::events` states
//! what structure each unit holds, `CE.Scan.Complexity` folds the
//! three numbers and `CE.Scan.Cycles` charges the recursion point;
//! this module only moves indices in one direction and effective
//! values in the other — so the number the report renders and the
//! number the core graded are one number by construction, not by two
//! implementations agreeing.

use super::metrics::FileMetrics;
use super::report::FN_CODES;
use anyhow::{Result, ensure};

/// The three derived codes — cyclomatic, cognitive, nesting — and
/// therefore their offsets inside a function's block of rows.
const CYCLOMATIC: usize = 3;
const COGNITIVE: usize = 4;
const NESTING: usize = 5;

/// A function's row for one code, given its file's row offset.
fn row_of(offset: usize, unit: usize, code: usize) -> usize {
    offset + 1 + FN_CODES * unit + code - 1
}

/// The call arcs as GLOBAL row indices onto cognitive rows, strictly
/// ascending — the wire's `callEdges`. Every arc stays inside the
/// file that minted it, which is what makes the chunk invariant
/// enough to keep an arc whole.
pub fn arcs(files: &[FileMetrics], blocks: &[usize]) -> Vec<[u64; 2]> {
    let mut out = Vec::new();
    let mut offset = 0;
    for (file, block) in files.iter().zip(blocks) {
        for &(from, to) in &file.calls {
            out.push([
                row_of(offset, from as usize, COGNITIVE) as u64,
                row_of(offset, to as usize, COGNITIVE) as u64,
            ]);
        }
        offset += block;
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Every unit's events keyed by its cognitive row — the wire's
/// `events` table, ascending by (row, seq) by construction because
/// the files, the units and each unit's events are walked in row
/// order. A unit with no structure sends no row and is derived as
/// (1, 0, 0) by the core all the same: the key's presence, not a
/// row's, puts every unit on the road.
pub fn events(files: &[FileMetrics], blocks: &[usize]) -> Vec<Vec<i64>> {
    let mut out = Vec::new();
    let mut offset = 0;
    for (file, block) in files.iter().zip(blocks) {
        for (unit, f) in file.functions.iter().enumerate() {
            let row = row_of(offset, unit, COGNITIVE) as u64;
            out.extend(f.events.iter().map(|e| e.row(row)));
        }
        offset += block;
    }
    out
}

/// The core's `derived` written back onto the functions it names:
/// `[rowIndex, value]` for every cyclomatic, cognitive and nesting
/// row, so afterwards each unit's three numbers are the ones judged.
/// An index that is not such a row, a unit left without its three, or
/// a `cocBumped` row disagreeing with the derived value at the same
/// index is wire drift and is refused by name — this side proves the
/// shape, never the policy.
pub fn apply(
    files: &mut [FileMetrics],
    blocks: &[usize],
    derived: &[[u64; 2]],
    bumped: &[[u64; 2]],
) -> Result<()> {
    for &[index, value] in derived {
        let (file, unit, code) = seat(blocks, index as usize).with_context_row(
            "derived",
            index,
            "not a complexity row of any file",
        )?;
        let func = files
            .get_mut(file)
            .and_then(|f| f.functions.get_mut(unit))
            .with_context_row("derived", index, "outside the measured functions")?;
        let value = u32::try_from(value).unwrap_or(u32::MAX);
        match code {
            CYCLOMATIC => func.cyclomatic = value,
            COGNITIVE => func.cognitive = value,
            _ => func.max_nesting = value,
        }
    }
    let units: usize = files.iter().map(|f| f.functions.len()).sum();
    ensure!(
        derived.len() == 3 * units && derived.windows(2).all(|w| w[0][0] < w[1][0]),
        "derived: {} rows for {units} functions — every unit answers three, ascending",
        derived.len()
    );
    for &[index, value] in bumped {
        let (file, unit, code) = seat(blocks, index as usize).with_context_row(
            "cocBumped",
            index,
            "not a cognitive row of any file",
        )?;
        ensure!(
            code == COGNITIVE && u64::from(files[file].functions[unit].cognitive) == value,
            "cocBumped row {index}: the core answered {value} against its own derived cognitive value"
        );
    }
    Ok(())
}

/// The (file, function, code) a global row index seats, or None when
/// the index is not one of a function's three complexity rows.
fn seat(blocks: &[usize], index: usize) -> Option<(usize, usize, usize)> {
    let mut offset = 0;
    for (file, &block) in blocks.iter().enumerate() {
        if index < offset + block {
            let inside = index.checked_sub(offset + 1)?;
            let (unit, code) = (inside / FN_CODES, inside % FN_CODES + 1);
            return (CYCLOMATIC..=NESTING)
                .contains(&code)
                .then_some((file, unit, code));
        }
        offset += block;
    }
    None
}

/// `Option::context` with the offending table and row named the same
/// way in every place it is needed.
trait RowContext<T> {
    fn with_context_row(self, table: &str, index: u64, why: &str) -> Result<T>;
}

impl<T> RowContext<T> for Option<T> {
    fn with_context_row(self, table: &str, index: u64, why: &str) -> Result<T> {
        self.ok_or_else(|| anyhow::anyhow!("{table} row {index}: {why}"))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/scan/complexity.rs"]
mod tests;
