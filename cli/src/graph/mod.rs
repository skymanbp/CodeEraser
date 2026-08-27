//! `ce graph` orchestration (M5-2b: `--sites` only — the resolver
//! ladder lands at 2f, judgment at 2g). Walk → detect → aggregate;
//! the same detector feeds the frozen slice instrument, so this
//! module stays resolution-free by construction. Resolution lives in
//! ladder/ with its config surfaces (cabal / cargo / gomod / roots /
//! jsonc) beside it; wire.rs bridges cached sites to edge rows for
//! phase 2.

pub mod cabal;
pub mod canvas;
pub mod cargo;
pub mod deadcode;
pub mod gomod;
pub mod jsonc;
pub mod keys;
pub mod ladder;
pub mod load;
pub mod md;
pub mod nodes;
pub mod roots;
pub mod sites;
pub mod spec;
pub mod store;
pub mod symbols;
pub mod symwire;
pub mod wire;

use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::Result;
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
/// scan walker's: .gitignore + .ceignore + built-ins + ce.toml;
/// mid-walk deletions degrade inside walk::each_surviving).
pub fn analyze(root: &Path) -> Result<Vec<FileSites>> {
    let (_config, rows) = walk::each_surviving(root, |path, lang, bytes| {
        // the scan-only arm is sized by scan, never graphed — its
        // rows would be all-empty noise on this face (plan v2.5;
        // review 2026-08-20 #4, the throat check is in sites.rs)
        if lang.scan_only() {
            return Ok(None);
        }
        // lossy on purpose: one stray non-UTF-8 file must not abort
        // the whole analysis (Opus review; matches the instrument,
        // which hashes exactly the text the detector saw)
        let text = String::from_utf8_lossy(&bytes);
        Ok(Some(FileSites {
            path: crate::scan::walk::rel_str(root, path),
            lang,
            sites: sites::detect(&text, lang),
        }))
    })?;
    let mut out: Vec<FileSites> = rows.into_iter().flatten().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// `ce graph --sites` entry: per-(lang, kind) counts on console,
/// full rows as JSON.
pub fn run_sites(root: &Path, json: bool) -> ExitCode {
    match analyze(root) {
        Ok(files) if json => {
            println!("{}", sites_json(&files));
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
        "{}",
        crate::i18n::line(
            "graph sites: {} across {} files",
            "图引用站点：{} 个，分布于 {} 个文件",
            &[
                &total,
                &files.iter().filter(|f| !f.sites.is_empty()).count(),
            ],
        )
    );
    // the per-(lang, kind) rows are pure data — nothing to translate
    for ((lang, kind), n) in counts(files) {
        println!("  {lang:<10} {kind:<12} {n}");
    }
}

/// The sites document's version identity — every other report family
/// carries one; the e2e matrix caught this face shipping bare JSON.
pub const SCHEMA_ID: &str = "ce.sites-report/0.1.0";

/// Full site rows as JSON — the `--sites` face and the MCP tool
/// share this one serialization.
pub fn sites_json(files: &[FileSites]) -> String {
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
    serde_json::json!({ "schema": SCHEMA_ID, "sites": rows }).to_string()
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

    /// The scan-only arm never reaches this face: before plan v2.5's
    /// boundary landed here, a grammarless .css/.js file fell through
    /// to the MARKDOWN detector and invented link sites (review
    /// 2026-08-20 #4).
    #[test]
    fn scan_only_files_are_neither_graphed_nor_md_fallback() {
        let dir = crate::testutil::scratch("graph-scanonly");
        let fixtures: [(&str, &[u8]); 2] = [
            ("style.css", b"/* [x](./style.css) */ a { color: red }\n"),
            ("app.js", b"// [doc](./app.js)\nconst x = 1;\n"),
        ];
        for (name, bytes) in fixtures {
            std::fs::write(dir.join(name), bytes).expect(name);
        }
        let files = analyze(&dir).expect("analyze");
        assert!(files.is_empty(), "no scan-only rows: {:?}", counts(&files));
        std::fs::remove_dir_all(&dir).ok();
    }
}
