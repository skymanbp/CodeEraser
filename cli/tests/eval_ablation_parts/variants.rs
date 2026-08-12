//! The ablation's judgment variants and their scoring. Each variant
//! is a deterministic FORMALIZATION of one upgrade-menu candidate
//! (the menu named directions, not specs; the exact predicate below
//! is what the frozen matrix is a claim about — see EVAL-SET.md).
#![allow(dead_code)]

use super::ShadowBlock;
use crate::eval_l2_parts as parts;
use crate::eval_support::git_run;
use codeeraser::dedup::tokens::fnv1a;
use codeeraser::fourclass::significant;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// The quality VARIANT's anchor threshold — kept at 20 as the
/// one-notch-stricter control now that the core itself enforces
/// ANCHOR_FLOOR = 19 (M5-1c-iii): the frozen decision matrix was
/// measured at 20, and the surviving delta between the two is
/// exactly the thinnest real anchor observed (19).
pub const QUALITY_ALNUM: usize = 20;

#[derive(Clone, Copy, PartialEq)]
pub enum Variant {
    Baseline,
    Quality,
    Freq,
    Chain,
    Flow,
    Phase3Edge,
}

pub const ALL: [(&str, Variant); 6] = [
    ("baseline", Variant::Baseline),
    ("quality", Variant::Quality),
    ("freq", Variant::Freq),
    ("chain", Variant::Chain),
    ("flow", Variant::Flow),
    ("phase3_edge", Variant::Phase3Edge),
];

/// Per-commit lookup context: evidence content by hash, and the
/// base-tree trim frequency the freq variant judges rarity against.
pub struct Ctx {
    pub contents: HashMap<u64, String>,
    pub freq: HashMap<String, u64>,
}

impl Ctx {
    pub fn build(sha: &str, texts: &parts::Texts, blocks: &[ShadowBlock]) -> Ctx {
        Ctx {
            contents: content_map(texts),
            // The tree scan is the expensive part; commits without a
            // single candidate site never consult it.
            freq: if blocks.is_empty() {
                HashMap::new()
            } else {
                base_freq(sha)
            },
        }
    }
}

/// hash -> trimmed content for every significant line of the batch,
/// both sides. Evidence hashes always resolve here (leftovers hash
/// exactly these lines); a collision would confuse the core the same
/// way, so it refuses loudly.
fn content_map(texts: &parts::Texts) -> HashMap<u64, String> {
    let mut map: HashMap<u64, String> = HashMap::new();
    for (before, after, _) in texts {
        for ln in before.lines().chain(after.lines()) {
            let t = ln.trim();
            if !significant(t) {
                continue;
            }
            if let Some(old) = map.insert(fnv1a(t.as_bytes()), t.to_string()) {
                assert_eq!(old, t, "fnv1a collision");
            }
        }
    }
    map
}

/// Trim-equal line frequency over the commit's BASE tree, restricted
/// to the slice languages with the memory/ excludes applied by hand
/// (ls-tree pathspec magic is not portable across git versions).
fn base_freq(sha: &str) -> HashMap<String, u64> {
    let base = format!("{sha}^");
    let mut freq: HashMap<String, u64> = HashMap::new();
    // --full-tree pins root-relative paths: a bare ls-tree run from
    // cli/ (cargo's CWD) prints prefix-relative names, which the
    // root-relative `show rev:path` then cannot resolve.
    for f in git_run(
        &["ls-tree", "-r", "--full-tree", "--name-only", &base],
        false,
    )
    .lines()
    {
        let ext = f.rsplit('.').next().unwrap_or("");
        if !matches!(ext, "py" | "ts" | "rs" | "go" | "md")
            || f.starts_with("memory/")
            || f.starts_with("cli/memory/")
        {
            continue;
        }
        for ln in git_run(&["show", &format!("{base}:{f}")], false).lines() {
            let t = ln.trim();
            if !t.is_empty() {
                *freq.entry(t.to_string()).or_insert(0) += 1;
            }
        }
    }
    freq
}

/// The widest evidence line a site carries, in alphanumeric chars
/// (codeeraser::fourclass::alnum_width — the same rule the aligner
/// ships to the core, one source).
pub fn anchor_alnum(b: &ShadowBlock, ctx: &Ctx) -> usize {
    b.hashes
        .iter()
        .map(|h| codeeraser::fourclass::alnum_width(&ctx.contents[h]))
        .max()
        .unwrap_or(0)
}

/// Apply a variant's site filter. Phase3Edge keeps every site — its
/// knob lives in the phase-3 pass, not here.
pub fn filter(v: Variant, blocks: Vec<ShadowBlock>, ctx: &Ctx) -> Vec<ShadowBlock> {
    match v {
        Variant::Baseline | Variant::Phase3Edge => blocks,
        Variant::Quality => blocks
            .into_iter()
            .filter(|b| anchor_alnum(b, ctx) >= QUALITY_ALNUM)
            .collect(),
        // "repository frequency weighting": a site needs one evidence
        // line whose content is UNIQUE in the base tree (a rare line
        // has provenance identity; a common one matches anywhere).
        Variant::Freq => blocks
            .into_iter()
            .filter(|b| {
                b.hashes
                    .iter()
                    .any(|h| ctx.freq.get(ctx.contents[h].as_str()) == Some(&1))
            })
            .collect(),
        Variant::Chain => chain_filter(blocks),
        Variant::Flow => flow_filter(blocks),
    }
}

