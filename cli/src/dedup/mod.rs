//! `ce dedup` clone-detection hot path (plan ADR-005): normalized
//! token stream → winnowing/Rabin-Karp fingerprints (Schleimer et al.
//! SIGMOD'03) → SQLite inverted index → clone blocks. T1/T2 only
//! here; T3 is the M5 cold path.

pub mod index;
pub mod pairs;
pub mod probe;
pub mod tokens;
pub mod winnow;

use crate::config::Config;
use crate::scan::{Format, lang::Lang, walk};
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// JSON output schema id; bump on shape change (plan §7.1).
/// 0.4.0: calibrated diversity floor (min_distinct, default 7) with
/// suppressed-count transparency; 0.3.0 added parameter self-id.
pub const SCHEMA_ID: &str = "ce.dedup-report/0.4.0";

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
    let (live, refreshed) = index_all(root, &config, &mut idx)?;
    let removed = idx.remove_missing(&live)?;
    let mut instances = idx.all_instances()?;
    let streams = load_streams(root, &pairs::candidate_files(&instances), &mut idx, p)?;
    if streams.1 > 0 {
        // a file changed between refresh and stream load; the streams
        // were re-fed into the index, so re-fetch the instances to
        // keep offsets and streams consistent (attack-review D1)
        instances = idx.all_instances()?;
    }
    let filter = pairs::Filter {
        min_tokens: min_tokens.unwrap_or(p.guarantee()),
        min_distinct: min_distinct.unwrap_or(pairs::DEFAULT_MIN_DISTINCT),
    };
    let found = pairs::clone_blocks(&instances, &streams.0, filter);
    let summary = Summary {
        files: live.len(),
        refreshed,
        removed,
        blocks: found.blocks.len(),
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

/// The dedup report as a self-contained JSON value (daemon wire use).
pub fn report_json(found: &pairs::Blocks, s: &Summary) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(Report {
        schema: SCHEMA_ID,
        blocks: &found.blocks,
        summary: s,
    })?)
}

/// Token streams for the files that share at least one fingerprint.
/// Every stream is fed back through refresh_file with the very bytes
/// just read: the content-hash fast path makes this free when nothing
/// changed, and re-indexes atomically when something did — stored
/// offsets can never disagree with the returned streams
/// (single-threaded; the M2 daemon serializes writers per ADR-003).
fn load_streams(
    root: &Path,
    files: &BTreeSet<String>,
    idx: &mut index::Index,
    p: Params,
) -> Result<(pairs::Streams, usize)> {
    let mut out = pairs::Streams::new();
    let mut changed = 0;
    for rel in files {
        let path = root.join(rel);
        let Some(lang) = Lang::from_path(&path) else {
            continue;
        };
        let src = std::fs::read(&path)?;
        if idx.refresh_file(rel, &src, lang, p)? {
            changed += 1;
        }
        out.insert(rel.clone(), tokens::stream(&src, lang)?);
    }
    Ok((out, changed))
}

fn index_all(
    root: &Path,
    config: &Config,
    idx: &mut index::Index,
) -> Result<(BTreeSet<String>, usize)> {
    let mut live = BTreeSet::new();
    let mut refreshed = 0;
    for path in walk::collect(root, &config.exclude).map_err(anyhow::Error::msg)? {
        let Some(lang) = Lang::from_path(&path) else {
            continue;
        };
        if lang.grammar().is_none() {
            continue; // Markdown: size-only, no token stream
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        let src = std::fs::read(&path)?;
        if idx.refresh_file(&rel, &src, lang, Params::default())? {
            refreshed += 1;
        }
        live.insert(rel);
    }
    Ok((live, refreshed))
}

#[derive(Serialize)]
pub struct Summary {
    files: usize,
    refreshed: usize,
    removed: usize,
    blocks: usize,
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
                "indexed {} files ({} refreshed, {} removed) — {} clone blocks (min {} tokens, distinct >= {}), {} low-diversity suppressed, {} hot chained, {} stale skipped",
                s.files,
                s.refreshed,
                s.removed,
                s.blocks,
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
