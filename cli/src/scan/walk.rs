//! File discovery with the exclusion model (plan §4.1):
//! .gitignore (via `ignore` crate) + .ceignore + built-in category
//! defaults + ce.toml globs. Declarative only. Since plan v2.18 step
//! #12 the walk also answers WHOSE each file is (gitmodules::owner):
//! a declared submodule's files come back tagged `foreign` — read by
//! the index for their references and mentions, measured by nobody
//! here — and an undeclared nested repository is pruned before it is
//! walked at all.

use crate::gitmodules::Owner;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The secret-file globs BOTH walks refuse — this measurement walk and
/// the mention universe (mention/walk.rs) — one table of NAMES, so the
/// privacy invariant plan §5.9-2 promises is spelled once (widened at
/// plan v2.17 L round, S-A9). The reach differs by design: here the
/// override set also prunes a matching DIRECTORY (a `.env/`
/// virtualenv), while the mention walk tests file basenames only.
/// Privacy fails safe: `id_*` or `*credentials*` over-matching a code
/// file costs coverage, never leaks a key into the index.
pub(crate) const SECRET_GLOBS: [&str; 8] = [
    ".env*",
    "*.pem",
    "*.key",
    "id_*",
    ".npmrc",
    ".pypirc",
    ".netrc",
    "*credentials*",
];

/// Built-in excludes: lockfiles, minified/generated, vendored,
/// snapshots, migrations (plan §4.1 category list); the secret globs
/// above join them in build_overrides.
const BUILTIN_EXCLUDES: &[&str] = &[
    "!package-lock.json",
    "!yarn.lock",
    "!pnpm-lock.yaml",
    "!Cargo.lock",
    "!*.min.js",
    "!*.min.css",
    "!*.pb.go",
    "!*_pb2.py",
    "!*.generated.*",
    "!vendor/",
    "!node_modules/",
    "!__snapshots__/",
    "!*.snap",
    "!migrations/",
    "!dist/",
    "!build/",
    "!target/",
    "!dist-newstyle/",
];

/// Config plus language-tagged candidate files — the shared opening
/// of every whole-tree MEASUREMENT (scan metrics, graph sites); the
/// config rides along so callers needing thresholds do not load it
/// twice. Own files only: a foreign file is measured by its own tree.
fn scoped_lang_files(
    root: &Path,
) -> Result<
    (
        crate::config::Config,
        Vec<(PathBuf, crate::scan::lang::Lang)>,
    ),
    String,
> {
    let config = crate::config::Config::load(root)?;
    let files = collect(root, &config.exclude)?
        .into_iter()
        .filter(|w| !w.foreign)
        .filter_map(|w| crate::scan::lang::Lang::from_path(&w.path).map(|l| (w.path, l)))
        .collect();
    Ok((config, files))
}

