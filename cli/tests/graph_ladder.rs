//! Resolution-ladder fixtures (design §6 2f exit row: per rung ≥1
//! fixture resolving exactly at that rung, plus ambiguity fixtures
//! that MUST stay Unresolved — an ambiguous fixture that resolves is
//! the red condition). One shared tree and one case table drive
//! every import-shaped language — per-language table pairs were the
//! dedup ratchet's first catch in this file. The Markdown rungs read
//! document CONTENT (headings, definitions), so their doc-shaped
//! habitat lives in graph_ladder_md.rs; the broken-chain case gets
//! its own tree.

use codeeraser::graph::ladder::{self, Outcome, Reason};
use codeeraser::scan::lang::Lang;

mod common;
use common::{Case, ext, fixture, no, ok, pkg, run_cases, site};

/// The shared fixture tree — every rung's habitat, all languages.
const TREE: &[(&str, &str)] = &[
    // TS R1: extension order + exact + traversal; widget.ts beats
    // widget/index.ts because the order IS the norm, not ambiguity
    ("src/app.ts", "export {};\n"),
    ("src/util.ts", "export {};\n"),
    ("src/widget.ts", "export {};\n"),
    ("src/widget/index.ts", "export {};\n"),
    ("shared/x.tsx", "export {};\n"),
    // TS R2: legacy.ts has no .js twin (rewrite fires); built.js
    // exists on disk (rewrite blocked)
    ("src/legacy.ts", "export {};\n"),
    ("src/built.ts", "export {};\n"),
    ("src/built.js", "// compiled artifact\n"),
    // TS R3: extends chain (base contributes baseUrl, child the
    // paths; JSONC comment + trailing comma exercised on purpose)
    (
        "tsconfig.json",
        "{\n  // jsonc on purpose\n  \"extends\": \"./tsconfig.base.json\",\n  \"compilerOptions\": {\n    \"paths\": {\n      \"@app/*\": [\"../src/*\"],\n      \"@dup/*\": [\"../src/dup_a/*\", \"../src/dup_b/*\"],\n    },\n  },\n}\n",
    ),
    (
        "tsconfig.base.json",
        "{\"compilerOptions\": {\"baseUrl\": \"./lib\"}}\n",
    ),
    ("lib/leaf.ts", "export {};\n"),
    ("src/dup_a/thing.ts", "export {};\n"),
    ("src/dup_b/thing.ts", "export {};\n"),
    // TS R4: one clean member, one duplicate-name pair, one member
    // whose exports conditions point at two distinct in-scope files
    (
        "packages/pkga/package.json",
        "{\"name\": \"pkga\", \"exports\": {\".\": {\"source\": \"./src/index.ts\", \"default\": \"./dist/index.js\"}, \"./sub\": \"./src/sub.ts\"}}\n",
    ),
    ("packages/pkga/src/index.ts", "export {};\n"),
    ("packages/pkga/src/sub.ts", "export {};\n"),
    ("packages/dupa/package.json", "{\"name\": \"dup-pkg\"}\n"),
    ("packages/dupb/package.json", "{\"name\": \"dup-pkg\"}\n"),
    (
        "packages/pkgb/package.json",
        "{\"name\": \"pkgb\", \"exports\": {\".\": {\"a\": \"./one.ts\", \"b\": \"./two.ts\"}}}\n",
    ),
    ("packages/pkgb/one.ts", "export {};\n"),
    ("packages/pkgb/two.ts", "export {};\n"),
    // TS R5: declared dependency vs physically vendored vs nothing
    (
        "package.json",
        "{\"name\": \"rootpkg\", \"dependencies\": {\"lodash\": \"^4\"}}\n",
    ),
    ("node_modules/leftover/index.js", "// vendored\n"),
    // Py R1-R3: package chain; R2 ambiguity needs the same dotted
    // path under two roots; pyproject declares one dependency (R4)
    ("pkg/__init__.py", "\n"),
    ("pkg/consumer.py", "\n"),
    ("pkg/mod.py", "\n"),
    ("pkg/sub/__init__.py", "\n"),
    ("pkg/sub/leaf.py", "\n"),
    ("other.py", "\n"),
    ("top.py", "\n"),
    ("tool/__init__.py", "\n"),
    ("src/tool/__init__.py", "\n"),
    (
        "pyproject.toml",
        "[project]\nname = \"fixture\"\ndependencies = [\"requests>=2.31\"]\n",
    ),
    // Rust R1-R3: lib+main double root (a walk ending AT the root
    // must refuse), a mod.rs chain, a 2018-style subdir, an E0761
    // double habitat, and a [[bin]] crate with its own tree
    (
        "Cargo.toml",
        "[package]\nname = \"rootcrate\"\n\n[dependencies]\nserde = \"1\"\nhelper-lib = { path = \"crates/helper\" }\n\n[[bin]]\nname = \"gen\"\npath = \"tools/gen.rs\"\n",
    ),
    ("src/lib.rs", "mod util;\n"),
    ("src/main.rs", "fn main() {}\n"),
    ("src/util.rs", "mod helper;\n"),
    ("src/util/helper.rs", "\n"),
    ("src/nest/mod.rs", "mod deep;\n"),
    ("src/nest/deep.rs", "\n"),
    ("src/dual.rs", "\n"),
    ("src/dual/mod.rs", "\n"),
    ("tools/gen.rs", "mod gadget;\n"),
    ("tools/gadget.rs", "\n"),
    // Rust R4: hyphenated member name (code says helper_lib), a
    // duplicate-name pair, registry dep declared in root Cargo.toml
    ("crates/helper/Cargo.toml", "[package]\nname='helper-lib'"),
    ("crates/helper/src/lib.rs", "\n"),
    ("crates/helper/src/sub.rs", "\n"),
    ("crates/dupx/Cargo.toml", "[package]\nname='dup-crate'"),
    ("crates/dupx/src/lib.rs", "\n"),
    ("crates/dupy/Cargo.toml", "[package]\nname='dup-crate'"),
    ("crates/dupy/src/lib.rs", "\n"),
    // Go R1-R3: nested module, a local replace target, a test-only
    // dir that refuses, and a duplicate module pair
    (
        "go.mod",
        "module x.io/root\nreplace x.io/away => ./vendored\n",
    ),
    ("cmd/main.go", "package main\n"),
    ("pkg/util/util.go", "package util\n"),
    ("pkg/util/util_test.go", "package util\n"),
    ("tstonly/only_test.go", "package tstonly\n"),
    ("vendored/v.go", "package vendored\n"),
    ("nested/go.mod", "module x.io/nested\n"),
    ("nested/lib/lib.go", "package lib\n"),
    ("dupg/a/go.mod", "module x.io/dup\n"),
    ("dupg/a/x/x.go", "package x\n"),
    ("dupg/b/go.mod", "module x.io/dup\n"),
    ("dupg/b/x/x.go", "package x\n"),
];

