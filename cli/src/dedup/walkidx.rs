//! Whole-tree index maintenance for `ce dedup` (split out of
//! dedup/mod.rs at M5-2e to keep both under the 300-line dogfood
//! line): the walk feeds every language file through refresh_file —
//! Markdown included since schema v4, entering as zero-fingerprint
//! graph rows — collects the resolver-config hashes for the phase-2
//! resolve_key, and reloads token streams of fingerprint-sharing
//! files.

use super::{Params, index, pairs, tokens};
use crate::config::Config;
use crate::graph::ladder::{java_header, lua_path};
use crate::graph::store;
use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One full-tree pass: what lives, how much of it carries tokens,
/// which files were content-refreshed (their edges cascade-dropped —
/// phase 1.5's work list), the resolver-config paths, and the
/// phase-2 key derived from the walked paths + config bytes.
pub(super) struct WalkIndex {
    pub live: BTreeSet<String>,
    /// The walked files the index never holds — no judged language —
    /// by path only: the HTML rungs' asset candidates (plan v2.30 step
    /// 5), hashed into the resolve_key like `live`, so an asset added
    /// or removed re-fires the sweep and a content edit does not.
    pub assets: BTreeSet<String>,
    pub configs: Vec<String>,
    pub tokenized: usize,
    pub dirty: BTreeSet<String>,
    pub resolve_key: i64,
    /// ce.toml `[graph] crate_roots` ∩ live: the Rust ladder's declared
    /// roots (a declaration naming a missing file declares nothing).
    pub crate_roots: BTreeSet<String>,
    /// ce.toml `[graph.search_roots]`: language → directories, each
    /// holding at least one walked file (plan v2.30).
    pub search_roots: BTreeMap<String, BTreeSet<String>>,
    /// Every walked Java file's header (plan v2.30 step 3): the Java
    /// ladder's candidate index, handed over through ladder::Scope.
    pub java: BTreeMap<String, java_header::Header>,
    /// The templates the walked Lua files assign to `package.path`
    /// (plan v2.30 step 4): search directories the Lua ladder adds to
    /// its defaults, handed over through ladder::Scope.
    pub lua: BTreeSet<lua_path::Template>,
}

/// One full-tree pass: the walk (refresh_tree) and then the phase-2
/// key over every resolver input — config bytes, the `[graph]`
/// declarations, md slug sets, TS fs facts.
pub(super) fn index_all(root: &Path, config: &Config, idx: &mut index::Index) -> Result<WalkIndex> {
    let mut out = WalkIndex {
        live: BTreeSet::new(),
        assets: BTreeSet::new(),
        configs: Vec::new(),
        tokenized: 0,
        dirty: BTreeSet::new(),
        resolve_key: 0,
        crate_roots: BTreeSet::new(),
        search_roots: BTreeMap::new(),
        java: BTreeMap::new(),
        lua: BTreeSet::new(),
    };
    // md slug sets are resolver INPUTS like config bytes (the anchor
    // rung reads the target's headings), so they join the key — a
    // heading edit anywhere re-fires the phase-2 sweep (M5 close,
    // repaying the 2f cross-file staleness debt). Key inputs only:
    // Scope.configs stays real config paths.
    let mut md_facts: Vec<(String, u64)> = Vec::new();
    let configs = refresh_tree(root, config, idx, &mut out, &mut md_facts)?;
    // collect() sorts and live is a BTreeSet — the key is a function
    // of the tree, not of walk order. TS fs facts (compiled-JS twins,
    // node_modules names) join like md slugs: the ladder stats them
    // but the walk can never carry them (clearance review MED — their
    // mutation previously never re-fired the sweep).
    let mut key_inputs = configs.clone();
    key_inputs.extend(declarations(config, &mut out)?);
    key_inputs.extend(md_facts);
    key_inputs.push((
        "walk:assets".to_string(),
        tokens::fnv1a(roots_bytes(out.assets.iter()).as_slice()),
    ));
    key_inputs.extend(crate::graph::keys::ts_fs_facts(root, &out.live));
    out.resolve_key = store::resolve_key(&out.live, &key_inputs);
    out.configs = configs.into_iter().map(|(path, _)| path).collect();
    Ok(out)
}

/// The walk: every judged file through refresh_file, the md facts
/// gathered, and the resolver configs returned with their content
/// hashes (key inputs). Reads go through walk::read_surviving: a
/// mid-walk deletion is a skip and the next run converges (the
/// survival rule scan and graph already walk under); an unreadable
/// file that EXISTS still aborts.
fn refresh_tree(
    root: &Path,
    config: &Config,
    idx: &mut index::Index,
    out: &mut WalkIndex,
    md_facts: &mut Vec<(String, u64)>,
) -> Result<Vec<(String, u64)>> {
    let mut configs: Vec<(String, u64)> = Vec::new();
    // foreign files (a declared submodule's) enter the index too: the
    // graph needs the references they hold and the advisory the names
    // they spell — their `foreign` flag is what keeps every
    // measurement (clone pairs, docdup, score rows) off them
    for walk::Walked { path, foreign } in
        walk::collect(root, &config.exclude).map_err(anyhow::Error::msg)?
    {
        let rel = walk::rel_str(root, &path);
        if store::is_resolver_config(&path) {
            configs.extend(walk::read_surviving(&path)?.map(|b| (rel, tokens::fnv1a(&b))));
            continue;
        }
        // judged_path: the scan-only arm (plan v2.5) never enters the
        // index — it feeds only judgment surfaces; every other walked
        // file is an asset a page may name (WalkIndex::assets)
        let Some(lang) = Lang::judged_path(&path) else {
            out.assets.insert(rel);
            continue;
        };
        let Some(src) = walk::read_surviving(&path)? else {
            continue; // vanished mid-walk: not live this pass
        };
        lang_fact(lang, &rel, &src, md_facts, out);
        if idx.refresh_file(&rel, &src, lang, Params::default(), foreign)? {
            out.dirty.insert(rel.clone());
        }
        if lang.fingerprints() {
            out.tokenized += 1;
        }
        out.live.insert(rel);
    }
    Ok(configs)
}

