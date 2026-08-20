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

/// Reads go through walk::read_surviving: a mid-walk deletion is a
/// skip and the next run converges (the survival rule scan and graph
/// already walk under); an unreadable file that EXISTS still aborts.
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
        let rel = walk::rel_str(root, &path);
        if store::is_resolver_config(&path) {
            configs.extend(walk::read_surviving(&path)?.map(|b| (rel, tokens::fnv1a(&b))));
            continue;
        }
        let Some(lang) = Lang::from_path(&path) else {
            continue;
        };
        let Some(src) = walk::read_surviving(&path)? else {
            continue; // vanished mid-walk: not live this pass
        };
        lang_fact(lang, &rel, &src, &mut md_facts);
        if idx.refresh_file(&rel, &src, lang, Params::default())? {
            out.dirty.insert(rel.clone());
        }
        if lang.grammar().is_some() {
            out.tokenized += 1;
        }
        out.live.insert(rel);
    }
    // collect() sorts and live is a BTreeSet — the key is a function
    // of the tree, not of walk order. TS fs facts (compiled-JS twins,
    // node_modules names) join like md slugs: the ladder stats them
    // but the walk can never carry them (clearance review MED — their
    // mutation previously never re-fired the sweep).
    let mut key_inputs = configs.clone();
    key_inputs.extend(md_facts);
    key_inputs.extend(crate::graph::keys::ts_fs_facts(root, &out.live));
    out.resolve_key = store::resolve_key(&out.live, &key_inputs);
    out.configs = configs.into_iter().map(|(path, _)| path).collect();
    Ok(out)
}

/// Cross-file resolver INPUTS per language (split from index_all at
/// the E01 fn gate): md heading slugs and the Rust pub-use surface
/// both join the resolve_key — the same discipline, one throat.
fn lang_fact(lang: Lang, rel: &str, src: &[u8], facts: &mut Vec<(String, u64)>) {
    let text = String::from_utf8_lossy(src);
    match lang {
        Lang::Markdown => facts.push((rel.to_string(), crate::graph::ladder::md::slug_hash(&text))),
        Lang::Rust => facts.push((
            rel.to_string(),
            crate::graph::ladder::rs_reexport::pubuse_hash(&text),
        )),
        _ => {}
    }
}

/// Token streams for the files that share at least one fingerprint.
/// Every stream is fed back through refresh_file with the very bytes
/// just read: the content-hash fast path makes this free when nothing
/// changed, and re-indexes atomically when something did — stored
/// offsets can never disagree with the returned streams (each
/// refresh is one idempotent transaction — the v1.7 convergent-
/// cache contract).
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
        let Some(src) = walk::read_surviving(&path)? else {
            continue;
        };
        if idx.refresh_file(rel, &src, lang, p)? {
            changed.insert(rel.clone());
        }
        out.insert(rel.clone(), tokens::stream(&src, lang)?);
    }
    Ok((out, changed))
}

/// Read-only streams for a pass that must never write (M5-close
/// review HIGH-1: the S1 candidate walk used load_streams and threw
/// its `changed` set away — a file saved mid-`ce clone` had its
/// cascade-dropped edges silently orphaned until an unrelated
/// resolve_key shift, because the stored content hash was already
/// updated). Fresh-read bytes; a stale stored offset is already the
/// extension pass's own guarded case (pairs::extend_anchor counts
/// stale_skipped, probe.rs walks the same line), and an unreadable
/// or untokenizable file simply provides no stream — "no claim
/// made", never an abort mid-candidates.
pub(super) fn read_streams(root: &Path, files: &BTreeSet<String>) -> pairs::Streams {
    let mut out = pairs::Streams::new();
    for rel in files {
        let Some((path, lang)) = lang_path(root, rel) else {
            continue;
        };
        let Ok(src) = std::fs::read(&path) else {
            continue;
        };
        if let Ok(stream) = tokens::stream(&src, lang) {
            out.insert(rel.clone(), stream);
        }
    }
    out
}

/// rel → absolute path + language; None for non-lang paths (the
/// shared gate of this walk and the probe's candidate loop).
pub(super) fn lang_path(root: &Path, rel: &str) -> Option<(std::path::PathBuf, Lang)> {
    let path = root.join(rel);
    let lang = Lang::from_path(&path)?;
    Some((path, lang))
}
