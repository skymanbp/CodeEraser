//! Mechanical partition machinery for the commit-slice ground truth
//! (see eval_commit_labels.rs for the semantics). The per-item review
//! record is corpus-specific DATA and lives in self.json /
//! requests.json (data as data — as Rust consts the parallel tables
//! read as clone blocks to our own dedup ratchet); every entry there
//! was verified against the raw diff it describes. The machinery
//! here is corpus-neutral.
//!
//! Compiled independently by eval_commit_labels and eval_l2, each
//! using a subset — the unused remainder is expected.
#![allow(dead_code)]

use crate::eval_support::{BodyLine, commit_color_diff, walk_color_diff};
use codeeraser::fourclass::significant;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// The active corpus's review record, embedded at compile time and
/// resolved once (corpus() pins external window ends via git — not a
/// per-row cost). corrections: reviewed content-coincidence entries
/// {sha, file, added, lines, why}. relocated_units: reviewed
/// relocation targets {sha, to, units}.
fn tables() -> &'static Value {
    static TABLES: OnceLock<Value> = OnceLock::new();
    TABLES.get_or_init(|| {
        let raw = match crate::eval_support::corpus().name.as_deref() {
            None => include_str!("self.json"),
            Some("requests") => include_str!("requests.json"),
            Some("ripgrep") => include_str!("ripgrep.json"),
            Some(other) => panic!("no review record for corpus {other}"),
        };
        serde_json::from_str(raw).expect("review record json")
    })
}

/// The record's `key` rows whose sha prefix matches `sha`.
fn rows_for<'a>(key: &str, sha: &'a str) -> impl Iterator<Item = &'static Value> + use<'a> {
    tables()[key]
        .as_array()
        .expect("review rows")
        .iter()
        .filter(move |r| sha.starts_with(r["sha"].as_str().expect("sha prefix")))
}

pub type PerFile = HashMap<String, u64>;

pub fn total(m: &PerFile) -> u64 {
    m.values().sum()
}

#[derive(Default)]
pub struct SideBuckets {
    pub nonsig: PerFile,
    pub cross: PerFile,
    /// Line identities behind `cross`, per file (attack review F2:
    /// counts alone cannot see a same-count substitution). A file
    /// with a reviewed correction is REMOVED from this map — the
    /// corrected lines' identities were never archived, so only its
    /// count (plus the coincidence-exact gate) remains authoritative.
    pub cross_lines: HashMap<String, Vec<usize>>,
    pub within: u64,
}

/// Moved-line partition of one commit, per direction (out = removed
/// side, into = added side).
#[derive(Default)]
pub struct Partition {
    pub out: SideBuckets,
    pub into: SideBuckets,
}

impl Partition {
    fn side_mut(&mut self, added: bool) -> &mut SideBuckets {
        if added { &mut self.into } else { &mut self.out }
    }
}

/// Trimmed content → files carrying it, per side, significant moved
/// lines only (a significant line's partner is significant too).
type SigSets<'a> = HashMap<&'a str, HashSet<&'a str>>;
fn sig_sets(lines: &[BodyLine]) -> (SigSets<'_>, SigSets<'_>) {
    let (mut removed, mut added) = (SigSets::new(), SigSets::new());
    for l in lines {
        if !significant(&l.content) {
            continue;
        }
        let map = if l.added { &mut added } else { &mut removed };
        map.entry(l.content.trim())
            .or_default()
            .insert(l.own_path());
    }
    (removed, added)
}

