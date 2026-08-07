//! Clone-block reconstruction from shared fingerprints: group by
//! hash, form cross-location pairs, then merge line-adjacent pairs of
//! the same file pair into blocks (jscpd-comparable output for the
//! M2 recall/precision cross-check).
//!
//! Reporting threshold: a single shared fingerprint proves a common
//! run of >= kgram tokens (the noise floor), not >= t; the block
//! filter is calibrated at the jscpd cross-check, not here.

use super::index::Instance;
use serde::Serialize;
use std::collections::BTreeMap;

/// A hash shared by more locations than this is boilerplate (import
/// blocks, license headers); its pairs explode quadratically. Skipped
/// pairs are COUNTED and reported, never silently dropped.
const HOT_CAP: usize = 64;

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub a_file: String,
    pub a_start: usize,
    pub a_end: usize,
    pub b_file: String,
    pub b_start: usize,
    pub b_end: usize,
    /// Shared fingerprints merged into this block.
    pub fingerprints: usize,
}

#[derive(Debug, Serialize)]
pub struct Blocks {
    pub blocks: Vec<Block>,
    /// Hashes skipped by HOT_CAP (transparency, plan review B2).
    pub hot_skipped: usize,
}

pub fn clone_blocks(instances: &[Instance]) -> Blocks {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    let mut hot_skipped = 0;
    let mut pairs: Vec<Block> = Vec::new();
    for group in by_hash.values().filter(|g| g.len() > 1) {
        if group.len() > HOT_CAP {
            hot_skipped += 1;
            continue;
        }
        for (i, a) in group.iter().enumerate() {
            for b in &group[i + 1..] {
                if let Some(p) = pair(a, b) {
                    pairs.push(p);
                }
            }
        }
    }
    Blocks {
        blocks: merge(pairs),
        hot_skipped,
    }
}

/// Canonically-ordered pair; same-file overlapping ranges are the
/// window sliding over itself, not a clone.
fn pair(a: &Instance, b: &Instance) -> Option<Block> {
    let (a, b) = if (&a.file, a.start_line) <= (&b.file, b.start_line) {
        (a, b)
    } else {
        (b, a)
    };
    if a.file == b.file && b.start_line <= a.end_line {
        return None;
    }
    Some(Block {
        a_file: a.file.clone(),
        a_start: a.start_line,
        a_end: a.end_line,
        b_file: b.file.clone(),
        b_start: b.start_line,
        b_end: b.end_line,
        fingerprints: 1,
    })
}

/// Merge pairs of the same file pair whose A and B ranges both
/// overlap or adjoin (fingerprint windows tile a clone region).
fn merge(mut pairs: Vec<Block>) -> Vec<Block> {
    pairs.sort_by(|x, y| {
        (&x.a_file, &x.b_file, x.a_start, x.b_start)
            .cmp(&(&y.a_file, &y.b_file, y.a_start, y.b_start))
    });
    let mut out: Vec<Block> = Vec::new();
    for p in pairs {
        match out.last_mut() {
            Some(last) if joins(last, &p) => {
                last.a_end = last.a_end.max(p.a_end);
                last.b_end = last.b_end.max(p.b_end);
                last.fingerprints += 1;
            }
            _ => out.push(p),
        }
    }
    out
}

fn joins(cur: &Block, next: &Block) -> bool {
    cur.a_file == next.a_file
        && cur.b_file == next.b_file
        && next.a_start <= cur.a_end + 1
        && next.b_start <= cur.b_end + 1
}
