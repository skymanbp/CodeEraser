//! The language and product definitions (plan v2.32 steps 1–2; design
//! booklet docs/reference/authority-track.md §4): the core holds every
//! table this side measures with — each judged language's scan, flow,
//! slot, site and call tables, the language rows, the walk's excludes,
//! the ladders' name tables, the prose vocabularies — and answers them
//! as one package, `tables/1` (proto 7.7.0). This side holds none of
//! them: every reader asks `get()`.
//!
//! Three sources, in order: this process's memory, the cache file
//! `<root>/.ce/tables-<ce>-<proto>.json` (cache.rs), the core. The
//! cache is a copy of one core's answer, kept beside the identity of
//! the core binary that gave it; a stale or damaged copy is fetched
//! again, never refused. No core, or a core older than 7.7.0, refuses
//! the run by name: there is no built-in copy to fall back on (booklet
//! §13 item 7), so a measurement can never run on tables the core
//! does not hold.

mod cache;
mod fetch;
pub mod leak;
mod pack;

pub use pack::{
    ByLang, Calls, Compdb, Docdup, Flags, Fourclass, Keys, Ladder, LangRow, Languages, Pack,
    Protocol, Tables, Tombstone, Walk,
};

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// The project whose `.ce/` holds the cache: set by each entry where it
/// resolves its root (the CLI's `or_cwd`, the hook gate, the daemon,
/// the GUI's task body); the first one set wins.
static ANCHOR: OnceLock<PathBuf> = OnceLock::new();

/// The package, or why there is none — read once per process.
static PACK: OnceLock<Result<Pack, String>> = OnceLock::new();

/// Set by `transient`: read the package, never write the cache.
static TRANSIENT: AtomicBool = AtomicBool::new(false);

/// Read the package without writing the cache file: `ce eject`, whose
/// work is to remove `.ce/`, must not create it (its stray walk reads
/// the build-output table).
pub fn transient() {
    TRANSIENT.store(true, Ordering::Relaxed);
}

/// Name the project whose `.ce/` caches the package. A process that
/// never names one caches under the project around its working
/// directory.
pub fn anchor(root: &Path) {
    let _ = ANCHOR.set(root.to_path_buf());
}

/// The package, read on first use; Err = the named reason there is
/// none (no core, a pre-7.7.0 core, a package that does not read). The
/// hook gate and the GUI ask this one, to stay inert or to show the
/// reason; everything else asks `get`.
pub fn load() -> Result<&'static Pack, &'static str> {
    PACK.get_or_init(|| {
        let root = match ANCHOR.get() {
            Some(root) => root.clone(),
            None => crate::root::project_root(Path::new(".")),
        };
        fetch::load(&root, !TRANSIENT.load(Ordering::Relaxed))
    })
    .as_ref()
    .map_err(String::as_str)
}

/// The package. A process without one cannot measure anything, so the
/// run is refused here, by name, exit 2 — the CLI's refusal code.
pub fn get() -> &'static Pack {
    match load() {
        Ok(pack) => pack,
        Err(why) => {
            eprintln!("ce: {why}");
            std::process::exit(2)
        }
    }
}

/// A core's hello against the package in memory: the digest it names
/// must be the one the package was read with. Unequal means the cache
/// check missed a change of core — a defect, named with both numbers
/// and the cache file, never read past. Nothing to compare before the
/// package is read, or with a hello that names no digest.
pub(crate) fn check_hello(core: &str, digest: Option<u64>) -> Result<(), String> {
    match (PACK.get(), digest) {
        (Some(Ok(pack)), Some(named)) if named != pack.digest => Err(format!(
            "core {core} names tables digest {named}, but this run read {} from {}",
            pack.digest,
            pack.source.display()
        )),
        _ => Ok(()),
    }
}
