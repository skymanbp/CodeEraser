//! NOTICE zero-drift gate (M7-P3; the D1-7 premise retired —
//! ast-grep-core never entered the tree, so the obligation is the
//! dependency-license inventory of what we actually distribute). The
//! committed NOTICE must byte-match its regeneration: the Rust
//! section straight from `cargo metadata` over both workspaces (the
//! lockfiles are the truth), the Haskell section from the ce-core
//! EXECUTABLE dependency closure in cabal's plan.json joined against
//! the frozen license table below. The closure is PER-PLATFORM
//! (probed 2026-08-18: Win32 rides in solely via `time` on Windows;
//! the POSIX closures carry no unix counterpart), while NOTICE
//! inventories the union over the three shipped binaries — so a
//! platform-only package absent from this host's closure is emitted
//! from its frozen row. CE_BLESS=1 regenerates.
//!
//! Table provenance: harvested 2026-08-17 from `ghc-pkg field '*'
//! name,version,license` over the GHC 9.14.1 global db and the cabal
//! store db (SPDX ids as recorded there). A closure package without a
//! row is a RED gate naming the package — a new dependency ships only
//! with its license read and frozen here (the hs_boot lesson:
//! machine-generated tables get re-probed, never guessed).

mod common;

use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

const HS_LICENSES: &[(&str, &str, &str)] = &[
    ("OneTuple", "0.4.3", "BSD-3-Clause"),
    ("QuickCheck", "2.18.0.0", "BSD-3-Clause"),
    ("StateVar", "1.2.2", "BSD-3-Clause"),
    ("Win32", "2.14.2.1", "BSD-3-Clause"),
    ("aeson", "2.3.1.0", "BSD-3-Clause"),
    ("array", "0.5.8.0", "BSD-3-Clause"),
    ("assoc", "1.1.1", "BSD-3-Clause"),
    ("base", "4.22.0.0", "BSD-3-Clause"),
    ("base-orphans", "0.9.4", "MIT"),
    ("bifunctors", "5.6.3", "BSD-3-Clause"),
    ("binary", "0.8.9.3", "BSD-3-Clause"),
    ("bytestring", "0.12.2.0", "BSD-3-Clause"),
    ("character-ps", "0.1", "BSD-3-Clause"),
    ("comonad", "5.0.10", "BSD-3-Clause"),
    ("containers", "0.8", "BSD-3-Clause"),
    ("contravariant", "1.5.6", "BSD-3-Clause"),
    ("data-fix", "0.3.4", "BSD-3-Clause"),
    ("deepseq", "1.5.1.0", "BSD-3-Clause"),
    ("directory", "1.3.10.0", "BSD-3-Clause"),
    ("distributive", "0.6.3", "BSD-3-Clause"),
    ("dlist", "1.0", "BSD-3-Clause"),
    ("exceptions", "0.10.11", "BSD-3-Clause"),
    ("file-io", "0.1.5", "BSD-3-Clause"),
    ("filepath", "1.5.4.0", "BSD-3-Clause"),
    ("ghc-boot-th", "9.14.1", "BSD-3-Clause"),
    ("ghc-internal", "9.1401.0", "BSD-3-Clause"),
    ("ghc-prim", "0.13.1", "BSD-3-Clause"),
    ("hashable", "1.5.1.0", "BSD-3-Clause"),
    ("indexed-traversable", "0.1.5", "BSD-2-Clause"),
    ("indexed-traversable-instances", "0.1.2.1", "BSD-2-Clause"),
    ("integer-conversion", "0.1.1", "BSD-3-Clause"),
    ("integer-logarithms", "1.0.5", "MIT"),
    ("mtl", "2.3.1", "BSD-3-Clause"),
    ("network-uri", "2.6.4.2", "BSD-3-Clause"),
    ("os-string", "2.0.8", "BSD-3-Clause"),
    ("parsec", "3.1.18.0", "BSD-2-Clause"),
    ("pretty", "1.1.3.6", "BSD-3-Clause"),
    ("primitive", "0.9.1.0", "BSD-3-Clause"),
    ("process", "1.6.26.1", "BSD-3-Clause"),
    ("random", "1.3.1", "BSD-3-Clause"),
    ("rts", "1.0.3", "BSD-3-Clause"),
    ("scientific", "0.3.8.1", "BSD-3-Clause"),
    ("semialign", "1.4", "BSD-3-Clause"),
    ("semigroupoids", "6.0.2", "BSD-2-Clause"),
    ("splitmix", "0.1.3.2", "BSD-3-Clause"),
    ("stm", "2.5.3.1", "BSD-3-Clause"),
    ("strict", "0.5.1", "BSD-3-Clause"),
    ("tagged", "0.8.10", "BSD-3-Clause"),
    ("template-haskell", "2.24.0.0", "BSD-3-Clause"),
    ("text", "2.1.3", "BSD-2-Clause"),
    ("text-iso8601", "0.2.0.0", "BSD-3-Clause"),
    ("text-short", "0.1.6.1", "BSD-3-Clause"),
    ("th-abstraction", "0.7.2.0", "ISC"),
    ("th-compat", "0.1.7", "BSD-3-Clause"),
    ("these", "1.2.1", "BSD-3-Clause"),
    ("time", "1.15", "BSD-2-Clause"),
    ("time-compat", "1.9.9", "BSD-3-Clause"),
    ("transformers", "0.6.1.2", "BSD-3-Clause"),
    ("transformers-compat", "0.8", "BSD-3-Clause"),
    ("unordered-containers", "0.2.21", "BSD-3-Clause"),
    ("uuid-types", "1.0.6.1", "BSD-3-Clause"),
    ("vector", "0.13.2.0", "BSD-3-Clause"),
    ("vector-stream", "0.1.0.1", "BSD-3-Clause"),
    ("witherable", "0.5", "BSD-3-Clause"),
];

