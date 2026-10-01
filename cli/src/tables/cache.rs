//! The package's cache file, `<root>/.ce/tables-<ce>-<proto>.json`: one
//! JSON object holding the package text as the core's answer gave it
//! (`pack`), the core's digest of it, this side's own fnv1a64 of the
//! text (`bytes_fnv1a64`), and the identity of the core binary that
//! answered — its path, length and modification time, one stat. A copy
//! is used only while all of that still holds: a rebuilt core, a second
//! core sharing the `.ce/`, a damaged file each read as "fetch again",
//! never as a refusal. Writing is best effort: the package is in memory
//! already, and a GUI started in a read-only directory still measures.

use super::pack::Pack;
use crate::dedup::tokens::fnv1a;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::path::{Path, PathBuf};

/// The core binary a package came from, as one stat sees it.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(super) struct Identity {
    path: String,
    len: u64,
    mtime_ns: u64,
}

impl Identity {
    /// Stat the core; None = it cannot be read (the caller refuses).
    pub(super) fn of(core: &Path) -> Option<Identity> {
        let meta = std::fs::metadata(core).ok()?;
        let mtime = meta.modified().ok()?;
        let since = mtime.duration_since(std::time::UNIX_EPOCH).ok()?;
        Some(Identity {
            path: core.display().to_string(),
            len: meta.len(),
            mtime_ns: u64::try_from(since.as_nanos()).ok()?,
        })
    }
}

/// The file as read: every field but the package text is checked.
#[derive(Deserialize)]
struct File<'a> {
    ce: String,
    proto: String,
    core: Identity,
    digest: u64,
    bytes_fnv1a64: u64,
    #[serde(borrow)]
    pack: &'a RawValue,
}

/// The cache file under `root`.
pub(super) fn path(root: &Path) -> PathBuf {
    let name = format!(
        "tables-{}-{}.json",
        env!("CARGO_PKG_VERSION"),
        crate::corelink::PROTO
    );
    root.join(".ce").join(name)
}

/// The cached package, when the file is this build's, from this core,
/// whole and readable; None = fetch it again.
pub(super) fn read(file: &Path, core: &Identity) -> Option<Pack> {
    let text = std::fs::read_to_string(file).ok()?;
    let f: File = serde_json::from_str(&text).ok()?;
    let ours = f.ce == env!("CARGO_PKG_VERSION") && f.proto == crate::corelink::PROTO;
    let bytes = f.pack.get();
    if !ours || f.core != *core || fnv1a(bytes.as_bytes()) != f.bytes_fnv1a64 {
        return None;
    }
    Pack::read(bytes, f.digest, file.to_path_buf()).ok()
}

/// Write the package beside its identity: a temporary name, then a
/// rename, so a reader sees a whole file or none. Another process that
/// wrote first wrote the same bytes; any failure leaves the run as it
/// is.
pub(super) fn write(file: &Path, core: &Identity, digest: u64, pack: &str) {
    let Some(dir) = file.parent() else {
        return;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let head = serde_json::json!({
        "ce": env!("CARGO_PKG_VERSION"),
        "proto": crate::corelink::PROTO,
        "core": core,
        "digest": digest,
        "bytes_fnv1a64": fnv1a(pack.as_bytes()),
    })
    .to_string();
    // the package text goes in as written; the head's own closing brace
    // makes room for it
    let body = format!("{},\"pack\":{pack}}}", &head[..head.len() - 1]);
    let tmp = file.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, body).is_err() || std::fs::rename(&tmp, file).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}