/// TS + Py rows — the kind-uniform import ladders. Tables split by
/// dispatch shape, not to hide length.
fn import_cases() -> Vec<Case> {
    let (ts, py, im) = (Lang::TypeScript, Lang::Python, "import");
    let (app, con) = ("src/app.ts", "pkg/consumer.py");
    vec![
        (ts, im, app, "./util", ok("src/util.ts", 1)),
        (ts, im, app, "./widget", ok("src/widget.ts", 1)),
        (ts, im, app, "./util.ts", ok("src/util.ts", 1)),
        (ts, im, app, "../shared/x", ok("shared/x.tsx", 1)),
        (ts, im, app, "./legacy.js", ok("src/legacy.ts", 2)),
        (ts, im, app, "./built.js", no(Reason::OutOfScope)),
        (ts, im, app, "@app/util", ok("src/util.ts", 3)),
        (ts, im, app, "leaf", ok("lib/leaf.ts", 3)),
        (ts, im, app, "@dup/thing", no(Reason::AmbiguousPaths)),
        (ts, im, app, "pkga", ok("packages/pkga/src/index.ts", 4)),
        (ts, im, app, "pkga/sub", ok("packages/pkga/src/sub.ts", 4)),
        (ts, im, app, "dup-pkg", no(Reason::AmbiguousWorkspace)),
        (ts, im, app, "pkgb", no(Reason::AmbiguousExports)),
        (ts, im, app, "lodash", ext(5)),
        (ts, im, app, "leftover", ext(5)),
        (ts, im, app, "unknown-pkg", no(Reason::OutOfScope)),
        (ts, im, app, "./missing", no(Reason::OutOfScope)),
        (py, im, con, ".mod", ok("pkg/mod.py", 1)),
        (py, im, con, ".", ok("pkg/__init__.py", 1)),
        (py, im, con, ".sub", ok("pkg/sub/__init__.py", 1)),
        (py, im, con, ".sub.leaf", ok("pkg/sub/leaf.py", 1)),
        (py, im, con, "..other", ok("other.py", 1)),
        (py, im, con, "...breaks", no(Reason::OutOfScope)),
        (py, im, con, "top", ok("top.py", 2)),
        (py, im, con, "pkg.mod", ok("pkg/mod.py", 2)),
        (py, im, con, "tool", no(Reason::AmbiguousRoot)),
        (py, im, con, "pkg.missing", ok("pkg/__init__.py", 3)),
        (py, im, con, "pkg.sub.missing", ok("pkg/sub/__init__.py", 3)),
        (py, im, con, "os", ext(4)),
        (py, im, con, "os.path", ext(4)),
        (py, im, con, "requests", ext(4)),
        (py, im, con, "nosuch_pkg", no(Reason::OutOfScope)),
    ]
}