/// "non-crossing block chain": per (from, to) pair edge keep the
/// maximum-total-lines subset whose source AND destination starts
/// both strictly increase (weighted LIS); crossing losers drop.
fn chain_filter(blocks: Vec<ShadowBlock>) -> Vec<ShadowBlock> {
    let mut groups: BTreeMap<(usize, usize), Vec<ShadowBlock>> = BTreeMap::new();
    for b in blocks {
        groups.entry((b.from_pair, b.to_pair)).or_default().push(b);
    }
    let mut out = Vec::new();
    for (_, mut g) in groups {
        g.sort_by_key(|b| (b.from_lines[0], b.to_lines[0]));
        let mut best: Vec<(usize, Option<usize>)> = Vec::with_capacity(g.len());
        for i in 0..g.len() {
            let mut b = (g[i].from_lines.len(), None);
            for j in 0..i {
                let chained =
                    g[j].from_lines[0] < g[i].from_lines[0] && g[j].to_lines[0] < g[i].to_lines[0];
                if chained && best[j].0 + g[i].from_lines.len() > b.0 {
                    b = (best[j].0 + g[i].from_lines.len(), Some(j));
                }
            }
            best.push(b);
        }
        let mut keep = vec![false; g.len()];
        let mut i = (0..g.len()).max_by_key(|&i| best[i].0);
        while let Some(k) = i {
            keep[k] = true;
            i = best[k].1;
        }
        out.extend(g.into_iter().zip(keep).filter(|(_, k)| *k).map(|(b, _)| b));
    }
    out
}

/// "destination exclusivity" (the b-matching / fixed-charge-flow
/// direction, greedy formalization): sites claim their destination
/// lines in (more lines, then canonical position) priority; a site
/// overlapping an earlier claim drops whole.
fn flow_filter(mut blocks: Vec<ShadowBlock>) -> Vec<ShadowBlock> {
    blocks.sort_by(|x, y| {
        (
            std::cmp::Reverse(x.from_lines.len()),
            x.from_pair,
            &x.from_lines,
            x.to_pair,
        )
            .cmp(&(
                std::cmp::Reverse(y.from_lines.len()),
                y.from_pair,
                &y.from_lines,
                y.to_pair,
            ))
    });
    let mut claimed: std::collections::HashSet<(usize, usize)> = Default::default();
    let mut kept = Vec::new();
    for b in blocks {
        if b.to_lines
            .iter()
            .any(|&l| claimed.contains(&(b.to_pair, l)))
        {
            continue;
        }
        claimed.extend(b.to_lines.iter().map(|&l| (b.to_pair, l)));
        kept.push(b);
    }
    super::canon(&mut kept);
    kept
}

/// One variant's per-commit score against the cross GT, in the
/// summable [count_hits, count_misses, id_misses, invention,
/// extras_files, extras_lines, pred, blocks_dropped] shape (hits and
/// misses are COUNT-level; id_misses carries the line-identity gap).
/// Misses are DATA here, not failures — that is the point of an
/// ablation row. blocks_dropped is set by the driver, not metrics:
/// identical output does NOT mean a filter never fired (a dropped
/// site's lines can be re-covered by siblings and phases 2/3).
#[derive(Default)]
pub struct Row(pub [u64; 8]);

impl Row {
    pub fn add(&mut self, o: &Row) {
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            *a += b;
        }
    }
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&v| v == 0)
    }
    pub fn to_json(&self) -> Value {
        json!(self.0.to_vec())
    }
}

pub fn metrics(
    delta: &BTreeMap<parts::FileKey, Vec<usize>>,
    gt: &parts::CrossGt,
    has_cross: bool,
) -> Row {
    let mut keys: Vec<&parts::FileKey> = gt.counts.keys().chain(delta.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut row = Row::default();
    let empty = Vec::new();
    for key in keys {
        let g = gt.counts.get(key).copied().unwrap_or(0);
        let lines = delta.get(key).unwrap_or(&empty);
        let p = lines.len() as u64;
        // Slots: 0 count_hits, 1 count_misses, 2 id_misses,
        //        3 invention, 4 extras_files, 5 extras_lines,
        //        6 pred (7 blocks_dropped is the driver's).
        row.0[0] += g.min(p);
        row.0[1] += g - g.min(p);
        row.0[6] += p;
        if p > g {
            row.0[4] += 1;
            row.0[5] += p - g;
        }
        if !has_cross {
            row.0[3] += p;
        }
        if let Some(want) = gt.lines.get(key) {
            row.0[2] += want.iter().filter(|l| !lines.contains(l)).count() as u64;
        }
    }
    row
}
