//! Reading the package: the cache file when it still holds, else the
//! core's `tables/1` answer, written back to the cache. The core is the
//! one this process names (corelink::resolve_core over the global
//! `--core` flag — then CE_CORE_BIN, a sibling of this executable, PATH).

use super::cache::{self, Identity};
use super::pack::Pack;
use crate::corelink::Link;
use std::path::{Path, PathBuf};

/// The package for the project at `root`, or the named refusal; a
/// fetched package is written back to the cache when `write`.
pub(super) fn load(root: &Path, write: bool) -> Result<Pack, String> {
    let resolved = crate::corelink::resolve_core(super::core_flag());
    let core = locate(&resolved).ok_or_else(|| {
        format!(
            "core unavailable: no `{resolved}` to answer tables/1 (export CE_CORE_BIN, or install \
             ce-core beside ce)"
        )
    })?;
    let id = Identity::of(&core).ok_or_else(|| {
        format!(
            "core unavailable: cannot stat {} for tables/1",
            core.display()
        )
    })?;
    let file = cache::path(root);
    if let Some(pack) = cache::read(&file, &id) {
        return Ok(pack);
    }
    let (text, digest, version) = ask(&core)?;
    let pack = Pack::read(&text, digest, file.clone()).map_err(|e| {
        format!(
            "tables/1 from core {version} ({}) lacks a table: {e}",
            core.display()
        )
    })?;
    if write {
        cache::write(&file, &id, digest, &text);
    }
    Ok(pack)
}

/// The core binary itself: a path as given, a bare name on PATH.
fn locate(core: &str) -> Option<PathBuf> {
    let given = Path::new(core);
    if given.components().count() > 1 || given.is_absolute() {
        return given
            .is_file()
            .then(|| std::path::absolute(given).ok())
            .flatten();
    }
    let exe = format!("{core}.exe");
    let names: &[&str] = if cfg!(windows) {
        &[&exe, core]
    } else {
        &[core]
    };
    crate::proc::on_path(names)
}

/// The core's answer: the package text (the reply less its envelope —
/// proto, type, id and digest), the digest, the core's version.
fn ask(core: &Path) -> Result<(String, u64, String), String> {
    let path = core.display().to_string();
    let (mut link, hello) =
        Link::open(&path).map_err(|e| format!("core unavailable: {path}: {e} (tables/1)"))?;
    if !link.has("tables/1") {
        return Err(format!(
            "pre-7.7.0 core {path} (proto {}): no tables/1",
            hello.proto
        ));
    }
    let mut reply = link
        .request("tables", serde_json::json!({}))
        .map_err(|e| format!("core {path}: tables/1: {e}"))?;
    let digest = reply["digest"]
        .as_u64()
        .ok_or_else(|| format!("tables/1 from core {} ({path}) lacks digest", hello.version))?;
    let obj = reply.as_object_mut().ok_or("tables/1 reply is no object")?;
    for key in ["proto", "type", "id", "digest"] {
        obj.remove(key);
    }
    Ok((reply.to_string(), digest, hello.version))
}
