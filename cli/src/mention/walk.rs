//! §1 of the sealed criterion: the corpus universe U — a SECOND walk,
//! independent of the scan's, because the two have opposite polarity:
//! the scan's exclusion model serves "what is measured", the veto
//! needs "what could reference a name". Every parameter is explicit,
//! frozen, and a `MENTION_REV` input (mod.rs):
//!   - hidden files walk (`.github/` mentions count); `.git/` and
//!     `.ce/` are cut by NAME (the observe feed would otherwise enter);
//!   - a NESTED repository — any directory below the root holding a
//!     real `.git` anchor (root.rs: the directory, or a gitfile whose
//!     pointer resolves) — is cut whole: it belongs to
//!     its own U, and the walker would otherwise let the outer
//!     `.gitignore` reach into it, which git never does. A path the
//!     root's `.gitmodules` declares is the one exception: a submodule
//!     checkout owns a `.git` file too, but it is part of THIS tree by
//!     declaration (the test suite rides at `cli/tests` that way since
//!     plan v2.18; gitmodules.rs reads the declaration with git's own
//!     grammar), and the declaration is tracked content, so one commit
//!     still yields one U: a declared checkout that is not seated
//!     REFUSES by name rather than letting U shrink to the walk;
//!   - `.gitignore` and `.ceignore` are honoured and nothing else: not
//!     `.ignore` (the walker's own default, off here), not the global,
//!     exclude or parent ignore files; `.git` is not required, so one
//!     commit yields one U on any machine, with or without its `.git`;
//!   - directory symlinks are not followed (a followed link escapes
//!     the root, an ancestor cycle aborts the walk); a FILE symlink is
//!     read through when its target is a regular file inside the
//!     root, and a target already walked is skipped silently. The
//!     identity is the canonical path, never a retained
//!     `same_file::Handle`: a handle owns an open descriptor, and |U|
//!     of them outrun a default NOFILE (the self corpus alone is 593),
//!     at which point files would leave U as "errors";
//!   - files above 4 MiB are skipped and COUNTED in this loop (the
//!     walker's own size filter is silent, before any filter);
//!   - walk errors (cycles, I/O, unreadable metadata) are skipped and
//!     counted; a duplicate target or a link to a non-file is not an
//!     error and touches no counter;
//!   - the universe's own exclusion table: the shared secret globs
//!     (scan/walk.rs — one table of NAMES for both walks; the scan's
//!     override set also prunes a matching directory such as a `.env/`
//!     virtualenv, while U tests file basenames only and walks such a
//!     directory, storing hashes) plus the omni-mentioners that name
//!     everything (`*.map`, `tags`, `TAGS`, `*.po`). Generated and
//!     vendored trees are NOT excluded (user ruling ③): they are in U
//!     and outside the judged domain.
//!
//! The binary rule lives here too: a UTF-16 BOM decodes; otherwise a
//! NUL in the first 8000 bytes (git's rule) skips the file — a late
//! NUL stays in U, exactly as `contracts/VERSIONING.md` does.

use crate::gitmodules::Owner;
use crate::scan::walk::{SECRET_GLOBS, contained, rel_str};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// The single-file ceiling, judged on metadata in the loop below
/// (published through the module root for the self-corpus pin).
pub const FILE_CAP: u64 = 4 * 1024 * 1024;

/// Files whose whole purpose is to name every symbol (measured list;
/// minified bundles keep their export names and are exactly the
/// population this instrument judges, so they stay in).
const OMNI_MENTIONERS: [&str; 4] = ["*.map", "tags", "TAGS", "*.po"];

/// The walked universe: `(absolute, repo-relative)` per file in
/// relative-path order, plus the two counters only this loop can take.
pub(super) struct Universe {
    pub files: Vec<(PathBuf, String)>,
    pub oversize: usize,
    pub errors: usize,
}

/// One candidate's fate: admitted, over the cap, silently skipped (a
/// target already walked, or a link into the exclusion table), or
/// unreadable (counted with the walk's own errors).
enum Admit {
    File,
    Oversize,
    Skip,
    Unreadable,
}

pub(super) fn universe(root: &Path) -> Result<Universe> {
    if let Some(rel) = crate::gitmodules::unseated(root).first() {
        anyhow::bail!("mentions: {}", crate::gitmodules::refusal(rel, root));
    }
    let mut gate = Gate::open(root)?;
    let (home, declared) = (root.to_path_buf(), crate::gitmodules::declared(root));
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .require_git(false)
        .follow_links(false)
        .add_custom_ignore_filename(".ceignore")
        .filter_entry(move |e| e.depth() == 0 || !is_cut(&home, &declared, e))
        .build();
    let mut out = Universe {
        files: Vec::new(),
        oversize: 0,
        errors: 0,
    };
    let mut candidates = Vec::new();
    for entry in walker {
        let Ok(entry) = entry else {
            out.errors += 1;
            continue;
        };
        let rel = rel_str(root, entry.path());
        if is_file_like(root, &entry, &rel) && !gate.excluded.is_match(basename(&rel)) {
            candidates.push((entry.path_is_symlink(), entry.into_path(), rel));
        }
    }
    // relative-path order BEFORE the identity pass: when a link and its
    // target (or two links) share one file, the lexicographically first
    // path is the one kept on every machine, not whichever the
    // directory read happened to yield
    candidates.sort_by(|a, b| a.2.cmp(&b.2));
    for (link, path, rel) in candidates {
        match gate.admit(&path, &rel, link) {
            Admit::File => out.files.push((path, rel)),
            Admit::Oversize => out.oversize += 1,
            Admit::Skip => {}
            Admit::Unreadable => out.errors += 1,
        }
    }
    Ok(out)
}

