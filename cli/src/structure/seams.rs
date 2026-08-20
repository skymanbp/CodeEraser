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
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The measured facts plus the name ledger the reply relabels with.
/// The wire rows live in a `wire::SeamTables` directly — the same
/// type the request carries, so measurement and codec cannot hold
/// two spellings of one shape (the ratchet flagged the mirror).
#[derive(Default)]
pub struct SeamFacts {
    /// (repo-relative path, total lines), index = wire fileId.
    pub files: Vec<(String, u64)>,
    /// Per file: (unit name, end line), index = wire unitId.
    pub unit_names: Vec<Vec<(String, u64)>>,
    /// The five request tables, assembled in place; the clone and
    /// churn legs (v2.7 ②) reuse family facts, never re-derived.
    pub tables: super::wire::SeamTables,
}

/// Mention names shorter than this are noise, not references (`new`,
/// `run`, `id` would edge every unit to every other) — an honest v1
/// floor, recorded in the booklet.
const NAME_FLOOR: usize = 3;

/// Churn window for the co-change leg — the §4.1 two-week anchor the
/// churn family reports by default; a measurement constant, not a
/// wire knob (prices are the knobs).
const CHURN_WINDOW_DAYS: u32 = 14;

/// Assemble the five seam tables over the judged files past `soft`.
pub fn seam_facts(
    root: &Path,
    db: Option<PathBuf>,
    files: &[FileMetrics],
    soft: u64,
) -> Result<SeamFacts> {
    let mut out = SeamFacts::default();
    let mut key_maps: Vec<BTreeMap<String, u64>> = Vec::new();
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
        let all = units::segments(&text, lang);
        let tops = top_level(&all);
        let fid = out.files.len() as u64;
        out.tables.files.push([fid, f.total_lines as u64]);
        out.files.push((f.path.clone(), f.total_lines as u64));
        key_maps.push(key_map(&all, &tops));
        push_units(&mut out, fid, &tops, f.total_lines as u64);
        push_refs(&mut out, fid, &tops, &text);
    }
    push_clones(&mut out, root, db)?;
    push_churn(&mut out, root, &key_maps);
    Ok(out)
}

/// T1/T2 clone-block spans inside the seam files, off the SAME index
/// the dedup family judges from — [fileId, start, end] rows for the
/// core's cut price (a seam through a block splits one duplicate
/// span over two files).
fn push_clones(out: &mut SeamFacts, root: &Path, db: Option<PathBuf>) -> Result<()> {
    if out.files.is_empty() {
        return Ok(());
    }
    let ids: BTreeMap<&str, (u64, u64)> = out
        .files
        .iter()
        .enumerate()
        .map(|(i, (p, total))| (p.as_str(), (i as u64, *total)))
        .collect();
    let (found, _stats) = crate::dedup::analyze(root, db, None, None)?;
    let mut rows = BTreeSet::new();
    for b in &found.blocks {
        let sides = [
            (&b.a_file, b.a_start, b.a_end),
            (&b.b_file, b.b_start, b.b_end),
        ];
        for (f, s, e) in sides {
            if let Some(&(fid, total)) = ids.get(f.as_str()) {
                let (s, e) = ((s as u64).max(1), (e as u64).min(total));
                if s <= e {
                    rows.insert([fid, s, e]);
                }
            }
        }
    }
    out.tables.clones = rows.into_iter().collect();
    Ok(())
}

/// Unit co-change pairs off the churn family's own commit ledger
/// over the window — [fileId, a, b] rows, a < b, distinct. Ledger
/// keys are joined onto the CURRENT snapshot's top-level units
/// (key-level; renamed units drop out honestly), and a tree without
/// git history prices the leg at zero rather than failing the
/// advisory.
fn push_churn(out: &mut SeamFacts, root: &Path, key_maps: &[BTreeMap<String, u64>]) {
    if out.files.is_empty() {
        return;
    }
    let ids: BTreeMap<&str, u64> = out
        .files
        .iter()
        .enumerate()
        .map(|(i, (p, _))| (p.as_str(), i as u64))
        .collect();
    let mut pairs = BTreeSet::new();
    for sha in seam_commits(root, &out.files).unwrap_or_default() {
        let mut touched: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
        for row in crate::churn::commit_ledger(root, &sha) {
            if let Some(&fid) = ids.get(row.path.as_str())
                && let Some(&top) = key_maps[fid as usize].get(&row.key)
            {
                touched.entry(fid).or_default().insert(top);
            }
        }
        for (fid, tops) in touched {
            for (i, &a) in tops.iter().enumerate() {
                for &b in tops.iter().skip(i + 1) {
                    pairs.insert([fid, a, b]);
                }
            }
        }
    }
    out.tables.churn = pairs.into_iter().collect();
}

/// Window commits touching any seam file (narrowed at git, so the
/// per-commit ledger classify only runs where a seam could care).
fn seam_commits(root: &Path, files: &[(String, u64)]) -> Result<Vec<String>> {
    let since = format!("{CHURN_WINDOW_DAYS} days ago");
    let mut args = vec![
        "log",
        "--since",
        &since,
        "--first-parent",
        "--no-merges",
        "--format=%H",
        "--",
    ];
    args.extend(files.iter().map(|(p, _)| p.as_str()));
    let stdout = crate::churn::git(root, &args)?;
    Ok(stdout.split_whitespace().map(str::to_string).collect())
}

/// Current-snapshot unit key -> owning top-level unit index — the
/// churn-ledger join surface (key-level; the ledger's nth caveat is
/// inherited, and the advisory face tolerates the proxy).
fn key_map(all: &[units::Unit], tops: &[units::Unit]) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for u in all {
        let owner = tops
            .iter()
            .position(|t| t.start_line <= u.start_line && u.end_line <= t.end_line);
        if let Some(t) = owner {
            m.entry(u.key.clone()).or_insert(t as u64);
        }
    }
    m
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
        out.tables
            .units
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
                out.tables.refs.push([fid, i as u64, j as u64]);
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
        let mut out = SeamFacts::default();
        push_refs(&mut out, 0, &tops, text);
        // beta_two mentions alpha_one; ab's beta_two_x is NOT a
        // word-bounded beta_two (identifier tail) — no edge
        assert_eq!(out.tables.refs, vec![[0, 1, 0]]);
        assert!(!mentions("xalpha_one()", "alpha_one"), "left bound");
    }
}
