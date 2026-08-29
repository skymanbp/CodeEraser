//! Entry-role measurement for file nodes (split from deadcode.rs at
//! the 300-line dogfood wall). Since proto 2.28.0 (batch-7 slice 3
//! main body) this side measures ROLE FACTS — named main, executable
//! dir, test convention, entry glob, doc entry, allow claim, declared
//! build target — and the entry DECISION is the core's role table
//! (CE.Graph.Cost.roleBits). The pre-2.28 legacy flags column this
//! module also produced retired at 5.0.0, once the symbols table
//! gave visibility a producer; nothing here measures bit 0, because
//! bit 0 is not a file fact — public-ness is a symbol fact (3l
//! re-review), and it now reaches the core through the export
//! surface (graph/symwire.rs) rather than through this column.

use super::targets::Declared;
use crate::scan::globs::{self, Inclusions};
use std::path::Path;

/// Frozen role-bit positions (wire node row column 4). Facts, never
/// verdicts: which entry bits each role lands on is the core's
/// roleBits table, where an ablation can perturb it.
pub(super) const ROLE_ENTRY_NAMED: i64 = 1;
const ROLE_ENTRY_DIR: i64 = 1 << 1;
const ROLE_TEST: i64 = 1 << 2;
const ROLE_GLOB: i64 = 1 << 3;
const ROLE_DOC: i64 = 1 << 4;
pub(super) const ROLE_ALLOW: i64 = 1 << 5;
const ROLE_DECLARED: i64 = 1 << 6;
/// A declared submodule's node (plan v2.18 step #12, wire 6.3.0): a
/// READER of this tree, never its candidate — the core lands it on
/// the entry bits, and this side measures none of its other roles
/// (they would only cost the reads).
pub(super) const ROLE_FOREIGN: i64 = 1 << 7;

/// Role facts of one file node. Main.hs is cabal's executable
/// main-is convention — nothing imports a main module, exactly like
/// main.rs; the declared-target role covers the manifests' OWN
/// declarations beside these name conventions.
pub(super) fn roles_of(root: &Path, path: &str, entries: &Inclusions, declared: &Declared) -> i64 {
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
    if globs::selected(entries, path) {
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

/// `ce:allow(deadcode) -- <why>` anywhere in the file claims
/// liveness — the one claim grammar (crate::allow: a bare marker
/// claims NOTHING). An unreadable file makes no claim. Full content
/// scan per file node per run; the index already read every byte
/// upstream, so the pages are warm.
fn allow_claim(root: &Path, path: &str) -> bool {
    std::fs::read_to_string(root.join(path))
        .is_ok_and(|text| crate::allow::allow_claim(&text, "ce:allow(deadcode)"))
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

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/flags.rs"]
mod tests;
