//! File discovery with the exclusion model (plan §4.1):
//! .gitignore (via `ignore` crate) + .ceignore + built-in category
//! defaults + ce.toml globs. Declarative only.

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::path::{Path, PathBuf};

/// Built-in excludes: lockfiles, minified/generated, vendored,
/// snapshots, migrations (plan §4.1 category list) — plus the secret
/// globs plan §5.9-2 promises (.env / *.pem / id_* / *.key): privacy
/// fails safe, so id_* over-matching a code file costs coverage,
/// never leaks a key into the index.
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
    "!.env",
    "!*.pem",
    "!id_*",
    "!*.key",
];

/// Config plus language-tagged candidate files — the shared opening
/// of every whole-tree analyzer (scan metrics, graph sites); the
/// config rides along so callers needing thresholds do not load it
/// twice.
pub fn scoped_lang_files(
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

/// Collect candidate files under `root`, honoring the exclusion model.
pub fn collect(root: &Path, extra_excludes: &[String]) -> Result<Vec<PathBuf>, String> {
    let overrides = build_overrides(root, extra_excludes)?;
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .add_custom_ignore_filename(".ceignore")
        .overrides(overrides)
        .hidden(true)
        .build();
    for entry in walker {
        let entry = entry.map_err(|e| format!("walk: {e}"))?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

/// Scope test for one file (existing or about to be created),
/// sharing collect()'s override set so the two cannot drift.
/// Known boundary: nested .gitignore/.ceignore files are honored by
/// the walk but not here — only the root-level ones apply, because a
/// not-yet-created file cannot be found by walking. For the guard's
/// budget rule that costs at most a spurious ask inside a
/// nested-ignored directory, never a silently skipped in-scope file.
pub fn in_scope(root: &Path, path: &Path, extra_excludes: &[String]) -> bool {
    let root = canon(root);
    let Ok(rel) = canon(path).strip_prefix(&root).map(Path::to_path_buf) else {
        return false; // outside the project root: not ours to judge
    };
    let Ok(overrides) = build_overrides(&root, extra_excludes) else {
        return false;
    };
    // Directory patterns ("!vendor/", "gen/") match the DIRECTORY, so
    // the file itself and every ancestor must be tested — the walk
    // prunes at the directory, a single direct-path probe does not
    // (attack review 2026-08-11 F10).
    let excluded = std::iter::successors(Some(rel.as_path()), |p| p.parent()).any(|p| {
        !p.as_os_str().is_empty()
            && matches!(overrides.matched(p, p != rel), ignore::Match::Ignore(_))
    });
    if excluded {
        return false;
    }
    !ignored_by(&root, ".gitignore", &rel) && !ignored_by(&root, ".ceignore", &rel)
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

fn ignored_by(root: &Path, ignore_file: &str, rel: &Path) -> bool {
    let mut b = ignore::gitignore::GitignoreBuilder::new(root);
    b.add(root.join(ignore_file));
    b.build()
        // parent-aware on purpose: "generated/" must cover its files
        .map(|g| g.matched_path_or_any_parents(rel, false).is_ignore())
        .unwrap_or(false)
}

fn build_overrides(root: &Path, extra: &[String]) -> Result<ignore::overrides::Override, String> {
    let mut builder = OverrideBuilder::new(root);
    for glob in BUILTIN_EXCLUDES {
        builder
            .add(glob)
            .map_err(|e| format!("builtin glob {glob}: {e}"))?;
    }
    for glob in extra {
        // ce.toml lists positive globs to exclude; Override semantics
        // need the leading '!' for exclusion. A user-written '!' would
        // double-negate into a silent no-op — reject it loudly.
        if glob.starts_with('!') {
            return Err(format!(
                "ce.toml exclude {glob}: write the glob without '!' (entries are exclusions already)"
            ));
        }
        // '\' is an ESCAPE here and candidates are '/'-spelled
        // (rel_str), so a Windows-written `src\generated\*.rs`
        // compiled to a pattern matching NOTHING — silent no-op on
        // this project's primary platform. Normalized, not refused
        // like the '!' above: '/' is the one canonical spelling here
        // and the separator reading is what the author meant.
        let g = format!("!{}", glob.replace('\\', "/"));
        builder
            .add(&g)
            .map_err(|e| format!("ce.toml exclude {glob}: {e}"))?;
    }
    builder.build().map_err(|e| format!("excludes: {e}"))
}
