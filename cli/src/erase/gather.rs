//! The erase plan's measurement leg: candidate FACTS from the three
//! source families, through their own library entries (same caches,
//! same core, same knobs — erase.md §two-phases). Nothing here
//! decides safety; every fact row goes to the core's erase/1
//! predicate. Byte equality is measured on the raw line slices —
//! the families' masked/normalized equivalences are how candidates
//! are FOUND, byte identity is what licenses deletion.

use crate::erase::model::Candidate;
use crate::graph::deadcode::{self, Advisory};
use crate::scan::lang::Lang;
use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub struct Gathered {
    pub candidates: Vec<Candidate>,
    /// path → fnv1a64 of its full text at plan time.
    pub hashes: BTreeMap<String, u64>,
    pub out_of_class: BTreeMap<&'static str, usize>,
}

/// Cached unit spans per path: (key, start_line, end_line).
type UnitSpans = BTreeMap<String, Vec<(String, i64, i64)>>;

/// All three measurement legs in family order: deadcode (graph),
/// docdup (verbatim pairs), dedup (whole-unit T1 twins) — all off
/// ONE snapshot (batch 9 P10: this boundary used to pay the full
/// measurement four times, once per leg plus its own index open).
pub fn candidates(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Gathered> {
    let (found, idx, db_path) = crate::dedup::snapshot(root, db)?;
    // dead files license erase rows; the symbol advisory licenses
    // nothing here (W4-F2), so the wire is built without it
    let w = deadcode::wire_of(root, &idx, &db_path, Advisory::No)?;
    let dead = deadcode_leg(root, core, &w)?;
    let lang_unres = lang_unresolved(&idx)?;
    let units = units_by_path(&idx)?;
    let (segs, dups, _) = crate::docdup::judge::rows_of(root, &idx, core)?;
    drop(idx);
    let mut cache = TextCache::new(root);
    let mut cands = dead_candidates(&dead)?;
    cands.extend(doc_candidates(&segs, &dups, &mut cache)?);
    let mut out_of_class = BTreeMap::new();
    let dead_paths: BTreeSet<&str> = dead.dead.iter().map(|d| d.path.as_str()).collect();
    cands.extend(twin_candidates(
        &found.blocks,
        &units,
        &dead_paths,
        &lang_unres,
        &mut cache,
        &mut out_of_class,
    )?);
    let mut hashes = BTreeMap::new();
    for c in &cands {
        if !hashes.contains_key(&c.path) {
            let h = crate::dedup::tokens::fnv1a(cache.text(&c.path)?.as_bytes());
            hashes.insert(c.path.clone(), h);
        }
    }
    Ok(Gathered {
        candidates: cands,
        hashes,
        out_of_class,
    })
}

/// The graph family, degraded refused by name: an erase plan built
/// on a judgment that judged nothing would present "no dead files"
/// as a fact (the deadcode --check posture, main_cmds.rs).
fn deadcode_leg(
    root: &Path,
    core: &str,
    w: &crate::graph::deadcode::GraphWire,
) -> Result<crate::graph::deadcode::Report> {
    let r = crate::graph::deadcode::judge_report(root, core, w)?;
    if let Some(reason) = &r.degraded {
        bail!("deadcode leg degraded ({reason}) — an erase plan cannot stand on it");
    }
    Ok(r)
}

/// Unresolved-site owners folded to per-language counts (keyed by
/// the language's wire code — Lang itself carries no Ord on
/// purpose) — the trust fact class-2 twin rows still carry: a twin
/// row is not a graph dead row, so no core confidence exists for it
/// (the H3 scope line). The dead-file road consumes the core's
/// confidence column instead since 2.32.0.
fn lang_unresolved(idx: &crate::dedup::index::Index) -> Result<BTreeMap<i64, i64>> {
    let mut by_lang = BTreeMap::new();
    for p in crate::graph::load::unresolved_paths(idx)? {
        if let Some(l) = Lang::from_path(Path::new(&p)) {
            *by_lang.entry(l as i64).or_insert(0) += 1;
        }
    }
    Ok(by_lang)
}

fn lang_count(map: &BTreeMap<i64, i64>, path: &str) -> i64 {
    Lang::from_path(Path::new(path))
        .and_then(|l| map.get(&(l as i64)).copied())
        .unwrap_or(0)
}

/// Class-3 rows (2.32.0, H3): the trust fact is the graph family's
/// OWN per-row confidence — a reply without the column means the
/// ledger never rode, refused by name, never defaulted.
fn dead_candidates(dead: &crate::graph::deadcode::Report) -> Result<Vec<Candidate>> {
    dead.dead
        .iter()
        .map(|d| {
            let code = 1 + crate::graph::deadcode::VERDICT_NAMES
                .iter()
                .position(|v| *v == d.verdict)
                .unwrap_or(0) as i64;
            let conf = d
                .conf
                .context("dead row carries no confidence — the graph ledger did not ride")?;
            Ok(Candidate {
                class: 3,
                facts: [code, conf, 0, 0],
                path: d.path.clone(),
                span: None,
                provenance: format!("deadcode: {} — {}", d.verdict, d.why),
            })
        })
        .collect()
}

/// Verbatim doc duplicates: for every core-reported pair, the
/// path-lexicographically FIRST occurrence survives and the other is
/// the candidate (erase.md: the survivor is chosen deterministically,
/// never judged "better").
fn doc_candidates(
    segs: &[crate::docdup::judge::candidates::SegRow],
    dups: &[(usize, usize, crate::docdup::judge::Doc)],
    cache: &mut TextCache,
) -> Result<Vec<Candidate>> {
    let mut out = Vec::new();
    for (a, b, m) in dups {
        let key = |i: usize| (segs[i].path.as_str(), segs[i].start_line);
        let (keep, gone) = if key(*a) <= key(*b) {
            (*a, *b)
        } else {
            (*b, *a)
        };
        let (s, t) = (&segs[keep], &segs[gone]);
        let eq = cache.slice(&s.path, s.start_line, s.end_line)?
            == cache.slice(&t.path, t.start_line, t.end_line)?;
        out.push(Candidate {
            class: 1,
            facts: [m.verbatim as i64, s.words, t.words, i64::from(eq)],
            path: t.path.clone(),
            span: Some((t.start_line, t.end_line)),
            provenance: format!(
                "docdup: twin of {}:{}-{} (J {}/{}, verbatim {} words)",
                s.path, s.start_line, s.end_line, m.inter, m.union, m.verbatim
            ),
        });
    }
    Ok(out)
}

/// Whole-unit T1 twins: a dedup block side that covers at least one
/// ENTIRE cached unit, byte-identical across sides. The candidate
/// targets side b's FILE (unit-tier liveness is not unlocked — R6 —
/// so "the copy is graph-dead" can only mean its file is); blocks
/// with no whole-unit coverage are counted out-of-class, not
/// silently dropped.
fn twin_candidates(
    blocks: &[crate::dedup::pairs::Block],
    units: &UnitSpans,
    dead_paths: &BTreeSet<&str>,
    lang_unres: &BTreeMap<i64, i64>,
    cache: &mut TextCache,
    out_of_class: &mut BTreeMap<&'static str, usize>,
) -> Result<Vec<Candidate>> {
    let mut out = Vec::new();
    for blk in blocks {
        // the copy under sentence is the DEAD side when exactly one
        // is; with neither dead, side b stands in so the advisory
        // still names a concrete file (the row refuses either way)
        let a = (&blk.a_file, blk.a_start, blk.a_end);
        let b = (&blk.b_file, blk.b_start, blk.b_end);
        let ((tf, ts, te), (of, os, oe)) = if dead_paths.contains(blk.a_file.as_str())
            && !dead_paths.contains(blk.b_file.as_str())
        {
            (a, b)
        } else {
            (b, a)
        };
        let covered = covered_units(units, tf, ts, te);
        if covered.is_empty() {
            *out_of_class.entry("t1t2_block_no_whole_unit").or_insert(0) += 1;
            continue;
        }
        let eq = cache.slice(of, os as i64, oe as i64)? == cache.slice(tf, ts as i64, te as i64)?;
        let dead = dead_paths.contains(tf.as_str());
        out.push(Candidate {
            class: 2,
            facts: [
                // the REAL coverage bit, not a constant 1 (batch-7
                // defect sweep): the pre-filter above makes it true
                // by construction today, but a hard-coded claim left
                // the core's reason-5 unit_not_covered unreachable —
                // if the filter and this mint ever disagree, the
                // core now refuses instead of licensing the erase
                i64::from(!covered.is_empty()),
                i64::from(eq),
                i64::from(dead),
                lang_count(lang_unres, tf),
            ],
            path: tf.clone(),
            span: None,
            provenance: format!(
                "dedup: T1 block twin at {of}:{os}-{oe} ({} tokens), covers unit {}",
                blk.tokens, covered[0]
            ),
        });
    }
    Ok(out)
}

/// Cached unit spans grouped by path — the coverage predicate's table
/// (graph symbols are the ONE persisted unit identity, candidates.rs
/// precedent: join on the cache, never a re-segmentation).
fn units_by_path(idx: &crate::dedup::index::Index) -> Result<UnitSpans> {
    let mut map: UnitSpans = BTreeMap::new();
    for s in crate::graph::symbols::symbol_rows(idx)? {
        map.entry(s.path)
            .or_default()
            .push((s.key, s.start_line, s.end_line));
    }
    Ok(map)
}

/// Unit keys fully inside [start, end] on `path` (1-based inclusive).
fn covered_units(units: &UnitSpans, path: &str, start: usize, end: usize) -> Vec<String> {
    units
        .get(path)
        .map(|v| {
            v.iter()
                .filter(|(_, s, e)| *s >= start as i64 && *e <= end as i64)
                .map(|(k, _, _)| k.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// One decode per file through the family read throat; a file that
/// vanished mid-plan aborts by name (dedup::walked_text's contract —
/// silent drop = silent-clean).
pub struct TextCache {
    root: PathBuf,
    map: BTreeMap<String, String>,
}

impl TextCache {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            map: BTreeMap::new(),
        }
    }

    pub fn text(&mut self, path: &str) -> Result<&str> {
        if !self.map.contains_key(path) {
            let (text, _) = crate::dedup::walked_text(&self.root, path)?;
            self.map.insert(path.to_string(), text);
        }
        Ok(self.map.get(path).expect("just inserted"))
    }

    /// Lines s..=e (1-based inclusive), joined verbatim.
    pub fn slice(&mut self, path: &str, s: i64, e: i64) -> Result<String> {
        let text = self.text(path)?;
        let lines: Vec<&str> = text.lines().collect();
        let (a, b) = (s.max(1) as usize - 1, (e.max(0) as usize).min(lines.len()));
        Ok(lines
            .get(a..b)
            .with_context(|| format!("{path}: span {s}-{e} out of range"))?
            .join("\n"))
    }
}