/// Go rows — package granularity: pkg() targets a DIRECTORY, and a
/// test-only dir, a double-declared module, and a dotless non-std
/// head all refuse.
fn go_cases() -> Vec<Case> {
    let (go, im, gm, nlib) = (Lang::Go, "import", "cmd/main.go", "nested/lib/lib.go");
    let pu = "pkg/util";
    vec![
        (go, im, gm, "x.io/root/pkg/util", pkg(pu, 1)),
        (go, im, nlib, "x.io/root/pkg/util", pkg(pu, 1)),
        (go, im, gm, "x.io/nested/lib", pkg("nested/lib", 1)),
        (go, im, gm, "x.io/root/tstonly", no(Reason::OutOfScope)),
        (go, im, gm, "x.io/root/missing", no(Reason::OutOfScope)),
        (go, im, gm, "x.io/away", pkg("vendored", 2)),
        (go, im, gm, "x.io/dup/x", no(Reason::AmbiguousWorkspace)),
        (go, im, gm, "fmt", ext(3)),
        (go, im, gm, "notreal", no(Reason::OutOfScope)),
        (go, im, gm, "net/http", ext(3)),
        (go, im, gm, "github.com/spf13/pflag", ext(3)),
    ]
}

/// Rust rows — kind-dispatched: mod_decl builds the tree, use walks.
fn rust_cases() -> Vec<Case> {
    let (rs, md, us) = (Lang::Rust, "mod_decl", "use");
    let (rlib, rutil, deep) = ("src/lib.rs", "src/util.rs", "src/nest/deep.rs");
    let (nmod, tbin) = ("src/nest/mod.rs", "tools/gen.rs");
    let (uh, hl) = ("src/util/helper.rs", "crates/helper/src/lib.rs");
    vec![
        // R1: root / mod.rs / 2018-subdir / [[bin]] anchors, the
        // E0761 double, and the #[path] honesty row
        (rs, md, rlib, "util", ok("src/util.rs", 1)),
        (rs, md, rlib, "nest", ok(nmod, 1)),
        (rs, md, rutil, "helper", ok(uh, 1)),
        (rs, md, nmod, "deep", ok(deep, 1)),
        (rs, md, tbin, "gadget", ok("tools/gadget.rs", 1)),
        (rs, md, rlib, "dual", no(Reason::AmbiguousPaths)),
        (rs, md, rlib, "pathed", no(Reason::OutOfScope)),
        // R2: crate:: walks; a walk ending AT the root sees lib+main
        // and refuses; a folded fragment's pre-{ prefix is complete;
        // a hand-folded mid-path fragment is refused
        (rs, us, rutil, "crate::nest::deep", ok(deep, 2)),
        (rs, us, deep, "crate::util::helper as h", ok(uh, 2)),
        (rs, us, rutil, "crate::nest::{", ok(nmod, 2)),
        (rs, us, rutil, "crate::missing", no(Reason::AmbiguousRoot)),
        (rs, us, tbin, "crate::gadget", ok("tools/gadget.rs", 2)),
        (rs, us, rutil, "crate::dual::x", no(Reason::AmbiguousPaths)),
        (rs, us, rutil, "foo::", no(Reason::OutOfScope)),
        // R3: the island red condition — an intra-file self::
        // reference must come home to its own file, never dangle
        (rs, us, deep, "self::helpers", ok(deep, 3)),
        (rs, us, nmod, "self::deep", ok(deep, 3)),
        (rs, us, deep, "super::x", ok(nmod, 3)),
        (rs, us, deep, "super::super::util", ok("src/util.rs", 3)),
        (rs, us, rutil, "super::super::x", no(Reason::OutOfScope)),
        // R4: normalized member name, member-tree descent (the audit
        // records the definition file, not the crate façade),
        // duplicate pair, registry dep, nothing, builtin (order
        // breaks table-tail cloning)
        (rs, us, rutil, "helper_lib::x", ok(hl, 4)),
        (
            rs,
            us,
            rutil,
            "helper_lib::sub::y",
            ok("crates/helper/src/sub.rs", 4),
        ),
        (rs, us, rlib, "dup_crate", no(Reason::AmbiguousWorkspace)),
        (rs, us, rutil, "serde::Serialize", ext(4)),
        (rs, us, rutil, "nowhere::x", no(Reason::OutOfScope)),
        (rs, us, rutil, "std::fs", ext(4)),
    ]
}

