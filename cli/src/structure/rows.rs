//! What the structure and arch requests carry by path (plan v2.33 W1
//! item 3): the graph's measured files and their arcs, the S5 staleness
//! facts off one windowed git log, the S6 clone and dead files. Reading
//! only — the directory tree, the dir-keyed tables and every count over
//! them are the core's (CE.Structure.Tree, CE.Structure.Raw,
//! CE.Arch.Tables), and a path the tree cannot place is the core's fault
//! to name.

use crate::graph::deadcode::GraphWire;
use crate::graph::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The graph as both families send it: the measured file paths in the
/// wire's node order (path order — the node set is a BTreeSet), the
/// file-to-file arcs as slot pairs (one per graph edge between two
/// measured files, so a pair repeats once per kind and rung), a file's
/// arcs to a section folded onto the section's file, and its arcs to a
/// package as the package's directory path.
pub struct Arcs {
    pub paths: Vec<String>,
    pub files: Vec<[usize; 2]>,
    pub sections: Vec<[usize; 2]>,
    pub packages: Vec<(usize, String)>,
}

/// The arcs off one graph wire. The measured tier: a foreign reader and
/// a walked asset are not measured files and have no slot.
pub fn arcs(w: &GraphWire) -> Arcs {
    let measured = crate::graph::deadcode::measured_nodes(w);
    let index_of: BTreeMap<i64, usize> = (measured.iter().enumerate())
        .map(|(slot, &(i, _))| (i, slot))
        .collect();
    let paths: Vec<String> = measured.iter().map(|&(_, p)| p.to_string()).collect();
    let slot: BTreeMap<&str, usize> = (paths.iter().enumerate())
        .map(|(f, p)| (p.as_str(), f))
        .collect();
    let mut out = Arcs {
        files: Vec::new(),
        sections: Vec::new(),
        packages: Vec::new(),
        paths: Vec::new(),
    };
    for e in &w.edges {
        if let (Some(&a), Some(&b)) = (index_of.get(&e[0]), index_of.get(&e[1])) {
            out.files.push([a, b]);
        }
        let (src, dst) = (&w.nodes[e[0] as usize], &w.nodes[e[1] as usize]);
        let Some(&f) = slot
            .get(src.path.as_str())
            .filter(|_| src.kind == GRAN_FILE)
        else {
            continue;
        };
        match dst.kind {
            GRAN_SECTION => out
                .sections
                .extend(slot.get(dst.path.as_str()).map(|&g| [f, g])),
            GRAN_PACKAGE => out.packages.push((f, dst.path.clone())),
            _ => {}
        }
    }
    out.paths = paths;
    out
}

/// The S5 staleness facts, one row per Markdown doc with reference
/// targets (path order): the doc, its newest change in the window (0 =
/// unchanged, the one sentinel), and the newest changes of the targets
/// that changed in the window (target path order). The stale predicate
/// and the doc's directory are the core's (CE.Structure.Raw, deriveStale).
pub type StaleDocs = Vec<(String, u64, Vec<u64>)>;

/// Every Markdown file's outgoing reference targets (the md ladder's own
/// resolutions) against ONE windowed `git log` pass.
pub fn stale_docs(root: &Path, w: &GraphWire, days: u32) -> Result<StaleDocs> {
    let mut targets: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for e in &w.edges {
        let s = &w.nodes[e[0] as usize];
        let d = &w.nodes[e[1] as usize];
        // a foreign doc is READ (its links are edges) and never
        // measured: its staleness is its own tree's, and it has no
        // directory in this one to be placed in (plan v2.18 step #12)
        if !s.foreign && s.path.ends_with(".md") && d.path != s.path {
            targets.entry(s.path.as_str()).or_default().insert(&d.path);
        }
    }
    let newest = newest_in_window(root, days)?;
    Ok(targets
        .into_iter()
        .map(|(md, tgts)| {
            let changed = tgts
                .iter()
                .filter_map(|tg| newest.get(*tg))
                .map(|&t| t as u64);
            let doc_ts = newest.get(md).copied().unwrap_or(0) as u64;
            (md.to_string(), doc_ts, changed.collect())
        })
        .collect())
}

/// Newest commit time per touched file inside the window, one git
/// pass (churn's own runner — no second git throat). The \x01
/// sentinel keeps an all-digit FILENAME from parsing as a commit
/// time. Superproject history only: a file under a declared
/// submodule never appears in this log (its pointer bump does), so a
/// doc citing `cli/tests/…` can never be found stale by that target
/// here — one-directional (a missed staleness, never a false alarm),
/// named by `ce churn`'s report rather than widened onto this wire.
fn newest_in_window(root: &Path, days: u32) -> Result<BTreeMap<String, i64>> {
    let since = format!("--since={days} days ago");
    let log = crate::churn::git(root, &["log", &since, "--format=%x01%ct", "--name-only"])?;
    let mut newest: BTreeMap<String, i64> = BTreeMap::new();
    let mut cur = 0i64;
    for line in log.lines() {
        if let Some(ts) = line.strip_prefix('\u{1}') {
            cur = ts.trim().parse().context("commit time")?;
        } else if !line.is_empty() {
            let e = newest.entry(line.to_string()).or_insert(cur);
            *e = (*e).max(cur);
        }
    }
    Ok(newest)
}

/// The S6 facts: each clone block's two files, and the dead files of the
/// liveness judgment — the per-file families' own outputs, never
/// re-derived here; the core counts them per directory. A degraded
/// liveness judgment REFUSES the whole rollup (zeros it would send
/// instead are fakes, and the axis's honest absence road already exists
/// — drop --deep).
pub type Redundancy = (Vec<(String, String)>, Vec<String>);

pub fn redundancy(
    root: &Path,
    core: &str,
    w: &GraphWire,
    found: &crate::dedup::pairs::Blocks,
) -> Result<Redundancy> {
    let blocks = (found.blocks.iter())
        .map(|b| (b.a_file.clone(), b.b_file.clone()))
        .collect();
    // the judgment's dead rows alone: the rollup names no verdict, so
    // it asks for no document (plan v2.32 step 4)
    let (judged, _, _) = crate::graph::deadcode::judged(root, core, w, &[])?;
    if let Some(reason) = &judged.degraded {
        anyhow::bail!("liveness degraded ({reason}) — refusing a fake-zero S6 rollup");
    }
    let dead = (judged.dead.iter())
        .map(|row| w.nodes[row[0] as usize].path.clone())
        .collect();
    Ok((blocks, dead))
}

#[cfg(test)]
#[path = "../../tests/unit/structure/rows.rs"]
mod tests;
