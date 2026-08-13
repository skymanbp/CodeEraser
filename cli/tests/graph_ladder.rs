//! TS resolution-ladder fixtures (design §6 2f exit row: per rung
//! ≥1 fixture resolving exactly at that rung, plus ambiguity
//! fixtures that MUST stay Unresolved — an ambiguous fixture that
//! resolves is the red condition). One shared tree + a case table;
//! the broken-chain and dispatcher cases get their own trees.

use codeeraser::graph::ladder::{self, Outcome, Reason, Scope};
use codeeraser::graph::store;
use codeeraser::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

mod common;
use common::tmp;

/// The shared fixture tree — every rung's habitat in one place.
const TREE: &[(&str, &str)] = &[
    ("src/app.ts", "export {};\n"),
    // R1: extension order + exact + traversal; widget.ts beats
    // widget/index.ts because the order IS the norm, not ambiguity
    ("src/util.ts", "export {};\n"),
    ("src/widget.ts", "export {};\n"),
    ("src/widget/index.ts", "export {};\n"),
    ("shared/x.tsx", "export {};\n"),
    // R2: legacy.ts has no .js twin (rewrite fires); built.js exists
    // on disk (rewrite blocked)
    ("src/legacy.ts", "export {};\n"),
    ("src/built.ts", "export {};\n"),
    ("src/built.js", "// compiled artifact\n"),
    // R3: extends chain (base contributes baseUrl, child the paths;
    // JSONC comment + trailing comma exercised on purpose)
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
    // R4: one clean member, one duplicate-name pair, one member
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
    // R5: declared dependency vs physically vendored vs nothing
    (
        "package.json",
        "{\"name\": \"rootpkg\", \"dependencies\": {\"lodash\": \"^4\"}}\n",
    ),
    ("node_modules/leftover/index.js", "// vendored\n"),
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

#[test]
fn ts_rungs_resolve_and_refuse() {
    let dir = tmp("ladder-ts");
    let (files, configs) = materialize(&dir, TREE);
    let scope = Scope {
        files: &files,
        configs: &configs,
        root: &dir,
    };
    let cases: &[(&str, Outcome)] = &[
        ("./util", ok("src/util.ts", 1)),
        ("./widget", ok("src/widget.ts", 1)),
        ("./util.ts", ok("src/util.ts", 1)),
        ("../shared/x", ok("shared/x.tsx", 1)),
        ("./legacy.js", ok("src/legacy.ts", 2)),
        ("./built.js", Outcome::Unresolved(Reason::OutOfScope)),
        ("@app/util", ok("src/util.ts", 3)),
        ("leaf", ok("lib/leaf.ts", 3)),
        ("@dup/thing", Outcome::Unresolved(Reason::AmbiguousPaths)),
        ("pkga", ok("packages/pkga/src/index.ts", 4)),
        ("pkga/sub", ok("packages/pkga/src/sub.ts", 4)),
        ("dup-pkg", Outcome::Unresolved(Reason::AmbiguousWorkspace)),
        ("pkgb", Outcome::Unresolved(Reason::AmbiguousExports)),
        ("lodash", Outcome::External { rung: 5 }),
        ("leftover", Outcome::External { rung: 5 }),
        ("unknown-pkg", Outcome::Unresolved(Reason::OutOfScope)),
        ("./missing", Outcome::Unresolved(Reason::OutOfScope)),
    ];
    for (spec, want) in cases {
        let got = ladder::resolve(Lang::TypeScript, "src/app.ts", spec, &scope);
        assert_eq!(got, *want, "spec {spec:?}");
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
    let (files, configs) = materialize(&dir, &[("m.py", "import os\n")]);
    let scope = Scope {
        files: &files,
        configs: &configs,
        root: &dir,
    };
    for lang in [Lang::Python, Lang::Rust, Lang::Go, Lang::Markdown] {
        assert_eq!(
            ladder::resolve(lang, "m.py", "os", &scope),
            Outcome::Unresolved(Reason::Unsupported),
            "{lang:?}"
        );
    }
}
