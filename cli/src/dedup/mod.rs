//! `ce dedup` clone-detection hot path (plan ADR-005): normalized
//! token stream → winnowing/Rabin-Karp fingerprints (Schleimer et al.
//! SIGMOD'03) → SQLite inverted index → clone blocks. T1/T2 only
//! here; T3 is the M5 cold path.

pub mod index;
pub mod pairs;
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
/// 0.3.0: summary self-identifies its parameters; hot groups chain
/// instead of vanishing; stale-anchor counter added.
pub const SCHEMA_ID: &str = "ce.dedup-report/0.3.0";

/// Batch entry point: refresh the index (incremental), reap deleted
/// files, verify anchors by token-stream extension, report blocks.
/// `min_tokens` lowers the report filter below the guarantee t for
/// calibration runs; detection below t is opportunistic (anchors are
/// only guaranteed at >= t). Informational exit in M2.
pub fn run(
    root: &Path,
    format: Format,
    db: Option<PathBuf>,
    min_tokens: Option<usize>,
) -> Result<ExitCode> {
    let (found, summary) = analyze(root, db, min_tokens)?;
    emit(format, &found, &summary)?;
    Ok(ExitCode::SUCCESS)
}

/// Library entry shared by the CLI and the daemon: index, verify,
/// and return the blocks + summary without printing anything.
pub fn analyze(
    root: &Path,
    db: Option<PathBuf>,
    min_tokens: Option<usize>,
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
    let min = min_tokens.unwrap_or(p.guarantee());
    let found = pairs::clone_blocks(&instances, &streams.0, min);
    let summary = Summary {
        files: live.len(),
        refreshed,
        removed,
        blocks: found.blocks.len(),
        hot_chained: found.hot_chained,
        stale_skipped: found.stale_skipped,
        kgram: p.kgram,
        window: p.window,
        min_tokens: min,
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
    kgram: usize,
    window: usize,
    min_tokens: usize,
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
                "indexed {} files ({} refreshed, {} removed) — {} clone blocks (min {} tokens), {} hot chained, {} stale skipped",
                s.files,
                s.refreshed,
                s.removed,
                s.blocks,
                s.min_tokens,
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
