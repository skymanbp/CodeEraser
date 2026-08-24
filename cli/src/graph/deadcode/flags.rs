//! Entry-role measurement for file nodes (split from deadcode.rs at
//! the 300-line dogfood wall). Since proto 2.28.0 (batch-7 slice 3
//! main body) this side measures ROLE FACTS — named main, executable
//! dir, test convention, entry glob, doc entry, allow claim, declared
//! build target — and the entry DECISION is the core's role table
//! (CE.Graph.Cost.roleBits). The legacy flags column is still
//! produced bit-identically to the pre-2.28 semantics and yields to
//! the roles column wherever a 2.28 core judges; it retires next
//! minor. Bit 0 (exported) stays unset at file granularity —
//! public-ness is a symbol fact (3l re-review).

use super::targets::Declared;
use crate::config::Config;
use std::path::Path;

/// Frozen role-bit positions (wire node row column 4). Facts, never
/// verdicts: which entry bits each role lands on is the core's
/// roleBits table, where an ablation can perturb it.
pub(super) const ROLE_ENTRY_NAMED: i64 = 1;
pub(super) const ROLE_ENTRY_DIR: i64 = 1 << 1;
pub(super) const ROLE_TEST: i64 = 1 << 2;
pub(super) const ROLE_GLOB: i64 = 1 << 3;
pub(super) const ROLE_DOC: i64 = 1 << 4;
pub(super) const ROLE_ALLOW: i64 = 1 << 5;
pub(super) const ROLE_DECLARED: i64 = 1 << 6;

/// (role, legacy flag bit): the transitional fold legacy_flags
/// applies — the pre-2.28 bit semantics verbatim, so an old core
/// judging the legacy column behaves exactly as before this minor.
/// ROLE_DECLARED is deliberately absent: its entry standing is NEW
/// and core-side only (the slice-3 defect fix — a declared
/// [[bin]] path earned no root while a stray main.rs did).
const LEGACY: [(i64, u32); 6] = [
    (ROLE_ENTRY_NAMED, 1),
    (ROLE_ENTRY_DIR, 1),
    (ROLE_TEST, 2),
    (ROLE_GLOB, 3),
    (ROLE_DOC, 5),
    (ROLE_ALLOW, 6),
];

/// Role facts of one file node. Main.hs is cabal's executable
/// main-is convention — nothing imports a main module, exactly like
/// main.rs; the declared-target role covers the manifests' OWN
/// declarations beside these name conventions.
pub(super) fn roles_of(root: &Path, path: &str, config: &Config, declared: &Declared) -> i64 {
    let base = path.rsplit('/').next().unwrap_or(path);
    let mut r = 0i64;
    if matches!(
        base,
        "main.rs" | "main.go" | "__main__.py" | "build.rs" | "Main.hs"
    ) {
        r |= ROLE_ENTRY_NAMED;
    }
    if ["src/bin/", "examples/", "benches/", "cmd/"]
        .iter()
        .any(|p| path.starts_with(p))
    {
        r |= ROLE_ENTRY_DIR;
    }
    if is_test(path, base) {
        r |= ROLE_TEST;
    }
    if config
        .graph
        .entry_globs
        .iter()
        .any(|g| glob_hit(g, path, base))
    {
        r |= ROLE_GLOB;
    }
    if matches!(base, "README.md" | "CLAUDE.md")
        || (path.starts_with("docs/") && matches!(base, "index.md" | "README.md"))
    {
        r |= ROLE_DOC;
    }
    if allow_claim(root, path) {
        r |= ROLE_ALLOW;
    }
    if declared.hit(path) {
        r |= ROLE_DECLARED;
    }
    r
}

/// The legacy flags column: the LEGACY fold, nothing else.
pub(super) fn legacy_flags(roles: i64) -> i64 {
    LEGACY.iter().fold(
        0,
        |f, &(role, bit)| {
            if roles & role != 0 { f | (1 << bit) } else { f }
        },
    )
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
    use std::collections::BTreeSet;

    fn no_targets() -> Declared {
        Declared::gather(Path::new("."), &BTreeSet::new())
    }

    /// The allow-claim role (batch-7 slice 3), table-driven — the
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
        let none = no_targets();
        for (name, text, want) in cases {
            if let Some(text) = text {
                std::fs::write(root.join(name), text).unwrap();
            }
            assert_eq!(allow_claim(&root, name), want, "{name}");
            let r = roles_of(&root, name, &cfg, &none);
            assert_eq!(r & ROLE_ALLOW != 0, want, "{name}");
            assert_eq!(legacy_flags(r) & (1 << 6) != 0, want, "{name}");
        }
        std::fs::remove_dir_all(&root).ok();
    }

    /// The legacy fold reproduces the pre-2.28 bits per role — one
    /// row per mapping, plus the declared role folding to NOTHING
    /// (its entry standing is core-side only).
    #[test]
    fn legacy_fold_is_the_pre_228_bits() {
        let rows = [
            (ROLE_ENTRY_NAMED, 1 << 1),
            (ROLE_ENTRY_DIR, 1 << 1),
            (ROLE_TEST, 1 << 2),
            (ROLE_GLOB, 1 << 3),
            (ROLE_DOC, 1 << 5),
            (ROLE_ALLOW, 1 << 6),
            (ROLE_DECLARED, 0),
        ];
        for (role, want) in rows {
            assert_eq!(legacy_flags(role), want, "role {role}");
        }
        assert_eq!(
            legacy_flags(ROLE_ENTRY_NAMED | ROLE_TEST | ROLE_DECLARED),
            (1 << 1) | (1 << 2),
        );
    }
}
