//! Entry-flag policy for file nodes (split from deadcode.rs at the
//! 300-line dogfood wall): the mechanical conventions plus user
//! config that mark a file as a root the liveness judgment must not
//! call dead. Policy data only — the wire row that carries these
//! bits stays in deadcode.rs with the rest of the request assembly.

use crate::config::Config;

/// Mechanical entry conventions (module header); bit 0 (exported)
/// stays unset at file granularity — public-ness is a symbol fact
/// (3l re-review: Haskell module export lists included — a header's
/// exports node is symbol-level, same stance as the five launch
/// languages). Main.hs is cabal's executable main-is convention —
/// nothing imports a main module, exactly like main.rs.
pub(super) fn flags_of(path: &str, config: &Config) -> i64 {
    let base = path.rsplit('/').next().unwrap_or(path);
    let mut f = 0i64;
    if matches!(
        base,
        "main.rs" | "main.go" | "__main__.py" | "build.rs" | "Main.hs"
    ) || ["src/bin/", "examples/", "benches/", "cmd/"]
        .iter()
        .any(|p| path.starts_with(p))
    {
        f |= 1 << 1;
    }
    if is_test(path, base) {
        f |= 1 << 2;
    }
    if config
        .graph
        .entry_globs
        .iter()
        .any(|g| glob_hit(g, path, base))
    {
        f |= 1 << 3;
    }
    if matches!(base, "README.md" | "CLAUDE.md")
        || (path.starts_with("docs/") && matches!(base, "index.md" | "README.md"))
    {
        f |= 1 << 5;
    }
    f
}

/// Spec.hs is the cabal test-suite main-is convention (hspec/stack
/// templates) — the test root nothing imports, like _test.go.
fn is_test(path: &str, base: &str) -> bool {
    base.ends_with("_test.go")
        || base.ends_with(".test.ts")
        || (base.starts_with("test_") && base.ends_with(".py"))
        || base == "Spec.hs"
        || path.starts_with("tests/")
        || path.contains("/tests/")
        || path.contains("/__tests__/")
}

/// entry_globs matching: an exact path, a `dir/` prefix, or a
/// `*.ext` basename pattern — the declarative subset the config
/// documents (full glob syntax is not promised).
fn glob_hit(glob: &str, path: &str, base: &str) -> bool {
    if let Some(ext) = glob.strip_prefix("*.") {
        return base.ends_with(&format!(".{ext}"));
    }
    glob == path || glob == base || (glob.ends_with('/') && path.starts_with(glob))
}
