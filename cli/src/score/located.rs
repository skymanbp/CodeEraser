//! The extent law's four request tables (plan v2.35 score amendment):
//! what this run already read, mapped onto the verdict universe's row
//! indices and sent as it is. Every selection - which reference line
//! stays inside a cycle, which file is dead, which function is over
//! its ceiling, the union of the lines - is the core's
//! (CE.Verdict.ExtentFacts); nothing here filters by kind, SCC or
//! threshold.
//!
//! - `contSpans`: per continuous row, its file's universe index (-1 =
//!   the scan-only arm) and, for a function row, the scan's own span.
//! - `dupSpans`: both member spans of every pair a duplicate family
//!   verified - T1/T2 blocks and T3 unit pairs (kind 0 / 1), docdup
//!   segment pairs (kind 2) - pairs inside one file included (the sim
//!   table counts those out; this table must not).
//! - `refLines`: every reference line of an edge the graph was built
//!   from, self arcs included.
//! - `proseLines`: the line count of each universe file without a size
//!   row, counted by `scan::metrics::size::total_lines`, the function
//!   the scan counts a file's lines with.

use crate::dedup;
use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// The marker for a continuous row whose file the universe does not
/// place (the scan-only arm).
const OUTSIDE: i64 = -1;

/// The four tables, in their wire order.
pub type Tables = (Vec<[i64; 3]>, Vec<[i64; 4]>, Vec<[i64; 3]>, Vec<[i64; 2]>);

/// `contSpans`: aligned with the continuous rows (`homes` and `spans`
/// are their file paths and spans, row for row).
pub fn cont_spans(
    homes: &[String],
    spans: &[(i64, i64)],
    idx: &HashMap<&str, i64>,
) -> Vec<[i64; 3]> {
    homes
        .iter()
        .zip(spans)
        .map(|(p, &(lo, hi))| [idx.get(p.as_str()).copied().unwrap_or(OUTSIDE), lo, hi])
        .collect()
}

/// One member span of a verified pair: (path, lo, hi, kind).
pub type Member<'a> = (&'a str, i64, i64, i64);

/// The verified pairs' member spans of the three duplicate families,
/// as (path, lo, hi, kind).
pub fn members<'a>(
    blocks: &'a [dedup::pairs::Block],
    t3: &'a dedup::t3::Judged,
    segs: &'a [crate::docdup::judge::candidates::SegRow],
    dups: impl Iterator<Item = (usize, usize)> + 'a,
) -> Vec<Member<'a>> {
    let mut out: Vec<Member<'a>> = Vec::new();
    for b in blocks {
        out.push((b.a_file.as_str(), b.a_start as i64, b.a_end as i64, 0));
        out.push((b.b_file.as_str(), b.b_start as i64, b.b_end as i64, 0));
    }
    for &(a, b, _) in &t3.clones {
        for u in [&t3.units[a], &t3.units[b]] {
            out.push((u.path.as_str(), u.start_line, u.end_line, 1));
        }
    }
    for (a, b) in dups {
        for s in [&segs[a], &segs[b]] {
            out.push((s.path.as_str(), s.start_line, s.end_line, 2));
        }
    }
    out
}

/// `dupSpans`: the members on universe files, deduplicated, ascending.
pub fn dup_spans(members: &[Member<'_>], idx: &HashMap<&str, i64>) -> Vec<[i64; 4]> {
    let set: BTreeSet<[i64; 4]> = members
        .iter()
        .filter_map(|&(p, lo, hi, kind)| idx.get(p).map(|&u| [u, lo, hi, kind]))
        .collect();
    set.into_iter().collect()
}

/// `refLines`: [src, dst, line] for every edge row of the index whose
/// two files are universe files, deduplicated, ascending.
pub fn ref_lines(index: &dedup::index::Index, idx: &HashMap<&str, i64>) -> Result<Vec<[i64; 3]>> {
    let rows: Vec<(String, i64, String)> = crate::graph::load::rows(
        index.raw(),
        "SELECT f.path, s.line, e.dst_path FROM edges e JOIN sites s ON s.id = e.site_id
         JOIN files f ON f.id = s.file_id",
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let set: BTreeSet<[i64; 3]> = rows
        .iter()
        .filter_map(|(src, line, dst)| {
            Some([*idx.get(src.as_str())?, *idx.get(dst.as_str())?, *line])
        })
        .collect();
    Ok(set.into_iter().collect())
}

/// `proseLines`: every universe file that owns no size row, with its
/// line count as the scan counts it.
pub fn prose_lines(root: &Path, files: &[String], homes: &[String]) -> Result<Vec<[i64; 2]>> {
    let sized: BTreeSet<&str> = homes.iter().map(String::as_str).collect();
    files
        .iter()
        .enumerate()
        .filter(|(_, p)| !sized.contains(p.as_str()))
        .map(|(u, p)| {
            let src = std::fs::read(root.join(p))
                .with_context(|| format!("reading {p} for its line count"))?;
            Ok([
                u as i64,
                crate::scan::metrics::size::total_lines(&src) as i64,
            ])
        })
        .collect()
}
