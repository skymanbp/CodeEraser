//! The PostToolUse re-measure (v2.35 guard lane, GD7). The write-time
//! probe never refreshes (dedup/probe.rs) and nothing refreshed at
//! PostToolUse, so between two Stops a file the session just CREATED
//! was not in the index at all and one it just edited sat there in its
//! pre-edit form: a copy of code written minutes ago passed at write
//! time. Here the one file that landed is refreshed by the same
//! per-file refresh a whole run performs (`Index::refresh_file`, one
//! transaction; the edge-debt ledger makes a partial refresh safe), so
//! the next probe meets it through the normal lookup.
//!
//! Only when the file is in scope, the project's own (not a declared
//! submodule's), of a fingerprinting language, and the index exists,
//! is built and is of THIS generation: it never builds or rebuilds an
//! index (a stale key would wipe it under `Index::open`; `peek` is
//! asked first and a foreign generation is left alone).

use super::super::coldstart;
use super::super::proto::Response;
use crate::dedup::{Params, index};
use crate::gitmodules::{self, Owner};
use crate::scan::lang::Lang;
use crate::scan::walk;
use anyhow::Result;
use std::path::Path;

pub(super) fn reply(root: &Path, file_path: &str) -> Response {
    let report = match reindex(root, file_path) {
        Ok(None) => serde_json::json!({"refreshed": true}),
        Ok(Some("current")) => serde_json::json!({"refreshed": false}),
        Ok(Some(why)) => serde_json::json!({"skipped": why}),
        Err(e) => serde_json::json!({"degraded": format!("{e:#}")}),
    };
    Response::ReindexReport { report }
}

/// Ok(None) = refreshed; Ok(Some(why)) = nothing to do, and why.
fn reindex(root: &Path, file_path: &str) -> Result<Option<&'static str>> {
    let Some(path) = walk::contained(root, file_path) else {
        return Ok(Some("outside"));
    };
    let Some(lang) = Lang::from_path(&path).filter(|l| l.fingerprints()) else {
        return Ok(Some("language"));
    };
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    if !walk::in_scope(root, &path, &cfg.exclude) {
        return Ok(Some("scope"));
    }
    // the probe's own spelling of the key (replies.rs: canonical, then
    // the one rel_str throat)
    let canon = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    let rel = walk::rel_str(root, &canon);
    if !matches!(gitmodules::owner(root, &gitmodules::declared(root), &rel), Owner::Own) {
        return Ok(Some("foreign"));
    }
    let (db, p) = (root.join(crate::dedup::INDEX_REL), Params::default());
    if !db.is_file() {
        return Ok(Some("no_index"));
    }
    // the generation first: `coldstart::index_ready` reads the stamp
    // through `Index::open`, which rebuilds a key it finds stale
    if !index::peek(&db, p)?.1 {
        return Ok(Some("generation"));
    }
    if !coldstart::index_ready(root) {
        return Ok(Some("cold"));
    }
    let Some(src) = walk::read_surviving(&path)? else {
        return Ok(Some("gone"));
    };
    let mut idx = index::Index::open(&db, p)?;
    Ok((!idx.refresh_file(&rel, &src, lang, p, false)?).then_some("current"))
}
