//! Split-ROI seam measurement (plan v2.6 §C): for every judged file
//! past the committed soft line, the top-level unit spans and the
//! intra-file mention edges — dense integer rows for the core's
//! advisory, names kept HERE for the relabel (§5.9.2). NOTHING in
//! this module judges: viability, pricing and exemption are the
//! core's (CE.Structure.Split), the structure family's own
//! measurement/judgment split (edges.rs precedent).

use crate::fourclass::units;
use crate::scan::lang::Lang;
use crate::scan::metrics::FileMetrics;
use anyhow::Result;
use std::path::Path;

/// The measured facts plus the name ledger the reply relabels with.
pub struct SeamFacts {
    /// (repo-relative path, total lines), index = wire fileId.
    pub files: Vec<(String, u64)>,
    /// Per file: (unit name, end line), index = wire unitId.
    pub unit_names: Vec<Vec<(String, u64)>>,
    pub file_rows: Vec<[u64; 2]>,
    pub unit_rows: Vec<[u64; 4]>,
    pub ref_rows: Vec<[u64; 3]>,
}

/// Mention names shorter than this are noise, not references (`new`,
/// `run`, `id` would edge every unit to every other) — an honest v1
/// floor, recorded in the booklet.
const NAME_FLOOR: usize = 3;

/// Assemble the three seam tables over the judged files past `soft`.
pub fn seam_facts(root: &Path, files: &[FileMetrics], soft: u64) -> Result<SeamFacts> {
    let mut out = SeamFacts {
        files: Vec::new(),
        unit_names: Vec::new(),
        file_rows: Vec::new(),
        unit_rows: Vec::new(),
        ref_rows: Vec::new(),
    };
    for f in files {
        let Some(lang) = Lang::judged_path(Path::new(&f.path)) else {
            continue;
        };
        if (f.total_lines as u64) <= soft {
            continue;
        }
        let Ok(bytes) = std::fs::read(root.join(&f.path)) else {
            continue; // mid-walk deletion: the file left the universe
        };
        let text = String::from_utf8_lossy(&bytes);
        let tops = top_level(&units::segments(&text, lang));
        let fid = out.files.len() as u64;
        out.file_rows.push([fid, f.total_lines as u64]);
        out.files.push((f.path.clone(), f.total_lines as u64));
        push_units(&mut out, fid, &tops, f.total_lines as u64);
        push_refs(&mut out, fid, &tops, &text);
    }
    Ok(out)
}

/// Outermost, non-overlapping, start-ordered units: a nested helper
/// belongs to its holder's side of every seam, so only the outer
/// span rides the wire.
fn top_level(all: &[units::Unit]) -> Vec<units::Unit> {
    let mut tops: Vec<units::Unit> = Vec::new();
    let mut sorted: Vec<&units::Unit> = all.iter().collect();
    sorted.sort_by_key(|u| (u.start_line, std::cmp::Reverse(u.end_line)));
    for u in sorted {
        let contained = tops
            .last()
            .is_some_and(|t| t.start_line <= u.start_line && u.end_line <= t.end_line);
        let overlaps = tops.last().is_some_and(|t| u.start_line <= t.end_line);
        if !contained && !overlaps {
            tops.push(u.clone());
        }
    }
    tops
}

fn push_units(out: &mut SeamFacts, fid: u64, tops: &[units::Unit], total: u64) {
    let mut names = Vec::new();
    for (i, u) in tops.iter().enumerate() {
        let end = (u.end_line as u64).min(total);
        out.unit_rows
            .push([fid, i as u64, u.start_line as u64, end]);
        let name = u.key.split('/').next().unwrap_or("").to_string();
        names.push((name, end));
    }
    out.unit_names.push(names);
}

/// One mention edge per (from, to) unit pair where `to`'s bare name
/// appears word-bounded inside `from`'s span — the measurable v1
/// proxy for "internal references a seam would sever" (§C cost).
fn push_refs(out: &mut SeamFacts, fid: u64, tops: &[units::Unit], text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let span = |u: &units::Unit| {
        let lo = u.start_line.saturating_sub(1).min(lines.len());
        let hi = u.end_line.min(lines.len());
        lines[lo..hi].join("\n")
    };
    let bodies: Vec<String> = tops.iter().map(span).collect();
    for (i, body) in bodies.iter().enumerate() {
        for (j, target) in tops.iter().enumerate() {
            let name = target.key.split('/').next().unwrap_or("");
            if i == j || name.len() < NAME_FLOOR {
                continue;
            }
            if mentions(body, name) {
                out.ref_rows.push([fid, i as u64, j as u64]);
            }
        }
    }
}

/// Word-bounded containment: the name with no identifier character
/// on either side.
fn mentions(body: &str, name: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut from = 0;
    while let Some(pos) = body[from..].find(name) {
        let at = from + pos;
        let before_ok = body[..at].chars().next_back().is_none_or(|c| !ident(c));
        let after_ok = body[at + name.len()..]
            .chars()
            .next()
            .is_none_or(|c| !ident(c));
        if before_ok && after_ok {
            return true;
        }
        from = at + name.len();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two Rust functions where the second mentions the first by
    /// name: one edge (1 -> 0), none the other way, and the
    /// three-char noise floor drops short names.
    #[test]
    fn mention_edges_are_word_bounded_and_floored() {
        let text =
            "fn alpha_one() { 1 }\nfn beta_two() { alpha_one() }\nfn ab() { beta_two_x() }\n";
        let tops = top_level(&units::segments(text, crate::scan::lang::Lang::Rust));
        assert_eq!(tops.len(), 3, "three top-level units");
        let mut out = SeamFacts {
            files: Vec::new(),
            unit_names: Vec::new(),
            file_rows: Vec::new(),
            unit_rows: Vec::new(),
            ref_rows: Vec::new(),
        };
        push_refs(&mut out, 0, &tops, text);
        // beta_two mentions alpha_one; ab's beta_two_x is NOT a
        // word-bounded beta_two (identifier tail) — no edge
        assert_eq!(out.ref_rows, vec![[0, 1, 0]]);
        assert!(!mentions("xalpha_one()", "alpha_one"), "left bound");
    }
}