/// Mechanical layers 1+2 for one commit: significance filter, then
/// cross/within partition of the significant moved lines.
pub fn partition(sha: &str) -> Partition {
    let base = format!("{sha}^");
    let raw = commit_color_diff(&base, sha);
    let lines: Vec<BodyLine> = walk_color_diff(&raw)
        .into_iter()
        .filter(|l| l.moved)
        .collect();
    let (removed, added) = sig_sets(&lines);
    let mut p = Partition::default();
    for l in &lines {
        let own = l.own_path();
        let opposite = if l.added { &removed } else { &added };
        let side = p.side_mut(l.added);
        if !significant(&l.content) {
            *side.nonsig.entry(own.into()).or_default() += 1;
        } else {
            let files = opposite
                .get(l.content.trim())
                .unwrap_or_else(|| panic!("{sha}: unpaired moved line {:?}", l.content));
            if files.contains(own) {
                side.within += 1;
            } else {
                *side.cross.entry(own.into()).or_default() += 1;
                side.cross_lines.entry(own.into()).or_default().push(l.line);
            }
        }
    }
    p
}

/// Layer 3: move the reviewed coincidence lines out of the mechanical
/// cross bucket (they all land there — a coincidence has no
/// within-file partner). Returns the applied entries for the record.
pub fn apply_corrections(sha: &str, p: &mut Partition) -> Vec<Value> {
    let mut applied = Vec::new();
    for r in rows_for("corrections", sha) {
        let (file, added) = (r["file"].as_str().expect("file"), r["added"] == true);
        let n = r["lines"].as_u64().expect("lines");
        let side_b = p.side_mut(added);
        let c = side_b
            .cross
            .get_mut(file)
            .unwrap_or_else(|| panic!("{sha}: correction target {file} not cross"));
        assert!(*c >= n, "{sha}: correction exceeds cross count of {file}");
        *c -= n;
        if *c == 0 {
            side_b.cross.remove(file);
        }
        // which n lines were the coincidence is not archived: the
        // file's line identities are no longer trustworthy
        side_b.cross_lines.remove(file);
        let side = if added { "added" } else { "removed" };
        applied.push(json!({"file": file, "side": side, "lines": n, "why": r["why"]}));
    }
    applied
}

/// The corrections' per-file line counts for one side of one commit.
pub fn per_file_corrections(sha: &str, added: bool) -> PerFile {
    rows_for("corrections", sha)
        .filter(|r| (r["added"] == true) == added)
        .map(|r| {
            let file = r["file"].as_str().expect("file").to_string();
            (file, r["lines"].as_u64().expect("lines"))
        })
        .collect()
}

/// Project review rows for one sha: carry `fields` through, split
/// the comma-joined units — the one shape both registers share.
fn project(key: &str, sha: &str, fields: &[&str]) -> Vec<Value> {
    rows_for(key, sha)
        .map(|r| {
            let mut o = serde_json::Map::new();
            for f in fields {
                o.insert((*f).into(), r[*f].clone());
            }
            let units = r["units"].as_str().expect("units");
            let split: Vec<&str> = units.split(',').map(str::trim).collect();
            o.insert("units".into(), json!(split));
            Value::Object(o)
        })
        .collect()
}

pub fn units_for(sha: &str) -> Vec<Value> {
    project("relocated_units", sha, &["to"])
}

/// The reviewed below-floor register for one sha (M5-1d): true
/// relocated lines whose destination offers no >=2-distinct
/// CONTIGUOUS companion, so no site can open (destFloor) — the
/// miss-side mirror of the extras ledger, itemized per line. Rows:
/// (side, file, 1-based line).
pub fn below_floor_for(sha: &str) -> Vec<(String, String, usize)> {
    rows_for("below_floor", sha)
        .map(|r| {
            (
                r["side"].as_str().expect("side").to_string(),
                r["file"].as_str().expect("file").to_string(),
                r["line"].as_u64().expect("line") as usize,
            )
        })
        .collect()
}

/// The reviewed source->destination edge layer (M5-1c-iii): one row
/// per (from file, to file) edge with the units that rode it. Units
/// absent from every edge row are arrival-level GT only.
pub fn edges_for(sha: &str) -> Vec<Value> {
    project("relocation_edges", sha, &["from", "to"])
}