/// The two `[graph]` declarations the resolver reads, seated on the
/// walk index and returned as key inputs: they are resolver INPUTS
/// like the manifests the walk collected, hashed into the key so that
/// editing a declaration re-fires the sweep. A crate root the walk did
/// not see, or one that is no Rust file, and a search root holding no
/// walked file, are each refused by name (the [structure] layout
/// posture): silently dropping one would put the tree back in the
/// false-dead shape the knob exists to end.
fn declarations(config: &Config, out: &mut WalkIndex) -> Result<Vec<(String, u64)>> {
    out.crate_roots = config.graph.declared_roots();
    for r in &out.crate_roots {
        anyhow::ensure!(
            r.ends_with(".rs") && out.live.contains(r),
            "[graph] crate_roots declares {r:?}, which is not a walked Rust file"
        );
    }
    out.search_roots = config.graph.declared_search_roots();
    let empty = out.search_roots.iter().find_map(|(lang, dirs)| {
        let prefix = |d: &str| {
            if d.is_empty() {
                String::new()
            } else {
                format!("{d}/")
            }
        };
        dirs.iter()
            .find(|d| !out.live.iter().any(|f| f.starts_with(&prefix(d))))
            .map(|d| (lang.clone(), d.clone()))
    });
    if let Some((lang, dir)) = empty {
        anyhow::bail!("[graph.search_roots] {lang} declares {dir:?}, which holds no walked file");
    }
    let search: Vec<String> = out
        .search_roots
        .iter()
        .flat_map(|(lang, dirs)| dirs.iter().map(move |d| format!("{lang}={d}")))
        .collect();
    Ok(vec![
        (
            "ce.toml#graph.crate_roots".to_string(),
            tokens::fnv1a(roots_bytes(out.crate_roots.iter()).as_slice()),
        ),
        (
            "ce.toml#graph.search_roots".to_string(),
            tokens::fnv1a(roots_bytes(search.iter()).as_slice()),
        ),
    ])
}

/// A declared set as one byte string for the key (NUL-joined: a
/// separator no path carries).
fn roots_bytes<'r>(roots: impl Iterator<Item = &'r String>) -> Vec<u8> {
    roots
        .flat_map(|r| r.bytes().chain(std::iter::once(0)))
        .collect()
}

/// Cross-file resolver INPUTS per language (split from index_all at
/// the E01 fn gate): md heading slugs, a page's id set, the Rust
/// pub-use surface, a Java file's declared package and the templates a
/// Lua file assigns to `package.path` all join the resolve_key — the
/// same discipline, one throat. A Java header's imports stay out of the key: they steer
/// only the file's own sites, which a change to them re-resolves as
/// the file's refresh does. A Lua file without templates adds no key
/// input, so a tree without them keys as it did before step 4.
fn lang_fact(
    lang: Lang,
    rel: &str,
    src: &[u8],
    facts: &mut Vec<(String, u64)>,
    out: &mut WalkIndex,
) {
    let text = String::from_utf8_lossy(src);
    match lang {
        Lang::Markdown => facts.push((rel.to_string(), crate::graph::ladder::md::slug_hash(&text))),
        Lang::Html => facts.push((
            rel.to_string(),
            crate::graph::ladder::html_head::id_hash(&text),
        )),
        Lang::Rust => facts.push((
            rel.to_string(),
            crate::graph::ladder::rs_reexport::pubuse_hash(&text),
        )),
        Lang::Java => {
            let header = java_header::read(&text);
            facts.push((rel.to_string(), tokens::fnv1a(header.package.as_bytes())));
            out.java.insert(rel.to_string(), header);
        }
        Lang::Lua => {
            let templates = lua_path::read(&text);
            if !templates.is_empty() {
                let bytes: Vec<u8> = templates
                    .iter()
                    .flat_map(|t| [t.dir.as_bytes(), b"\0", t.suffix.as_bytes(), b"\0"].concat())
                    .collect();
                facts.push((rel.to_string(), tokens::fnv1a(&bytes)));
                out.lua.extend(templates);
            }
        }
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
        // the re-feed keeps the owner the walk stamped: candidates
        // are own files by construction (all_instances), and the flag
        // is the index's fact, not this pass's to re-decide
        let foreign = idx.is_foreign(rel)?;
        if idx.refresh_file(rel, &src, lang, p, foreign)? {
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

/// rel → absolute path + language; None for non-lang paths AND for
/// the scan-only arm (the shared gate of this walk and the probe's
/// candidate loop — index files are judged by construction, so the
/// judged_path form here is defense in depth, not a behavior change).
pub(super) fn lang_path(root: &Path, rel: &str) -> Option<(std::path::PathBuf, Lang)> {
    let path = root.join(rel);
    let lang = Lang::judged_path(&path)?;
    Some((path, lang))
}