/// The names cut by name: the repository's own metadata and the
/// product's own state (the observe feed would otherwise enter U).
const CUT_NAMES: [&str; 2] = [".git", ".ce"];

/// The walker's entry filter: a cut name, or a directory that is a
/// repository of its own and not a declared submodule — the owner
/// rule the measurement walk reads too (gitmodules::owner: a declared
/// prefix is Foreign and stays in U, an undeclared `.git` anchor is
/// Cut). Its parents were filtered before it, so the entry alone is
/// asked.
fn is_cut(root: &Path, declared: &BTreeSet<String>, e: &DirEntry) -> bool {
    let name = e.file_name().to_string_lossy();
    CUT_NAMES.contains(&name.as_ref())
        || (e.file_type().is_some_and(|t| t.is_dir())
            && crate::gitmodules::owner(root, declared, &rel_str(root, e.path())) == Owner::Cut)
}

/// The same cut read off a path: `rel` has a cut-name component, or
/// one of its prefixes — the entry itself included, since git lists
/// a gitlink as the bare path `sub` and a nested repository as
/// `sub/` — is a repository of its own and not a declared submodule.
/// Published for the census (census.rs) and the K23 formula
/// (tests/it/mention_universe.rs), so the walk's rule has one
/// implementation.
pub fn cut(root: &Path, rel: &str) -> bool {
    rel.split('/').any(|seg| CUT_NAMES.contains(&seg))
        || crate::gitmodules::owner(root, &crate::gitmodules::declared(root), rel) == Owner::Cut
}

/// A regular file, or a symlink whose target is a regular file inside
/// the root (`walk::contained` canonicalizes through the link — the
/// ONE containment authority; a directory link is not file-like and
/// is simply not a candidate).
fn is_file_like(root: &Path, e: &DirEntry, rel: &str) -> bool {
    if e.path_is_symlink() {
        return std::fs::metadata(e.path()).is_ok_and(|m| m.is_file())
            && contained(root, rel).is_some();
    }
    e.file_type().is_some_and(|t| t.is_file())
}

/// The identity pass: the exclusion table and the set of files already
/// walked, keyed by path relative to the canonical root. A regular
/// file keys under its own relative path; a link keys under its
/// target's — the same string the target itself would use, so
/// whichever of the two sorts first is kept and the other is a
/// duplicate. No descriptor stays open between two candidates.
struct Gate {
    canon_root: PathBuf,
    excluded: &'static GlobSet,
    walked: HashSet<String>,
}

impl Gate {
    fn open(root: &Path) -> Result<Gate> {
        Ok(Gate {
            canon_root: std::fs::canonicalize(root)
                .with_context(|| format!("canonicalize {}", root.display()))?,
            excluded: &EXCLUDED,
            walked: HashSet::new(),
        })
    }

    /// Size and identity, both through the link.
    fn admit(&mut self, path: &Path, rel: &str, link: bool) -> Admit {
        let Ok(meta) = std::fs::metadata(path) else {
            return Admit::Unreadable;
        };
        if meta.len() > FILE_CAP {
            return Admit::Oversize;
        }
        let key = if link {
            let Ok(target) = std::fs::canonicalize(path) else {
                return Admit::Unreadable;
            };
            let key = rel_str(&self.canon_root, &target);
            if self.excluded.is_match(basename(&key)) {
                return Admit::Skip; // a link must not read an excluded file through
            }
            key
        } else {
            rel.to_string()
        };
        if self.walked.insert(key) {
            Admit::File
        } else {
            Admit::Skip
        }
    }
}

fn basename(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// The universe's own exclusion table as one basename matcher — the
/// same glob engine the walker uses, so `.env*` means here what it
/// means in the scan's override set. Built once: the two tables are
/// literals, so a build failure is a programming error, not a run.
static EXCLUDED: LazyLock<GlobSet> = LazyLock::new(|| {
    let mut b = GlobSetBuilder::new();
    for glob in SECRET_GLOBS.iter().chain(OMNI_MENTIONERS.iter()) {
        b.add(Glob::new(glob).expect(glob));
    }
    b.build().expect("mention exclusion table")
});

/// Whether the exclusion table leaves `rel` out of U — the third of
/// the walk's published rules (with `FILE_CAP` and `decode`), so the
/// K23 pins subtract exactly what the walk subtracts.
pub fn excluded(rel: &str) -> bool {
    EXCLUDED.is_match(basename(rel))
}

/// The universe's text of one file, or None for a binary (skipped and
/// counted by the caller): a UTF-16 BOM decodes, otherwise a NUL in
/// the first 8000 bytes is git's binary verdict. Lossy on purpose —
/// one stray byte must not lose a file's mentions. Published through
/// the module root for the same pin as `FILE_CAP`.
pub fn decode(bytes: &[u8]) -> Option<String> {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => Some(utf16(rest, u16::from_le_bytes)),
        [0xFE, 0xFF, rest @ ..] => Some(utf16(rest, u16::from_be_bytes)),
        _ if early_nul(bytes) => None,
        _ => Some(String::from_utf8_lossy(bytes).into_owned()),
    }
}

/// git's rule.
fn early_nul(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(8000)].contains(&0)
}

fn utf16(rest: &[u8], unit: fn([u8; 2]) -> u16) -> String {
    let units: Vec<u16> = rest.chunks_exact(2).map(|c| unit([c[0], c[1]])).collect();
    String::from_utf16_lossy(&units)
}
