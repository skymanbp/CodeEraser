//! `ce dedup` clone-detection hot path (plan ADR-005): normalized
//! token stream → winnowing/Rabin-Karp fingerprints (Schleimer et al.
//! SIGMOD'03) → SQLite inverted index → clone blocks. T1/T2 only
//! here; T3 is the M5 cold path.

pub mod candidates;
pub mod groups;
pub mod index;
pub mod minhash;
pub mod pairs;
pub mod probe;
pub(crate) mod schema;
mod sources;
pub mod struct_fp;
pub mod tokens;
pub mod unitcache;
mod walkidx;
pub mod winnow;

use crate::config::Config;
use crate::graph::{ladder, store, wire};
use crate::scan::Format;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// JSON output schema id; bump on shape change (plan §7.1).
/// 0.5.0: clone groups — k-way families aggregated from pairwise
/// blocks (attack-review R8 denominator inflation); 0.4.0 added the
/// calibrated diversity floor (min_distinct, default 7).
pub const SCHEMA_ID: &str = "ce.dedup-report/0.5.0";

/// CLI options for [`run`] (bundled: six loose params would trip the
/// project's own params threshold).
pub struct RunOpts {
    pub format: Format,
    pub db: Option<PathBuf>,
    pub min_tokens: Option<usize>,
    pub min_distinct: Option<usize>,
    pub check: bool,
}

/// Batch entry point: refresh the index (incremental), reap deleted
/// files, verify anchors by token-stream extension, report blocks.
/// `min_tokens` lowers the report filter below the guarantee t for
/// calibration runs; detection below t is opportunistic (anchors are
/// only guaranteed at >= t). `check` turns the run into the R12
/// ratchet gate against ce.toml [dedup] budget.
pub fn run(root: &Path, opts: RunOpts) -> Result<ExitCode> {
    let (found, summary) = analyze(root, opts.db, opts.min_tokens, opts.min_distinct)?;
    emit(opts.format, &found, &summary)?;
    if opts.check {
        return check_budget(root, summary.blocks);
    }
    Ok(ExitCode::SUCCESS)
}

/// R12 only-shrink ratchet: over budget fails; under budget prints
/// the new floor so the budget gets ratcheted down in the same PR.
fn check_budget(root: &Path, blocks: usize) -> Result<ExitCode> {
    let cfg = Config::load(root).map_err(anyhow::Error::msg)?;
    let Some(budget) = cfg.dedup.budget else {
        anyhow::bail!("--check needs [dedup] budget in ce.toml");
    };
    if blocks > budget {
        eprintln!(
            "dedup ratchet: {blocks} clone blocks > budget {budget} — new duplication must not land"
        );
        return Ok(ExitCode::FAILURE);
    }
    if blocks < budget {
        println!(
            "dedup ratchet: {blocks} clone blocks < budget {budget} — ratchet the budget down"
        );
    }
    Ok(ExitCode::SUCCESS)
}

/// Refresh the index for `root` and open it — the one opening every
/// cached-graph consumer (deadcode, `ce clone --units`) walks; the
/// path rides back for error messages that name the database.
pub fn refreshed_index(root: &Path, db: Option<PathBuf>) -> Result<(index::Index, PathBuf)> {
    analyze(root, db.clone(), None, None)?;
    let db_path = db.unwrap_or_else(|| root.join(".ce/index.db"));
    let idx = index::Index::open(&db_path, Params::default())?;
    Ok((idx, db_path))
}

/// Library entry shared by the CLI and the daemon: index, verify,
/// and return the blocks + summary without printing anything.
pub fn analyze(
    root: &Path,
    db: Option<PathBuf>,
    min_tokens: Option<usize>,
    min_distinct: Option<usize>,
) -> Result<(pairs::Blocks, Summary)> {
    let config = Config::load(root).map_err(anyhow::Error::msg)?;
    let db_path = db.unwrap_or_else(|| root.join(".ce/index.db"));
    let p = Params::default();
    let mut idx = index::Index::open(&db_path, p)?;
    let walked = walkidx::index_all(root, &config, &mut idx)?;
    let removed = idx.remove_missing(&walked.live)?;
    let mut instances = idx.all_instances()?;
    let streams = walkidx::load_streams(root, &pairs::candidate_files(&instances), &mut idx, p)?;
    if !streams.1.is_empty() {
        // a file changed between refresh and stream load; the streams
        // were re-fed into the index, so re-fetch the instances to
        // keep offsets and streams consistent (attack-review D1)
        instances = idx.all_instances()?;
    }
    resolve_edges(&mut idx, root, &walked, &streams.1)?;
    let filter = pairs::Filter {
        min_tokens: min_tokens.unwrap_or(p.guarantee()),
        min_distinct: min_distinct.unwrap_or(pairs::DEFAULT_MIN_DISTINCT),
    };
    let found = pairs::clone_blocks(&instances, &streams.0, filter);
    let summary = Summary {
        // has_tokens files only: Markdown rows are graph cache, and
        // counting them would silently change dedup-report/0.5.0
        files: walked.tokenized,
        refreshed: walked.dirty.len(),
        removed,
        blocks: found.blocks.len(),
        groups: found.groups.len(),
        hot_chained: found.hot_chained,
        stale_skipped: found.stale_skipped,
        low_diversity_suppressed: found.low_diversity_suppressed,
        kgram: p.kgram,
        window: p.window,
        min_tokens: filter.min_tokens,
        min_distinct: filter.min_distinct,
    };
    Ok((found, summary))
}

