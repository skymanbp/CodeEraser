//! `ce graph` orchestration (M5-2b: `--sites` only — the resolver
//! ladder lands at 2f, judgment at 2g). Walk → detect → aggregate;
//! the same detector feeds the frozen slice instrument, so this
//! module stays resolution-free by construction.

pub mod md;
pub mod sites;
pub mod spec;

use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

/// Every site of one file, path repo-relative with forward slashes.
pub struct FileSites {
    pub path: String,
    pub lang: Lang,
    pub sites: Vec<sites::RawSite>,
}

/// Detect reference sites across the tree (exclusion model = the
/// scan walker's: .gitignore + .ceignore + built-ins + ce.toml).
pub fn analyze(root: &Path) -> Result<Vec<FileSites>> {
    let (_config, candidates) = walk::scoped_lang_files(root).map_err(anyhow::Error::msg)?;
    let mut out = Vec::new();
    for (path, lang) in candidates {
        let text =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(FileSites {
            path: rel,
            lang,
            sites: sites::detect(&text, lang),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// `ce graph --sites` entry: per-(lang, kind) counts on console,
/// full rows as JSON.
pub fn run_sites(root: &Path, json: bool) -> ExitCode {
    match analyze(root) {
        Ok(files) if json => {
            println!("{}", rows_json(&files));
            ExitCode::SUCCESS
        }
        Ok(files) => {
            print_counts(&files);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("ce graph: {err:#}");
            ExitCode::from(2)
        }
    }
}

/// Site counts keyed (lang, kind) — the shape the slice doc freezes.
pub fn counts(files: &[FileSites]) -> BTreeMap<(&'static str, &'static str), usize> {
    let mut map = BTreeMap::new();
    for file in files {
        for site in &file.sites {
            *map.entry((file.lang.name(), site.kind)).or_insert(0) += 1;
        }
    }
    map
}

fn print_counts(files: &[FileSites]) {
    let total: usize = files.iter().map(|f| f.sites.len()).sum();
    println!(
        "graph sites: {total} across {} files",
        files.iter().filter(|f| !f.sites.is_empty()).count()
    );
    for ((lang, kind), n) in counts(files) {
        println!("  {lang:<10} {kind:<12} {n}");
    }
}

fn rows_json(files: &[FileSites]) -> String {
    let rows: Vec<serde_json::Value> = files
        .iter()
        .flat_map(|f| {
            f.sites.iter().map(|s| {
                serde_json::json!({
                    "path": f.path,
                    "lang": f.lang.name(),
                    "kind": s.kind,
                    "line": s.line,
                    "spec": s.spec,
                    "owner": s.owner,
                })
            })
        })
        .collect();
    serde_json::json!({ "sites": rows }).to_string()
}