/// Packages that enter the executable closure on SOME platforms only.
/// Probed 2026-08-18 against both CI closures: Win32 is referenced by
/// exactly one parent (`time`, Windows-conditional dep), and the Linux
/// regeneration came out exactly one row smaller with no unix package
/// anywhere in its closure. A platform where the package is absent
/// still emits its frozen HS_LICENSES row, so every platform
/// regenerates the same union NOTICE.
const HS_PLATFORM_ONLY: &[&str] = &["Win32"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

/// Every third-party (name, version, license) in one workspace's
/// resolved graph — the lockfile is the population, `license` the
/// manifest's own claim (a crate without one is named, not guessed).
fn cargo_rows(manifest: &Path, rows: &mut BTreeSet<(String, String, String)>) {
    let out = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(manifest)
        .output()
        .expect("cargo metadata");
    assert!(
        out.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("metadata json");
    for p in v["packages"].as_array().expect("packages") {
        let name = p["name"].as_str().expect("name");
        if name == "codeeraser" || name == "ce-gui" {
            continue; // our own workspace members carry the LICENSE file
        }
        let lic = p["license"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| "(no SPDX id — see the crate's license file)".into());
        rows.insert((
            name.to_string(),
            p["version"].as_str().expect("version").to_string(),
            lic,
        ));
    }
}

/// The ce-core executable's transitive package closure from cabal's
/// plan.json (present because CI builds Haskell before the Rust
/// tests — the same ordering the wire goldens already rely on).
fn hs_closure(root: &Path) -> BTreeSet<(String, String)> {
    let plan: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("core/dist-newstyle/cache/plan.json"))
            .expect("plan.json — run `cabal build all` first"),
    )
    .expect("plan json");
    let units = plan["install-plan"].as_array().expect("install-plan");
    let by_id: HashMap<&str, &Value> = units
        .iter()
        .map(|u| (u["id"].as_str().expect("id"), u))
        .collect();
    let exe = units
        .iter()
        .find(|u| u["pkg-name"] == "ce-core" && u["component-name"] == "exe:ce-core")
        .expect("exe:ce-core unit");
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack = vec![exe["id"].as_str().expect("id")];
    while let Some(id) = stack.pop() {
        // A dep id missing from install-plan would silently shrink the
        // legal inventory — same failure class the coverage gate below
        // reddens for, so it reddens BY NAME too (0 hits on both CI
        // platforms as of 2026-08-18).
        let u = by_id
            .get(id)
            .unwrap_or_else(|| panic!("dep id {id} not in install-plan"));
        if !seen.insert(id) {
            continue;
        }
        for d in u["depends"].as_array().into_iter().flatten() {
            stack.push(d.as_str().expect("dep"));
        }
        for c in u["components"].as_object().into_iter().flatten() {
            for d in c.1["depends"].as_array().into_iter().flatten() {
                stack.push(d.as_str().expect("dep"));
            }
        }
    }
    seen.iter()
        .filter_map(|id| {
            let u = by_id[id];
            let n = u["pkg-name"].as_str()?;
            (n != "ce-core").then(|| {
                (
                    n.to_string(),
                    u["pkg-version"].as_str().unwrap().to_string(),
                )
            })
        })
        .collect()
}

/// Closure joined against the frozen table — a missing row is red BY
/// NAME so a new dependency cannot ship license-unread. Platform-only
/// packages absent from THIS host's closure join from their frozen
/// rows, so the emitted set is the union over shipped platforms.
fn hs_rows(root: &Path) -> Vec<(String, String, String)> {
    let mut closure = hs_closure(root);
    for name in HS_PLATFORM_ONLY {
        if !closure.iter().any(|(n, _)| n == name) {
            let rows: Vec<_> = HS_LICENSES.iter().filter(|(n, _, _)| n == name).collect();
            assert_eq!(
                rows.len(),
                1,
                "platform-only {name}: exactly one frozen row"
            );
            closure.insert((rows[0].0.to_string(), rows[0].1.to_string()));
        }
    }
    closure
        .into_iter()
        .map(|(n, v)| {
            let lic = HS_LICENSES
                .iter()
                .find(|(tn, tv, _)| *tn == n && *tv == v)
                .unwrap_or_else(|| {
                    panic!(
                        "no frozen license row for {n} {v} — probe ghc-pkg and extend HS_LICENSES"
                    )
                })
                .2;
            (n, v, lic.to_string())
        })
        .collect()
}

fn notice_text(root: &Path) -> String {
    let mut rust = BTreeSet::new();
    cargo_rows(&root.join("cli/Cargo.toml"), &mut rust);
    cargo_rows(&root.join("gui/src-tauri/Cargo.toml"), &mut rust);
    let mut s = String::from(
        "CodeEraser NOTICE\n=================\n\n\
         Copyright 2026 skymanbp\n\
         Licensed under the Apache License, Version 2.0 (see LICENSE).\n\n\
         This distribution bundles the third-party components below.\n\
         Regenerate with: CE_BLESS=1 cargo test --test notice_gate\n",
    );
    let hs = hs_rows(root);
    s += "\n== ce / GUI (Rust crates; cargo metadata over both lockfiles) ==\n\n";
    for (n, v, l) in &rust {
        s += &format!("{n} {v} — {l}\n");
    }
    s += "\n== ce-core (Haskell packages; executable dependency closure, \
          union over shipped platforms) ==\n\n";
    for (n, v, l) in &hs {
        s += &format!("{n} {v} — {l}\n");
    }
    s
}

#[test]
fn notice_matches_its_regeneration() {
    let root = repo_root();
    let text = notice_text(&root);
    common::assert_matches_golden(&text, &root.join("NOTICE"));
}
