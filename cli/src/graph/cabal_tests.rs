//! The cabal surface's facts: the self-corpus parse the 3l
//! reconciliation stands on, the dead-region boundary the clearance
//! review drew, and the two package-privacy facts the mounts table
//! reads (plan v2.17 piece (5)).

use super::{Cabal, nearest, parse};

/// The REAL core cabal is the self-corpus resolution anchor: two
/// stanzas (executable app; test-suite app+test — the multi-root
/// comma line), five deps through a multi-line build-depends
/// block. The 3l reconciliation stands on exactly this parse:
/// 176 hs sites = 78 in-corpus R1 edges + 80 declared-external
/// (base/containers/bytestring/array) + 18 aeson out-of-db.
#[test]
fn the_self_corpus_cabal_parses_to_its_known_facts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent");
    let c = parse(root, "core/ce-core.cabal").expect("parse");
    assert_eq!(c.dir, "core");
    let roots: Vec<&[String]> = c.stanzas.iter().map(|s| s.roots.as_slice()).collect();
    assert_eq!(
        roots,
        [
            ["core/app".to_string()].as_slice(),
            ["core/app".to_string(), "core/test".to_string()].as_slice()
        ]
    );
    assert_eq!(
        c.deps,
        ["aeson", "array", "base", "bytestring", "containers"]
    );
    // the self cabal's main-is facts, pinned with the rest (2.28.0)
    let mains: Vec<Option<&str>> = c.stanzas.iter().map(|s| s.main_is.as_deref()).collect();
    assert_eq!(mains, [Some("Main.hs"), Some("Spec.hs")]);
    // an executable-only package keeps every module (§4 bit 1), and
    // the nearest probe finds this very file from its own directory
    assert!(!c.has_library);
    assert!(c.keeps_private("core/app/CE/Score.hs"));
    assert_eq!(
        nearest(root, "core/app/CE").as_deref(),
        Some("core/ce-core.cabal")
    );
}

/// One scratch package per probe — the tests run in parallel, and a
/// pid-keyed single path raced the moment a second test used it.
fn parse_str(tag: &str, text: &str) -> Cabal {
    let dir = crate::testutil::scratch(&format!("cabal-{tag}"));
    crate::testutil::write_tree(&dir, &[("pkg/x.cabal", text)]);
    let parsed = parse(&dir, "pkg/x.cabal").expect("parse");
    std::fs::remove_dir_all(&dir).ok();
    parsed
}

/// The two dead-region boundary cases the clearance review drew
/// (both counterfactual against the first `live`-bit cut):
/// a column-0 comment INSIDE a live stanza must not open a dead
/// region, and a `common` stanza drops its roots but its
/// build-depends still join the file-wide union feeding R2.
#[test]
fn comments_keep_stanzas_live_and_common_deps_still_count() {
    let c = parse_str(
        "note",
        "library\n-- top-level note\n  hs-source-dirs: src\n  build-depends: base\n",
    );
    assert_eq!(c.stanzas[0].roots, ["pkg/src".to_string()]);
    assert_eq!(c.deps, ["base"]);
    let c = parse_str(
        "common",
        "common deps\n  hs-source-dirs: gen\n  build-depends: text\nlibrary\n  hs-source-dirs: src\n",
    );
    assert_eq!(c.stanzas.len(), 1, "common opens no stanza");
    assert_eq!(
        c.stanzas[0].roots,
        ["pkg/src".to_string()],
        "common roots dropped"
    );
    assert_eq!(
        c.deps,
        ["text"],
        "common build-depends still declared in this file"
    );
}

/// The privacy facts (sealed criterion §4 bit 1): a library's
/// exposed module is open, its other-module is hidden, a module the
/// library exposes but a test-suite also compiles stays open, and a
/// file under no stanza root spells no module. The package root is
/// `pkg`, so the module name is read below `pkg/src`.
#[test]
fn hidden_modules_are_other_minus_exposed_and_keep_their_files() {
    let c = parse_str(
        "hidden",
        "library\n  hs-source-dirs: src\n  exposed-modules: A, A.B\n  other-modules:\n    Internal.C\n    A.B\ntest-suite t\n  hs-source-dirs: test\n  other-modules: A\n",
    );
    assert!(c.has_library);
    assert_eq!(
        c.hidden_modules.iter().collect::<Vec<_>>(),
        ["Internal.C"],
        "listed under other-modules and nowhere under exposed-modules"
    );
    let kept: Vec<&str> = [
        "pkg/src/A.hs",
        "pkg/src/A/B.hs",
        "pkg/src/Internal/C.hs",
        "pkg/test/A.hs",
        "pkg/stray/Internal/C.hs",
    ]
    .into_iter()
    .filter(|p| c.keeps_private(p))
    .collect();
    assert_eq!(kept, ["pkg/src/Internal/C.hs"]);
}

/// Stanza order must not matter (review of this piece): an executable
/// written ABOVE the library with no hs-source-dirs roots at the
/// package directory, an ancestor of `src`, and a first-root read
/// spelled `src.B` for the library's hidden `B`; the mirror — a
/// library rooted at `.` above an executable rooted at `app` — hid
/// the executable's own other-module the same way. And a NAMED
/// `library x` header is an internal sublibrary, not a public one:
/// a package with only that keeps everything.
#[test]
fn privacy_does_not_depend_on_stanza_order_or_count_sublibraries() {
    let exec_first = parse_str(
        "order-a",
        "executable app\n  main-is: Main.hs\nlibrary\n  hs-source-dirs: src\n  exposed-modules: A\n  other-modules: B\n",
    );
    assert!(exec_first.keeps_private("pkg/src/B.hs"));
    assert!(!exec_first.keeps_private("pkg/src/A.hs"));
    let lib_first = parse_str(
        "order-b",
        "library\n  hs-source-dirs: .\n  exposed-modules: A\nexecutable app\n  hs-source-dirs: app\n  other-modules: Opts\n",
    );
    assert!(lib_first.keeps_private("pkg/app/Opts.hs"));
    assert!(!lib_first.keeps_private("pkg/A.hs"));
    let sub_only = parse_str(
        "sublib",
        "library internal-utils\n  hs-source-dirs: src\n  exposed-modules: U\nexecutable app\n  main-is: Main.hs\n",
    );
    assert!(!sub_only.has_library, "a named library is a sublibrary");
    assert!(sub_only.keeps_private("pkg/src/U.hs"));
}
