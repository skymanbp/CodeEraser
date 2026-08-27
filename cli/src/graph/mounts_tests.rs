//! The producer's own facts (K30's producer half, sealed criterion
//! §4): the row projection covers every node, and each bit-1 arm reads
//! its manifest or file the way the criterion says. The graph read
//! (`facts`) needs a real index and is measured end to end in
//! tests/it/graph_mounts.rs.

use super::*;
use crate::graph::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use crate::testutil::{node, scratch, write_tree};

/// Every node gets exactly one row in node order: file facts land on
/// file nodes, a file without facts reads zero, both bits compose on
/// one file (a first-bit-wins `if/else` would lose one), and a
/// package, a section and a phantom file node read `[0,0,0]` — never
/// a missing row (§4 W8-F4: the core reads absence as zeros, so
/// absence must be impossible here).
#[test]
fn every_node_gets_a_row_and_only_file_nodes_carry_facts() {
    let nodes = [
        node("pkg", "", GRAN_PACKAGE),
        node("src/a.rs", "", GRAN_FILE),
        node("src/b.rs", "", GRAN_FILE),
        node("docs/x.md", "Intro", GRAN_SECTION),
        node("gone.rs", "", GRAN_FILE),
    ];
    let mut facts = MountFacts::default();
    facts.mounts.insert("src/a.rs".into(), (1, 2));
    facts.reexported.insert("src/a.rs".into());
    facts.pkg_private.insert("src/a.rs".into());
    facts.pkg_private.insert("src/b.rs".into());
    facts.pkg_private.insert("pkg".into()); // a package path is never a file fact
    let rows = mount_rows(&nodes, &facts);
    let want = [
        [0, 0, 0],
        [1, 2, MOUNT_REEXPORTED | MOUNT_PKG_PRIVATE],
        [0, 0, MOUNT_PKG_PRIVATE],
        [0, 0, 0],
        [0, 0, 0],
    ];
    assert_eq!(rows.len(), nodes.len(), "one row per node, always");
    for (i, row) in want.into_iter().enumerate() {
        assert_eq!(rows[&(i as i64)], row, "node {i}");
    }
}

/// `package main` and an `internal/` segment keep a file; a library
/// package outside internal/ does not; a `_test.go` never does; the
/// clause is read past a leading comment block in BOTH directions — a
/// doc comment's indented `package main` example must not keep a
/// `package cmdutil` file, and a comment naming another package must
/// not hide a real `package main`.
#[test]
fn go_privacy_reads_the_clause_and_the_path() {
    let root = scratch("mounts-go");
    let tree = [
        (
            "cmd/main.go",
            "// Command x.\n//go:build linux\n\npackage main\n",
        ),
        ("lib/lib.go", "package lib\n"),
        ("internal/y/y.go", "package y\n"),
        ("internal/y/y_test.go", "package y\n"),
        (
            "pkg/doc.go",
            "/*\nPackage pkg does things.\n*/\npackage pkg // import \"x/pkg\"\n",
        ),
        (
            "pkg/example.go",
            "/*\nExample:\n\n\tpackage main\n\n\tfunc main() {}\n*/\npackage cmdutil\n",
        ),
        (
            "legacy/legacy.go",
            "/*\nDeprecated: package helper moved here.\n*/\npackage main\n",
        ),
    ];
    write_tree(&root, &tree);
    let kept: Vec<&str> = tree
        .iter()
        .map(|(p, _)| *p)
        .filter(|p| go_private(&root, p))
        .collect();
    assert_eq!(kept, ["cmd/main.go", "internal/y/y.go", "legacy/legacy.go"]);
    std::fs::remove_dir_all(&root).ok();
}

/// The Rust arm's three outcomes: a virtual workspace manifest is not
/// a package and keeps nothing (a stray file under it); no lib target
/// keeps the whole package (a helper module included); a lib target
/// keeps the bin roots alone — the default main, a declared [[bin]]
/// path, a src/bin child — and neither the lib root, a plain module,
/// nor a test.
#[test]
fn rust_privacy_is_nothing_the_whole_package_or_its_bin_roots() {
    let root = scratch("mounts-rs");
    write_tree(
        &root,
        &[
            ("Cargo.toml", "[workspace]\nmembers = [\"lib\", \"tool\"]\n"),
            (
                "lib/Cargo.toml",
                "[package]\nname = \"lib\"\n[[bin]]\nname = \"gen\"\npath = \"src/tools/gen.rs\"\n",
            ),
            ("tool/Cargo.toml", "[package]\nname = \"tool\"\n"),
        ],
    );
    let sources = [
        "scratch/x.rs",
        "lib/src/lib.rs",
        "lib/src/main.rs",
        "lib/src/bin/extra.rs",
        "lib/src/tools/gen.rs",
        "lib/src/module.rs",
        "lib/tests/t.rs",
        "tool/src/main.rs",
        "tool/src/util.rs",
    ];
    let files: BTreeSet<String> = sources.map(String::from).into();
    let targets = |manifest: &str| RustTargets::of(cargo::package(&root, manifest), &files);
    let (root_ws, lib, tool) = (
        targets("Cargo.toml"),
        targets("lib/Cargo.toml"),
        targets("tool/Cargo.toml"),
    );
    let kept: Vec<&str> = sources
        .into_iter()
        .filter(|p| {
            let pkg = match p.split('/').next() {
                Some("lib") => &lib,
                Some("tool") => &tool,
                _ => &root_ws,
            };
            pkg.keeps(p)
        })
        .collect();
    assert_eq!(
        kept,
        [
            "lib/src/main.rs",
            "lib/src/bin/extra.rs",
            "lib/src/tools/gen.rs",
            "tool/src/main.rs",
            "tool/src/util.rs",
        ]
    );
    std::fs::remove_dir_all(&root).ok();
}
