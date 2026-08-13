//! `ce graph` orchestration (M5-2b: `--sites` only — the resolver
//! ladder lands at 2f, judgment at 2g). Walk → detect → aggregate;
//! the same detector feeds the frozen slice instrument, so this
//! module stays resolution-free by construction.

pub mod md;
pub mod sites;
pub mod spec;
pub mod store;

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
        // lossy on purpose: one stray non-UTF-8 file must not abort
        // the whole analysis (Opus review; matches the instrument,
        // which hashes exactly the text the detector saw)
        let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
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
                    "nth": s.nth,
                    "spec": s.spec,
                    "owner": s.owner,
                })
            })
        })
        .collect();
    serde_json::json!({ "sites": rows }).to_string()
}

#[cfg(test)]
mod tests {
    use super::{analyze, counts};

    /// End-to-end walk on a real (temp) tree — `ce graph --sites`'
    /// engine gets its own test instead of only a smoke number in a
    /// commit message (Opus review). Includes a non-UTF-8 in-scope
    /// file: lossy reading keeps the run alive and still detects.
    #[test]
    fn analyze_walks_counts_and_survives_non_utf8() {
        let dir = std::env::temp_dir().join(format!("ce-graph-walk-{}", std::process::id()));
        let fixtures: [(&str, &[u8]); 3] = [
            ("a.py", b"import os\n"),
            ("b.md", b"[x](./a.py)\n"),
            ("c.py", b"import sys\n\xff\xfe\n"),
        ];
        std::fs::create_dir_all(&dir).expect("mkdir");
        for (name, bytes) in fixtures {
            std::fs::write(dir.join(name), bytes).expect(name);
        }
        let files = analyze(&dir).expect("analyze");
        let by = counts(&files);
        assert_eq!(
            by.get(&("python", "import")),
            Some(&2),
            "lossy file still detected"
        );
        assert_eq!(by.get(&("markdown", "link")), Some(&1));
        std::fs::remove_dir_all(&dir).ok();
    }
}
