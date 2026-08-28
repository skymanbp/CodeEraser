//! File discovery with the exclusion model (plan §4.1):
//! .gitignore (via `ignore` crate) + .ceignore + built-in category
//! defaults + ce.toml globs. Declarative only.

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
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
/// of every whole-tree analyzer (scan metrics, graph sites); the
/// config rides along so callers needing thresholds do not load it
/// twice.
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
        .filter_map(|p| crate::scan::lang::Lang::from_path(&p).map(|l| (p, l)))
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

/// Collect candidate files under `root`, honoring the exclusion model.
/// A declared submodule the checkout has not seated (gitmodules.rs)
/// refuses BY NAME first: its files are tree content a filesystem
/// walk cannot see, so the walk would measure a tree missing them and
/// `ce baseline` would persist the shrunken ratchet as an improvement
/// — "a gate that could not judge must never pass" (ADR-008 P1). A
/// declared path the exclusion model prunes anyway (`vendor/`) judges
/// nothing seated or not, so it is not refused. `ce guard` never
/// comes through here: a fail-open hook must not start refusing every
/// write on a shallow clone.
pub fn collect(root: &Path, extra_excludes: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut scope = Scope::new(root, extra_excludes)?;
    for rel in crate::gitmodules::unseated(root) {
        if scope.contains_dir(&rel) {
            return Err(crate::gitmodules::refusal(&rel, root));
        }
    }
    let mut files = Vec::new();
    for entry in builder(root, extra_excludes)?.build() {
        let entry = entry.map_err(|e| format!("walk: {e}"))?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
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
/// Built once per batch: the matcher caches per directory.
pub struct Scope {
    root: PathBuf,
    matcher: ignore::IncrementalIgnore,
}

impl Scope {
    pub fn new(root: &Path, extra_excludes: &[String]) -> Result<Self, String> {
        let root = canon(root);
        let matcher = builder(&root, extra_excludes)?
            .build_matchers()
            .pop()
            .ok_or_else(|| "walk: no matcher for the root".to_string())?;
        Ok(Self { root, matcher })
    }

    /// `path` (absolute or root-relative) is one the walk would yield.
    pub fn contains(&mut self, path: &Path) -> bool {
        let full = canon(&self.root.join(path));
        let Ok(rel) = full.strip_prefix(&self.root) else {
            return false; // outside the project root: not ours to judge
        };
        !rel.as_os_str().is_empty() && !self.matcher.matched(rel, false).is_ignore()
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
        add_user_glob(&mut builder, glob, true, "exclude")?;
    }
    builder.build().map_err(|e| format!("excludes: {e}"))
}

/// One user-written glob into an override set — the two rules every
/// ce.toml glob reader shares (exclude, and the rulepack's classes in
/// scan::classes; a second copy is what the dedup gate refuses):
/// a leading '!' is refused, because an exclude entry is an exclusion
/// already and a class entry an inclusion already, so a user '!'
/// would double-negate into a silent no-op; and '\' normalizes to
/// '/', because '\' is an ESCAPE in this syntax while candidates are
/// '/'-spelled (rel_str) — a Windows-written `src\generated\*.rs`
/// compiled to a pattern matching NOTHING on this project's primary
/// platform. Normalized, not refused: the separator reading is what
/// the author meant. `exclude` selects the direction.
pub(crate) fn add_user_glob(
    builder: &mut OverrideBuilder,
    glob: &str,
    exclude: bool,
    what: &str,
) -> Result<(), String> {
    if glob.starts_with('!') {
        let already = if exclude { "exclusions" } else { "inclusions" };
        return Err(format!(
            "ce.toml {what} {glob}: write the glob without '!' (entries are {already} already)"
        ));
    }
    let g = glob.replace('\\', "/");
    let g = if exclude { format!("!{g}") } else { g };
    builder
        .add(&g)
        .map(|_| ())
        .map_err(|e| format!("ce.toml {what} {glob}: {e}"))
}