/// Repo-root-relative path in the report's canonical spelling
/// (forward slashes) — the ONE form file identities are keyed and
/// fingerprinted under; scan reports and baseline entities must
/// never disagree on it.
pub fn rel_str(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

/// Read a walked file, degrading a MID-WALK DELETION to None — one
/// vanished file must not abort the whole run (the probe.rs stale
/// precedent; M5-close review LOW, shared by scan and graph). Every
/// other error still surfaces: an unreadable file that EXISTS is a
/// real defect, not a race.
pub fn read_surviving(path: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    use anyhow::Context;
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(_) if !path.exists() => Ok(None),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

/// Walk + read with the survival rule in ONE place: the per-file
/// judgment stays with the caller, the loop shape lives here (scan
/// and graph carried twin loops until the ratchet bit the pair).
pub fn each_surviving<T>(
    root: &Path,
    mut per_file: impl FnMut(&Path, crate::scan::lang::Lang, Vec<u8>) -> anyhow::Result<T>,
) -> anyhow::Result<(crate::config::Config, Vec<T>)> {
    let (config, candidates) = scoped_lang_files(root).map_err(anyhow::Error::msg)?;
    let mut out = Vec::new();
    for (path, lang) in candidates {
        let Some(bytes) = read_surviving(&path)? else {
            continue;
        };
        out.push(per_file(&path, lang, bytes)?);
    }
    Ok((config, out))
}

/// The walker every measurement shares — the exclusion model in ONE
/// builder, so `collect` and `Scope` cannot drift (they did: the
/// scope test hand-rolled the model from the root ignore files and
/// missed git's VCS boundary, nested ignore files and the hidden rule).
fn builder(root: &Path, extra_excludes: &[String]) -> Result<WalkBuilder, String> {
    let overrides = build_overrides(root, extra_excludes)?;
    let mut b = WalkBuilder::new(root);
    b.add_custom_ignore_filename(".ceignore")
        .overrides(overrides)
        .hidden(true);
    Ok(b)
}

/// One walked file and whose it is. `foreign` = a declared submodule's
/// (gitmodules::Owner::Foreign): the index reads it for the references
/// and mentions it holds, and no measurement counts it — its own tree
/// gates it. A nested repository nobody declared never reaches here.
pub struct Walked {
    pub path: PathBuf,
    pub foreign: bool,
}

/// Collect candidate files under `root`, honoring the exclusion model.
/// A declared submodule the checkout has not seated (gitmodules.rs)
/// refuses BY NAME first: its files are readers a filesystem walk
/// cannot see, so the graph would judge this tree's files without the
/// references those readers hold and the advisory would call their
/// mentioned names unmentioned — "a gate that could not judge must
/// never pass" (ADR-008 P1). A declared path the exclusion model
/// prunes anyway (`vendor/`) reads nothing seated or not, so it is not
/// refused. `ce guard` never comes through here: a fail-open hook must
/// not start refusing every write on a shallow clone.
pub fn collect(root: &Path, extra_excludes: &[String]) -> Result<Vec<Walked>, String> {
    let mut scope = Scope::new(root, extra_excludes)?;
    for rel in crate::gitmodules::unseated(root) {
        if scope.contains_dir(&rel) {
            return Err(crate::gitmodules::refusal(&rel, root));
        }
    }
    let mut owners = Owners::new(root);
    let (home, declared) = (root.to_path_buf(), owners.declared.clone());
    let mut files = Vec::new();
    for entry in builder(root, extra_excludes)?
        // a nested repository is pruned at its door: its files are
        // nobody's here, and walking them would only cost the reads
        .filter_entry(move |e| {
            e.depth() == 0
                || !e.file_type().is_some_and(|t| t.is_dir())
                || crate::gitmodules::owner(&home, &declared, &rel_str(&home, e.path()))
                    != Owner::Cut
        })
        .build()
    {
        let entry = entry.map_err(|e| format!("walk: {e}"))?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            let foreign = match owners.of_file(entry.path()) {
                Owner::Own => false,
                Owner::Foreign => true,
                Owner::Cut => continue, // pruned above; a race cannot re-admit it
            };
            files.push(Walked {
                path: entry.into_path(),
                foreign,
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

/// The owner rule memoized per DIRECTORY: `gitmodules::owner` stats a
/// `.git` per path segment, and a file's answer is its directory's (a
/// file never owns a repository, and a declared path is a directory),
/// so one lookup per directory serves every file in it.
struct Owners {
    root: PathBuf,
    declared: BTreeSet<String>,
    memo: BTreeMap<PathBuf, Owner>,
}

impl Owners {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            declared: crate::gitmodules::declared(root),
            memo: BTreeMap::new(),
        }
    }

    fn of_file(&mut self, path: &Path) -> Owner {
        let dir = path.parent().unwrap_or(path).to_path_buf();
        if let Some(&o) = self.memo.get(&dir) {
            return o;
        }
        let o = crate::gitmodules::owner(&self.root, &self.declared, &rel_str(&self.root, &dir));
        self.memo.insert(dir, o);
        o
    }
}

/// The exclusion model asked of ONE path (existing or about to be
/// created): the walk's own matcher, incremental (`ignore` 0.4.33
/// `build_matchers`) — glob overrides, `.gitignore`/`.ceignore` in
/// every ancestor directory, the hidden rule, and git's VCS boundary
/// (a nested `.git`, file or directory, stops the outer `.gitignore`
/// exactly as a traversal does). The hand-rolled predicate this
/// replaced read the ROOT ignore files only, so the guard judged
/// `cli/tests/…` under superproject rules the walk never applied
/// there and hidden paths the walk never yields. Ancestor DIRECTORIES
/// are read, never the file, so a not-yet-created path answers too.
/// Built once per batch: the matcher caches per directory. The owner
/// rule is asked too: a path under a declared submodule or a nested
/// repository is one the walk would never MEASURE, so the guard stays
/// inert on it — the submodule's own ce.toml (root.rs re-roots there)
/// is the tree that budgets its writes.
pub struct Scope {
    root: PathBuf,
    matcher: ignore::IncrementalIgnore,
    declared: BTreeSet<String>,
}

impl Scope {
    pub fn new(root: &Path, extra_excludes: &[String]) -> Result<Self, String> {
        let root = canon(root);
        let matcher = builder(&root, extra_excludes)?
            .build_matchers()
            .pop()
            .ok_or_else(|| "walk: no matcher for the root".to_string())?;
        let declared = crate::gitmodules::declared(&root);
        Ok(Self {
            root,
            matcher,
            declared,
        })
    }

    /// `path` (absolute or root-relative) is one the walk would yield
    /// as this project's own.
    pub fn contains(&mut self, path: &Path) -> bool {
        let full = canon(&self.root.join(path));
        let Ok(rel) = full.strip_prefix(&self.root) else {
            return false; // outside the project root: not ours to judge
        };
        !rel.as_os_str().is_empty()
            && !self.matcher.matched(rel, false).is_ignore()
            && crate::gitmodules::owner(&self.root, &self.declared, &rel_str(&self.root, &full))
                == Owner::Own
    }

    /// A root-relative DIRECTORY the walk would descend into.
    fn contains_dir(&mut self, rel: &str) -> bool {
        !self.matcher.matched(rel, true).is_ignore()
    }
}

/// One-shot `Scope` for the surfaces that ask about a single write.
pub fn in_scope(root: &Path, path: &Path, extra_excludes: &[String]) -> bool {
    Scope::new(root, extra_excludes).is_ok_and(|mut s| s.contains(path))
}

/// `root/rel` confined to `root` — None when it escapes. The ONE
/// containment authority: an ABSOLUTE `rel` replaces root outright in
/// a join and `../` walks out, which the MCP surface has checked
/// since M7 and the daemon's unauthenticated socket did not.
pub fn contained(root: &Path, rel: &str) -> Option<PathBuf> {
    let joined = root.join(rel);
    canon(&joined).starts_with(canon(root)).then_some(joined)
}

/// Canonicalize for prefix comparison; a not-yet-created file borrows
/// its parent's canonical form so drive-letter case cannot split it.
fn canon(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| match (p.parent(), p.file_name()) {
        (Some(dir), Some(name)) => canon(dir).join(name),
        _ => p.to_path_buf(),
    })
}

fn build_overrides(root: &Path, extra: &[String]) -> Result<ignore::overrides::Override, String> {
    let mut builder = OverrideBuilder::new(root);
    let secrets = SECRET_GLOBS.iter().map(|g| format!("!{g}"));
    for glob in BUILTIN_EXCLUDES
        .iter()
        .map(|g| (*g).to_string())
        .chain(secrets)
    {
        builder
            .add(&glob)
            .map_err(|e| format!("builtin glob {glob}: {e}"))?;
    }
    for glob in extra {
        super::globs::add_user_glob(&mut builder, glob, true, "exclude")?;
    }
    builder.build().map_err(|e| format!("excludes: {e}"))
}
