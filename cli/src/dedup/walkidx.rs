//! Whole-tree index maintenance for `ce dedup` (split out of
//! dedup/mod.rs at M5-2e to keep both under the 300-line dogfood
//! line): the walk feeds every language file through refresh_file —
//! Markdown included since schema v4, entering as zero-fingerprint
//! graph rows — collects the resolver-config hashes for the phase-2
//! resolve_key, and reloads token streams of fingerprint-sharing
//! files.

use super::{Params, index, pairs, tokens};
use crate::config::Config;
use crate::graph::store;
use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// One full-tree pass: what lives, how much of it carries tokens,
/// which files were content-refreshed (their edges cascade-dropped —
/// phase 1.5's work list), the resolver-config paths, and the
/// phase-2 key derived from the walked paths + config bytes.
pub(super) struct WalkIndex {
    pub live: BTreeSet<String>,
    pub configs: Vec<String>,
    pub tokenized: usize,
    pub dirty: BTreeSet<String>,
    pub resolve_key: i64,
}

pub(super) fn index_all(root: &Path, config: &Config, idx: &mut index::Index) -> Result<WalkIndex> {
    let mut out = WalkIndex {
        live: BTreeSet::new(),
        configs: Vec::new(),
        tokenized: 0,
        dirty: BTreeSet::new(),
        resolve_key: 0,
    };
    let mut configs: Vec<(String, u64)> = Vec::new();
    // md slug sets are resolver INPUTS like config bytes (the anchor
    // rung reads the target's headings), so they join the key — a
    // heading edit anywhere re-fires the phase-2 sweep (M5 close,
    // repaying the 2f cross-file staleness debt). Key inputs only:
    // Scope.configs stays real config paths.
    let mut md_facts: Vec<(String, u64)> = Vec::new();
    for path in walk::collect(root, &config.exclude).map_err(anyhow::Error::msg)? {
        let rel = rel_of(root, &path);
        if store::is_resolver_config(&path) {
            configs.push((rel, tokens::fnv1a(&std::fs::read(&path)?)));
            continue;
        }
        let Some(lang) = Lang::from_path(&path) else {
            continue;
        };
        let src = std::fs::read(&path)?;
        if lang == Lang::Markdown {
            let text = String::from_utf8_lossy(&src);
            md_facts.push((rel.clone(), crate::graph::ladder::md::slug_hash(&text)));
        }
        if idx.refresh_file(&rel, &src, lang, Params::default())? {
            out.dirty.insert(rel.clone());
        }
        if lang.grammar().is_some() {
            out.tokenized += 1;
        }
        out.live.insert(rel);
    }
    // collect() sorts and live is a BTreeSet — the key is a function
    // of the tree, not of walk order
    let mut key_inputs = configs.clone();
    key_inputs.extend(md_facts);
    out.resolve_key = store::resolve_key(&out.live, &key_inputs);
    out.configs = configs.into_iter().map(|(path, _)| path).collect();
    Ok(out)
}

/// Token streams for the files that share at least one fingerprint.
/// Every stream is fed back through refresh_file with the very bytes
/// just read: the content-hash fast path makes this free when nothing
/// changed, and re-indexes atomically when something did — stored
/// offsets can never disagree with the returned streams
/// (single-threaded; the M2 daemon serializes writers per ADR-003).
pub(super) fn load_streams(
    root: &Path,
    files: &BTreeSet<String>,
    idx: &mut index::Index,
    p: Params,
) -> Result<(pairs::Streams, BTreeSet<String>)> {
    let mut out = pairs::Streams::new();
    let mut changed = BTreeSet::new();
    for rel in files {
        let Some((path, lang)) = lang_path(root, rel) else {
            continue;
        };
        let src = std::fs::read(&path)?;
        if idx.refresh_file(rel, &src, lang, p)? {
            changed.insert(rel.clone());
        }
        out.insert(rel.clone(), tokens::stream(&src, lang)?);
    }
    Ok((out, changed))
}

/// rel → absolute path + language; None for non-lang paths (the
/// shared gate of this walk and the probe's candidate loop).
pub(super) fn lang_path(root: &Path, rel: &str) -> Option<(std::path::PathBuf, Lang)> {
    let path = root.join(rel);
    let lang = Lang::from_path(&path)?;
    Some((path, lang))
}

fn rel_of(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}