#[test]
fn rungs_resolve_and_refuse() {
    let fx = fixture("ladder-rungs", TREE);
    let all = import_cases()
        .into_iter()
        .chain(rust_cases())
        .chain(go_cases());
    run_cases(&fx, all.collect());
}

/// An extends cycle must refuse the whole tsconfig rung — config
/// beyond the modeled chain is config_depth, never a guess.
#[test]
fn extends_cycle_is_config_depth() {
    let fx = fixture(
        "ladder-cycle",
        &[
            ("a.ts", "export {};\n"),
            ("tsconfig.json", "{\"extends\": \"./other.json\"}\n"),
            ("other.json", "{\"extends\": \"./tsconfig.json\"}\n"),
        ],
    );
    assert_eq!(
        ladder::resolve(
            Lang::TypeScript,
            &site("import", "a.ts", "anything", 1),
            &fx.scope()
        ),
        Outcome::Unresolved(Reason::ConfigDepth)
    );
}

/// Inline `mod` bodies anchor self/super at the FILE, not its parent
/// (the audited interpolate.rs and globset rows): ups consume inline
/// depth before any file climb, self inside an inline module is a
/// self edge, and a post-climb tail may still name a FILE module.
#[test]
fn inline_mod_super_comes_home() {
    let fx = fixture(
        "ladder-inline",
        &[
            ("Cargo.toml", "[package]\nname='inl'"),
            (
                "src/lib.rs",
                "pub fn escape() {}\nmod util;\n#[cfg(test)]\nmod tests {\n    use super::escape;\n    use self::helper::x;\n    use super::util;\n    mod deep {\n        use super::super::escape;\n    }\n}\n",
            ),
            ("src/util.rs", "\n"),
        ],
    );
    let scope = fx.scope();
    let at = |spec: &'static str, line: usize| {
        ladder::resolve(Lang::Rust, &site("use", "src/lib.rs", spec, line), &scope)
    };
    assert_eq!(at("super::escape", 5), ok("src/lib.rs", 3));
    assert_eq!(at("self::helper::x", 6), ok("src/lib.rs", 3));
    assert_eq!(at("super::util", 7), ok("src/util.rs", 3));
    assert_eq!(at("super::super::escape", 9), ok("src/lib.rs", 3));
}