/// Phases 2 and 1.5 (design §3), AFTER every refresh of the run so
/// the sweep covers all current sites: the ladder resolver rides the
/// key gate — a stable key touches nothing, a shifted key sweeps
/// every edge. Without a sweep, content refreshes (walk or stream
/// reload) kept the key but cascade-dropped their files' edges; the
/// per-file pass restores exactly those.
fn resolve_edges(
    idx: &mut index::Index,
    root: &Path,
    walked: &walkidx::WalkIndex,
    stream_dirty: &BTreeSet<String>,
) -> Result<()> {
    let scope = ladder::Scope {
        files: &walked.live,
        configs: &walked.configs,
        root,
    };
    let mut resolver = |s: &store::CachedSite| wire::edges(s, &scope);
    if idx.ensure_edges_resolved(walked.resolve_key, &mut resolver)? {
        return Ok(());
    }
    let mut dirty = walked.dirty.clone();
    dirty.extend(stream_dirty.iter().cloned());
    idx.resolve_refreshed(&dirty, &mut resolver)
}

/// The dedup report as a self-contained JSON value (daemon wire use).
pub fn report_json(found: &pairs::Blocks, s: &Summary) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(Report {
        schema: SCHEMA_ID,
        blocks: &found.blocks,
        groups: &found.groups,
        summary: s,
    })?)
}

#[derive(Serialize)]
pub struct Summary {
    files: usize,
    refreshed: usize,
    removed: usize,
    blocks: usize,
    groups: usize,
    hot_chained: usize,
    stale_skipped: usize,
    low_diversity_suppressed: usize,
    kgram: usize,
    window: usize,
    min_tokens: usize,
    min_distinct: usize,
}

#[derive(Serialize)]
struct Report<'a> {
    schema: &'static str,
    blocks: &'a [pairs::Block],
    groups: &'a [groups::Group],
    summary: &'a Summary,
}

fn emit(format: Format, found: &pairs::Blocks, s: &Summary) -> Result<()> {
    match format {
        Format::Console => {
            for b in &found.blocks {
                println!(
                    "dup {}:{}-{} <-> {}:{}-{} ({} tokens)",
                    b.a_file, b.a_start, b.a_end, b.b_file, b.b_start, b.b_end, b.tokens
                );
            }
            println!(
                "indexed {} files ({} refreshed, {} removed) — {} clone blocks in {} groups (min {} tokens, distinct >= {}), {} low-diversity suppressed, {} hot chained, {} stale skipped",
                s.files,
                s.refreshed,
                s.removed,
                s.blocks,
                s.groups,
                s.min_tokens,
                s.min_distinct,
                s.low_diversity_suppressed,
                s.hot_chained,
                s.stale_skipped
            );
        }
        Format::Json => {
            let rep = Report {
                schema: SCHEMA_ID,
                blocks: &found.blocks,
                groups: &found.groups,
                summary: s,
            };
            println!("{}", serde_json::to_string_pretty(&rep)?);
        }
    }
    Ok(())
}

/// Winnowing parameters. Guarantee threshold t = matches of at least
/// `t` normalized tokens are always detected (SIGMOD'03 correctness
/// bound); noise threshold k = matches shorter than `kgram` tokens are
/// never reported. window = t - k + 1.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub kgram: usize,
    pub window: usize,
}

impl Params {
    /// Winnowing guarantee threshold t: every match of at least this
    /// many tokens shares a fingerprint — also the report threshold.
    pub fn guarantee(&self) -> usize {
        self.window + self.kgram - 1
    }
}

impl Default for Params {
    /// t = 50 tokens aligns with the jscpd min-tokens default
    /// (plan §4.1 clone row); k = 25 → window 26.
    fn default() -> Self {
        Self {
            kgram: 25,
            window: 26,
        }
    }
}
