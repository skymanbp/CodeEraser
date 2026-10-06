//! `ce graph` orchestration (M5-2b: `--sites` only — the resolver
//! ladder lands at 2f, judgment at 2g). Walk → detect → aggregate;
//! the same detector feeds the frozen slice instrument, so this
//! module stays resolution-free by construction. Resolution lives in
//! ladder/ (Markdown and HTML) and in the core for Python, TypeScript /
//! TSX, Rust, Lua, Go, C / C++, R, Java and Haskell (resolve/, which
//! sends the package.json, tsconfig, go.mod, DESCRIPTION, .cabal,
//! pyproject and compile-database texts, the Cargo.toml documents, the
//! file-system and syntax-tree facts the TS and Rust rungs ask for and
//! the Java headers the core reads); wire.rs bridges cached sites to
//! edge rows for phase 2.

// The 92e728b1 go.mod and compile-database readers, the fa83a48d
// cabal reader, the dd0eec61 tsconfig chain and JSONC cleaner and the
// 1324c927 Cargo reader, frozen
// (tests subrepo unit/graph/oracle_cfg/): the
// frozen ladders (ladder::frozen) read them at their old paths, and the
// differential gate holds the core's readers against them (plan v2.33
// W2-text).
#[cfg(test)]
#[path = "../../tests/unit/graph/frozen_cfg.rs"]
pub(crate) mod oracle_cfg;
#[cfg(test)]
pub(crate) use oracle_cfg::{cabal, cargo, cmdline, compdb, compdb_flags, gomod, jsonc, roots_ts};

pub mod cabal_find;
pub mod canvas;
pub mod compdb_find;
pub mod deadcode;
pub mod keys;
pub mod ladder;
pub mod load;
pub mod md;
pub mod mounts;
pub mod nodes;
pub mod owed;
pub mod resolve;
pub mod roots;
pub mod sites;
pub mod spec;
pub mod store;
pub mod stored;
pub mod symbols;
pub mod symwire;
pub mod wire;

use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::Result;
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
/// full rows as JSON — the document and its lines the core lays out
/// (plan v2.32 steps 4-5).
pub fn run_sites(root: &Path, core: &str, json: bool) -> ExitCode {
    let answer = analyze(root).and_then(|files| sites_document(core, &files));
    crate::report::graph_face("sites", answer, json)
}

/// The sites document (CE.Graph.Sites): each file's place in path
/// order and its language, each site's integers with its kind by the
/// package's storage code (the `store` table), and the paths, specs
/// and owners the core spells in (and orders the paths by) — the
/// `--sites` face and the MCP tool print this one document.
pub fn sites_document(core: &str, files: &[FileSites]) -> Result<crate::document::Answer> {
    let mut rows: Vec<[i64; 6]> = Vec::new();
    let mut texts = SiteTexts::default();
    for (f, file) in files.iter().enumerate() {
        for s in &file.sites {
            let kind = store::kind_code(s.kind)?;
            let owned = i64::from(s.owner.is_some());
            rows.push([
                rows.len() as i64,
                f as i64,
                kind,
                s.line as i64,
                s.nth as i64,
                owned,
            ]);
            texts.specs.push(s.spec.clone());
            texts.owners.push(s.owner.clone().unwrap_or_default());
        }
        texts.paths.push(file.path.clone());
    }
    let req = crate::document::Request::new("sites")
        .range("files", files.len())
        .range("sites", rows.len())
        .rows(
            "langs",
            files
                .iter()
                .enumerate()
                .map(|(f, x)| [f as i64, x.lang as i64])
                .collect::<Vec<_>>(),
        )
        .rows("sites", rows)
        .text("path", texts.paths)
        .text("site_spec", texts.specs)
        .text("site_owner", texts.owners);
    crate::document::assemble(core, req)
}

/// The sites document's strings: the paths, each site's spec and owner.
#[derive(Default)]
struct SiteTexts {
    paths: Vec<String>,
    specs: Vec<String>,
    owners: Vec<String>,
}

#[cfg(test)]
#[path = "../../tests/unit/graph.rs"]
mod tests;
