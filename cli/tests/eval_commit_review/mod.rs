//! Mechanical partition machinery for the commit-slice ground truth
//! (semantics: the retired commit-labels battery, git history). The
//! per-item review record is corpus-specific DATA and lives in
//! self.json / requests.json (data as data — as Rust consts the
//! parallel tables read as clone blocks to our own dedup ratchet);
//! every entry there was verified against the raw diff it describes.
//! The machinery here is corpus-neutral.
//!
//! Compiled by eval_l2, which uses a subset — the unused remainder
//! is expected.
#![allow(dead_code)]

use crate::eval_support::{BodyLine, commit_color_diff, walk_color_diff};
use codeeraser::fourclass::significant;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// A named corpus's review record, embedded at compile time (all
/// three parse once). corrections: reviewed content-coincidence
/// entries {sha, file, added, lines, why}. relocated_units: reviewed
/// relocation targets {sha, to, units}. CI gates iterate every
/// frozen corpus, so resolution is BY NAME — resolving through the
/// active corpus would silently read the wrong record there (the
/// active-corpus form below serves the generators).
fn tables_for(corpus: Option<&str>) -> &'static Value {
    static TABLES: OnceLock<[(Option<&'static str>, Value); 3]> = OnceLock::new();
    let parse = |raw: &str| serde_json::from_str(raw).expect("review record json");
    let all = TABLES.get_or_init(|| {
        [
            (None, parse(include_str!("self.json"))),
            (Some("requests"), parse(include_str!("requests.json"))),
            (Some("ripgrep"), parse(include_str!("ripgrep.json"))),
        ]
    });
    &all.iter()
        .find(|(n, _)| *n == corpus)
        .unwrap_or_else(|| panic!("no review record for corpus {corpus:?}"))
        .1
}

/// The ACTIVE corpus's review record — the generators' view.
fn tables() -> &'static Value {
    static NAME: OnceLock<Option<String>> = OnceLock::new();
    let name = NAME.get_or_init(crate::eval_support::corpus_name);
    tables_for(name.as_deref())
}

/// A record's `key` rows whose sha prefix matches `sha`.
fn rows_of<'a>(
    t: &'static Value,
    key: &str,
    sha: &'a str,
) -> impl Iterator<Item = &'static Value> + use<'a> {
    t[key]
        .as_array()
        .expect("review rows")
        .iter()
        .filter(move |r| sha.starts_with(r["sha"].as_str().expect("sha prefix")))
}

/// The ACTIVE corpus's `key` rows for `sha` (generator view).
fn rows_for<'a>(key: &str, sha: &'a str) -> impl Iterator<Item = &'static Value> + use<'a> {
    rows_of(tables(), key, sha)
}

pub type PerFile = HashMap<String, u64>;

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

/// Project a record's review rows for one sha: carry `fields`
/// through, split the comma-joined units — the one shape both
/// registers share.
fn project(t: &'static Value, key: &str, sha: &str, fields: &[&str]) -> Vec<Value> {
    rows_of(t, key, sha)
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

/// The named corpus's unit register (CI-gate view — see tables_for).
pub fn units_in(corpus: Option<&str>, sha: &str) -> Vec<Value> {
    project(tables_for(corpus), "relocated_units", sha, &["to"])
}

/// The reviewed below-floor register for one sha (M5-1d): true
/// relocated lines whose destination offers no >=2-distinct
/// CONTIGUOUS companion, so no site can open (destFloor) — the
/// miss-side mirror of the extras ledger, itemized per line. Rows:
/// (side, file, 1-based line).
pub fn below_floor_for(sha: &str) -> Vec<(String, String, usize)> {
    below_floor(rows_for("below_floor", sha), sha)
}

fn below_floor<'a>(
    rows: impl Iterator<Item = &'a Value>,
    sha: &str,
) -> Vec<(String, String, usize)> {
    let rows: Vec<_> = rows
        .map(|r| {
            (
                r["side"].as_str().expect("side").to_string(),
                r["file"].as_str().expect("file").to_string(),
                r["line"].as_u64().expect("line") as usize,
            )
        })
        .collect();
    // Every consumer waives per ROW; a duplicated row would widen
    // the miss allowance silently (Codex review C3). Asserted here,
    // the one throat every projection shares.
    let distinct: HashSet<_> = rows.iter().collect();
    assert_eq!(distinct.len(), rows.len(), "{sha}: duplicate register row");
    rows
}

/// The reviewed source->destination edge layer (M5-1c-iii): one row
/// per (from file, to file) edge with the units that rode it. Units
/// absent from every edge row are arrival-level GT only.
/// The named corpus's edge register (CI-gate view — see tables_for).
pub fn edges_in(corpus: Option<&str>, sha: &str) -> Vec<Value> {
    project(tables_for(corpus), "relocation_edges", sha, &["from", "to"])
}
