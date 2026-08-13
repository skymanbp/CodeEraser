//! Resolution-ladder fixtures (design §6 2f exit row: per rung ≥1
//! fixture resolving exactly at that rung, plus ambiguity fixtures
//! that MUST stay Unresolved — an ambiguous fixture that resolves is
//! the red condition). One shared tree and ONE (lang, from, spec,
//! want) case table drive every language — per-language table pairs
//! were the dedup ratchet's first catch in this file. The
//! broken-chain and dispatcher cases get their own trees.

use codeeraser::graph::ladder::{self, Outcome, Reason, Scope};
use codeeraser::graph::store;
use codeeraser::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

mod common;
use common::tmp;

/// The shared fixture tree — every rung's habitat, both languages.
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
];

fn materialize(dir: &Path, tree: &[(&str, &str)]) -> (BTreeSet<String>, Vec<String>) {
    let mut files = BTreeSet::new();
    let mut configs = Vec::new();
    for (rel, content) in tree {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, content).expect(rel);
        if rel.starts_with("node_modules/") {
            continue; // the real walk never enters node_modules
        }
        if Lang::from_path(&path).is_some() {
            files.insert(rel.to_string());
        }
        if store::is_resolver_config(&path) {
            configs.push(rel.to_string());
        }
    }
    (files, configs)
}

fn ok(path: &str, rung: u8) -> Outcome {
    Outcome::Resolved {
        path: path.to_string(),
        rung,
    }
}

fn no(reason: Reason) -> Outcome {
    Outcome::Unresolved(reason)
}

fn ext(rung: u8) -> Outcome {
    Outcome::External { rung }
}

/// (lang, from-file, spec) → expected outcome, all rungs of both
/// shipped ladders plus every refusal shape.
fn cases() -> Vec<(Lang, &'static str, &'static str, Outcome)> {
    let (ts, py) = (Lang::TypeScript, Lang::Python);
    let (app, con) = ("src/app.ts", "pkg/consumer.py");
    vec![
        (ts, app, "./util", ok("src/util.ts", 1)),
        (ts, app, "./widget", ok("src/widget.ts", 1)),
        (ts, app, "./util.ts", ok("src/util.ts", 1)),
        (ts, app, "../shared/x", ok("shared/x.tsx", 1)),
        (ts, app, "./legacy.js", ok("src/legacy.ts", 2)),
        (ts, app, "./built.js", no(Reason::OutOfScope)),
        (ts, app, "@app/util", ok("src/util.ts", 3)),
        (ts, app, "leaf", ok("lib/leaf.ts", 3)),
        (ts, app, "@dup/thing", no(Reason::AmbiguousPaths)),
        (ts, app, "pkga", ok("packages/pkga/src/index.ts", 4)),
        (ts, app, "pkga/sub", ok("packages/pkga/src/sub.ts", 4)),
        (ts, app, "dup-pkg", no(Reason::AmbiguousWorkspace)),
        (ts, app, "pkgb", no(Reason::AmbiguousExports)),
        (ts, app, "lodash", ext(5)),
        (ts, app, "leftover", ext(5)),
        (ts, app, "unknown-pkg", no(Reason::OutOfScope)),
        (ts, app, "./missing", no(Reason::OutOfScope)),
        (py, con, ".mod", ok("pkg/mod.py", 1)),
        (py, con, ".", ok("pkg/__init__.py", 1)),
        (py, con, ".sub", ok("pkg/sub/__init__.py", 1)),
        (py, con, ".sub.leaf", ok("pkg/sub/leaf.py", 1)),
        (py, con, "..other", ok("other.py", 1)),
        (py, con, "...breaks", no(Reason::OutOfScope)),
        (py, con, "top", ok("top.py", 2)),
        (py, con, "pkg.mod", ok("pkg/mod.py", 2)),
        (py, con, "tool", no(Reason::AmbiguousRoot)),
        (py, con, "pkg.missing", ok("pkg/__init__.py", 3)),
        (py, con, "pkg.sub.missing", ok("pkg/sub/__init__.py", 3)),
        (py, con, "os", ext(4)),
        (py, con, "os.path", ext(4)),
        (py, con, "requests", ext(4)),
        (py, con, "nosuch_pkg", no(Reason::OutOfScope)),
    ]
}

#[test]
fn rungs_resolve_and_refuse() {
    let dir = tmp("ladder-rungs");
    let (files, configs) = materialize(&dir, TREE);
    let scope = Scope {
        files: &files,
        configs: &configs,
        root: &dir,
    };
    for (lang, from, spec, want) in cases() {
        let got = ladder::resolve(lang, from, spec, &scope);
        assert_eq!(got, want, "{lang:?} {spec:?}");
    }
}

/// An extends cycle must refuse the whole tsconfig rung — config
/// beyond the modeled chain is config_depth, never a guess.
#[test]
fn extends_cycle_is_config_depth() {
    let dir = tmp("ladder-cycle");
    let tree: &[(&str, &str)] = &[
        ("a.ts", "export {};\n"),
        ("tsconfig.json", "{\"extends\": \"./other.json\"}\n"),
        ("other.json", "{\"extends\": \"./tsconfig.json\"}\n"),
    ];
    let (files, configs) = materialize(&dir, tree);
    let scope = Scope {
        files: &files,
        configs: &configs,
        root: &dir,
    };
    assert_eq!(
        ladder::resolve(Lang::TypeScript, "a.ts", "anything", &scope),
        Outcome::Unresolved(Reason::ConfigDepth)
    );
}

/// Languages without a ladder yet are honest Unsupported ledger
/// rows, never silent skips (dispatcher contract).
#[test]
fn missing_ladders_are_unsupported() {
    let dir = tmp("ladder-none");
    let (files, configs) = materialize(&dir, &[("m.rs", "use x;\n")]);
    let scope = Scope {
        files: &files,
        configs: &configs,
        root: &dir,
    };
    for lang in [Lang::Rust, Lang::Go, Lang::Markdown] {
        assert_eq!(
            ladder::resolve(lang, "m.rs", "x", &scope),
            Outcome::Unresolved(Reason::Unsupported),
            "{lang:?}"
        );
    }
}
