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
pub const SCHEMA_ID: &str = "ce.dedup-report/0.1.0";

/// Batch entry point: refresh the index (incremental), reap deleted
/// files, report clone blocks. Informational exit in M2 — thresholds
/// and the ratchet arrive with guard wiring (M3).
pub fn run(root: &Path, format: Format, db: Option<PathBuf>) -> Result<ExitCode> {
    let config = Config::load(root).map_err(anyhow::Error::msg)?;
    let db_path = db.unwrap_or_else(|| root.join(".ce/index.db"));
    let mut idx = index::Index::open(&db_path)?;
    let (live, refreshed) = index_all(root, &config, &mut idx)?;
    let removed = idx.remove_missing(&live)?;
    let found = pairs::clone_blocks(&idx.all_instances()?);
    let summary = Summary {
        files: live.len(),
        refreshed,
        removed,
        blocks: found.blocks.len(),
        hot_skipped: found.hot_skipped,
    };
    emit(format, &found, &summary)?;
    Ok(ExitCode::SUCCESS)
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
struct Summary {
    files: usize,
    refreshed: usize,
    removed: usize,
    blocks: usize,
    hot_skipped: usize,
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
                    "dup {}:{}-{} <-> {}:{}-{} ({} fp)",
                    b.a_file, b.a_start, b.a_end, b.b_file, b.b_start, b.b_end, b.fingerprints
                );
            }
            println!(
                "indexed {} files ({} refreshed, {} removed) — {} clone blocks, {} hot hashes skipped",
                s.files, s.refreshed, s.removed, s.blocks, s.hot_skipped
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
