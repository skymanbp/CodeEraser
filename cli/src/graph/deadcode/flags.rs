//! Entry-flag policy for file nodes (split from deadcode.rs at the
//! 300-line dogfood wall): the mechanical conventions plus user
//! config that mark a file as a root the liveness judgment must not
//! call dead. Policy data only — the wire row that carries these
//! bits stays in deadcode.rs with the rest of the request assembly.

use crate::config::Config;
use std::path::Path;

/// Mechanical entry conventions (module header); bit 0 (exported)
/// stays unset at file granularity — public-ness is a symbol fact
/// (3l re-review: Haskell module export lists included — a header's
/// exports node is symbol-level, same stance as the five launch
/// languages). Main.hs is cabal's executable main-is convention —
/// nothing imports a main module, exactly like main.rs. Bit 6 is
/// the inline liveness claim (batch-7 slice 3): the core reserved
/// it in entryMask from day one, and until now nothing produced it.
pub(super) fn flags_of(root: &Path, path: &str, config: &Config) -> i64 {
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
    if allow_claim(root, path) {
        f |= 1 << 6;
    }
    f
}

/// `ce:allow(deadcode) -- <why>` anywhere in the file claims
/// liveness — the docdup exemption discipline transplanted: a BARE
/// marker without the ` -- why` tail claims NOTHING ("no why = a
/// violation", plan :79). An unreadable file makes no claim. Full
/// content scan per file node per run; the index already read every
/// byte upstream, so the pages are warm.
fn allow_claim(root: &Path, path: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join(path)) else {
        return false;
    };
    text.match_indices("ce:allow(deadcode)").any(|(i, m)| {
        text[i + m.len()..]
            .trim_start_matches([' ', '\t'])
            .starts_with("-- ")
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The bit-6 producer (batch-7 slice 3), table-driven — the
    /// docdup discipline transplanted: only a why-bearing marker
    /// claims; a bare marker and an absent file claim nothing.
    #[test]
    fn allow_claim_requires_the_why_tail() {
        let cases = [
            (
                "a.py",
                Some("# ce:allow(deadcode) -- loader-invoked\n"),
                true,
            ),
            ("b.py", Some("# ce:allow(deadcode)\n"), false),
            ("missing.py", None, false),
        ];
        let root = crate::testutil::scratch("dc-allow");
        let cfg = crate::config::Config::default();
        for (name, text, want) in cases {
            if let Some(text) = text {
                std::fs::write(root.join(name), text).unwrap();
            }
            assert_eq!(allow_claim(&root, name), want, "{name}");
            assert_eq!(flags_of(&root, name, &cfg) & (1 << 6) != 0, want, "{name}");
        }
        std::fs::remove_dir_all(&root).ok();
    }
}
