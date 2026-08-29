//! The reading (git's own grammar), the owner rule and the seating
//! questions, the last two on ONE scratch tree.

use super::*;
use std::path::PathBuf;

/// Every spelling git reads, and the two it must not: one document
/// per case so the table is the grammar, not a loop over stanzas.
/// The newline-in-value and tab cases are `git config -f` 2.52's
/// own answers on this machine, byte for byte.
#[test]
fn the_reading_is_gits_own() {
    const CASES: [(&str, &[&str]); 12] = [
        ("[submodule \"s\"]\n\tpath = cli/tests\n", &["cli/tests"]),
        (
            "[submodule \"s\"]\n\tpath = \"cli/tests\"\n",
            &["cli/tests"],
        ),
        (
            "[submodule \"s\"]\n\tpath = cli/tests # the suite\n",
            &["cli/tests"],
        ),
        (
            "[submodule \"s\"]\n\tpath = cli/tests ; note\n",
            &["cli/tests"],
        ),
        ("[SUBMODULE \"s\"]\n\tPath = cli/tests\n", &["cli/tests"]),
        ("[submodule \"s\"] path = cli/tests\n", &["cli/tests"]),
        ("[include]\n\tpath = ../elsewhere\n", &[]),
        ("[submodule \"a#b\"]\n\tpath = \"a#b\"\n", &["a#b"]),
        ("[submodule \"s\"]\n\tpath = \"a\\\\b\"\n", &["a/b"]),
        (
            "[submodule \"s\"]\n\tpath = cli/\\\n\ttests\n",
            &["cli/\n\ttests"],
        ),
        ("[submodule \"s\"]\n\tpath = a\t\tb  \n", &["a\t\tb"]),
        (
            "[submodule \"s\"]\n\tpath = declared/\n\turl = x\n",
            &["declared"],
        ),
    ];
    for (text, want) in CASES {
        let got: Vec<String> = parse(text).into_iter().collect();
        assert_eq!(got, want, "{text:?}");
    }
}

/// A full-line comment, a non-path key and a `]` inside the
/// subsection name declare nothing; two stanzas declare two.
#[test]
fn only_a_submodule_stanzas_path_declares() {
    let text = "[submodule \"a]b\"]\n\t# path = ghost\n\tignore = all\n\turl = u\n\
                \tpath = one\n[submodule \"two\"]\n\tpath=two\n[core]\n\tpath = no\n";
    let got: Vec<String> = parse(text).into_iter().collect();
    assert_eq!(got, ["one", "two"]);
}

/// One scratch tree every shape the owner rule and the seating
/// questions are asked about stands in, declared by one `.gitmodules`:
/// `suite` (a gitfile pointing nowhere), `hollow` (an empty directory
/// — the clone-without-recurse and deinit shape), `filed` (a regular
/// file at the path), `copied` (files without a checkout), `real` (a
/// checkout), `vendored/inner` (declared under an undeclared
/// repository — unreachable) and `ghost` (absent — a stale stanza);
/// `vendored` is the undeclared repository and `src` is this tree's.
fn shapes(tag: &str) -> (PathBuf, BTreeSet<String>) {
    let root = crate::testutil::scratch(tag);
    crate::testutil::write_tree(
        &root,
        &[
            ("suite/.git", "gitdir: ../.git/modules/suite\n"),
            ("suite/it/a.rs", ""),
            ("hollow/.keep", ""),
            ("filed", "not a directory\n"),
            ("copied/a.rs", ""),
            ("real/.git/HEAD", "ref: refs/heads/main\n"),
            ("vendored/.git/HEAD", "ref: refs/heads/main\n"),
            ("vendored/lib.rs", ""),
            ("vendored/inner/.keep", ""),
            ("src/lib.rs", ""),
            (
                ".gitmodules",
                "[submodule \"suite\"]\n\tpath = suite\n\
                 [submodule \"hollow\"]\n\tpath = hollow\n\
                 [submodule \"filed\"]\n\tpath = filed\n\
                 [submodule \"copied\"]\n\tpath = copied\n\
                 [submodule \"real\"]\n\tpath = real\n\
                 [submodule \"inner\"]\n\tpath = vendored/inner\n\
                 [submodule \"ghost\"]\n\tpath = ghost\n",
            ),
        ],
    );
    std::fs::remove_file(root.join("hollow/.keep")).expect("hollow is empty");
    let declared = declared(&root);
    (root, declared)
}

/// The owner rule: a declared prefix answers Foreign before its
/// `.git` is even looked at (seated, unseated or absent), an
/// undeclared real anchor answers Cut — a declaration below it
/// included — and everything else, a path UNDER nothing too, is Own.
#[test]
fn owner_reads_declarations_before_anchors() {
    let (root, declared) = shapes("gitmodules-owner");
    let cases = [
        ("suite/it/a.rs", Owner::Foreign),
        ("suite", Owner::Foreign),
        ("hollow/x.rs", Owner::Foreign),
        ("real/x.rs", Owner::Foreign),
        ("ghost/x.rs", Owner::Foreign),
        ("vendored/lib.rs", Owner::Cut),
        ("vendored", Owner::Cut),
        ("vendored/inner/x.rs", Owner::Cut),
        ("src/lib.rs", Owner::Own),
        ("src", Owner::Own),
        ("", Owner::Own),
    ];
    for (rel, want) in cases {
        assert_eq!(owner(&root, &declared, rel), want, "{rel}");
    }
    std::fs::remove_dir_all(&root).ok();
}

/// Seating is the real anchor, asked through the owner rule: the
/// gitfile pointing nowhere, the empty directory, the regular file
/// and the copied files are present and unseated; the checkout is
/// seated, and gated once it carries a ce.toml; the declaration under
/// the undeclared repository and the absent one name nothing (codex
/// review of step #12: an `.exists()` seated the broken gitfile and
/// only the empty directory read unseated).
#[test]
fn seating_reads_the_real_anchor_through_the_owner_rule() {
    let (root, _declared) = shapes("gitmodules-seating");
    assert_eq!(unseated(&root), ["copied", "filed", "hollow", "suite"]);
    assert_eq!(seated(&root), ["real"]);
    assert_eq!(gated(&root), Vec::<String>::new());
    std::fs::write(root.join("real/ce.toml"), "").expect("gate");
    assert_eq!(gated(&root), ["real"]);
    std::fs::remove_dir_all(&root).ok();
}
