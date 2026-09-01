//! The recursion increment's measuring half (ADR-008 fourth
//! instalment, plan v2.23): the call arcs projected onto row indices
//! on the way out, and the values the core judged with written back
//! on the way in.
//!
//! Neither the cycle nor the constant 1 lives here. `scan::calls`
//! states who calls whom inside one parse unit, `CE.Scan.Cycles`
//! finds the cycles and charges the point, and this module only
//! moves indices in one direction and effective values in the other
//! — so the number the report renders, the number the pinned mirror
//! grades and the number the core graded are one number by
//! construction, not by two implementations agreeing.

use super::metrics::FileMetrics;
use super::report::FN_CODES;
use anyhow::{Result, ensure};

/// The cognitive metric's frozen code, and therefore its offset
/// inside a function's block of rows.
const COGNITIVE: usize = 4;

/// A function's cognitive row, given its file's row offset.
fn row_of(offset: usize, unit: usize) -> usize {
    offset + 1 + FN_CODES * unit + COGNITIVE - 1
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
                row_of(offset, from as usize) as u64,
                row_of(offset, to as usize) as u64,
            ]);
        }
        offset += block;
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The core's `cocBumped` written back onto the functions it names:
/// `[rowIndex, effectiveValue]`. An index that is not a cognitive
/// row, or a value below what was measured, is wire drift and is
/// refused by name — this side proves the shape, never the policy.
pub fn apply(files: &mut [FileMetrics], blocks: &[usize], bumped: &[[u64; 2]]) -> Result<()> {
    for &[index, value] in bumped {
        let (file, unit) = seat(blocks, index as usize)
            .with_context_row(index, "not a cognitive row of any file")?;
        let func = files
            .get_mut(file)
            .and_then(|f| f.functions.get_mut(unit))
            .with_context_row(index, "outside the measured functions")?;
        ensure!(
            value >= func.cognitive as u64,
            "cocBumped row {index}: the core answered {value} under the measured {}",
            func.cognitive
        );
        func.cognitive = u32::try_from(value).unwrap_or(u32::MAX);
    }
    Ok(())
}

/// The (file, function) a global row index seats, or None when the
/// index is not that file's cognitive row for some function.
fn seat(blocks: &[usize], index: usize) -> Option<(usize, usize)> {
    let mut offset = 0;
    for (file, &block) in blocks.iter().enumerate() {
        if index < offset + block {
            let inside = index.checked_sub(offset + 1)?;
            let unit = inside / FN_CODES;
            return (inside % FN_CODES == COGNITIVE - 1).then_some((file, unit));
        }
        offset += block;
    }
    None
}

/// `Option::context` with the offending row named the same way in
/// both places it is needed.
trait RowContext<T> {
    fn with_context_row(self, index: u64, why: &str) -> Result<T>;
}

impl<T> RowContext<T> for Option<T> {
    fn with_context_row(self, index: u64, why: &str) -> Result<T> {
        self.ok_or_else(|| anyhow::anyhow!("cocBumped row {index}: {why}"))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/scan/coc.rs"]
mod tests;
