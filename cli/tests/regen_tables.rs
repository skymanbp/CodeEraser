//! Reproducible regeneration checks for the three machine-generated
//! stdlib tables (M5-close audit D8: the generators lived only in a
//! session scratchpad, so a toolchain bump had no in-repo drift
//! detector). Each test re-derives its table from the SAME recorded
//! pipeline the header of the table cites and asserts set equality —
//! drift prints the exact additions/removals to splice. All three
//! are #[ignore]: they shell out to machine toolchains (ghc-pkg, go,
//! python) that CI does not install for the Rust job.
//!
//!   cargo test --test regen_tables -- --ignored --nocapture

use std::collections::BTreeSet;
use std::process::Command;

fn run(tool: &str, args: &[&str]) -> String {
    let out = Command::new(tool)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("{tool} must be on PATH for this regen check: {e}"));
    assert!(out.status.success(), "{tool} {args:?} failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn diff(name: &str, frozen: &BTreeSet<String>, fresh: &BTreeSet<String>) {
    let added: Vec<_> = fresh.difference(frozen).collect();
    let removed: Vec<_> = frozen.difference(fresh).collect();
    assert!(
        added.is_empty() && removed.is_empty(),
        "{name} drifted from the toolchain:\n  toolchain has, table lacks: {added:?}\n  \
         table has, toolchain lacks: {removed:?}\n  splice these and re-run"
    );
}

/// Frozen-table tail shared by every check: parse the whitespace
/// table, diff against the fresh derivation (the go/py twins were
/// the ratchet's catch in this file).
fn check_against(name: &str, table: &str, fresh: BTreeSet<String>) {
    let frozen: BTreeSet<String> = table.split_whitespace().map(str::to_string).collect();
    diff(name, &frozen, &fresh);
}

/// `go list std` minus internal and vendor packages — Go's rule is
/// per SEGMENT (`log/internal` is as unimportable as `internal/x`;
/// the first cut of this filter missed the trailing-segment form and
/// flagged the CORRECT frozen table as drifted).
#[test]
#[ignore]
fn go_std_table_matches_the_toolchain() {
    let fresh = run("go", &["list", "std"])
        .split_whitespace()
        .filter(|p| !p.split('/').any(|s| s == "internal" || s == "vendor"))
        .map(str::to_string)
        .collect();
    check_against("go STD", codeeraser::graph::ladder::go::STD, fresh);
}

/// `sys.stdlib_module_names`, public names only — the py.rs table's
/// recorded pipeline.
#[test]
#[ignore]
fn py_stdlib_table_matches_the_toolchain() {
    let script = "import sys\nprint('\\n'.join(sorted(sys.stdlib_module_names)))";
    let fresh = run("python", &["-c", script])
        .split_whitespace()
        .filter(|m| !m.starts_with('_'))
        .map(str::to_string)
        .collect();
    check_against(
        "python STDLIB",
        codeeraser::graph::ladder::py::STDLIB,
        fresh,
    );
}

/// The global package db's exposed modules per package — hs_boot.rs's
/// recorded pipeline (`ghc-pkg list --global` + per-package `field
/// exposed,exposed-modules`; re-export entries keep their EXPOSED
/// name, `from pkg:Orig` annotations skipped).
#[test]
#[ignore]
fn hs_boot_table_matches_the_toolchain() {
    let pkgs = run("ghc-pkg", &["list", "--global", "--simple-output"]);
    let mut fresh_names: BTreeSet<String> = BTreeSet::new();
    let mut fresh_mods: BTreeSet<String> = BTreeSet::new();
    for spec in pkgs.split_whitespace() {
        let name = spec
            .rfind('-')
            .filter(|&i| {
                spec[i + 1..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
            })
            .map_or(spec, |i| &spec[..i]);
        if run("ghc-pkg", &["field", spec, "exposed"]).contains("False") {
            continue;
        }
        let mods = exposed_modules(&run("ghc-pkg", &["field", spec, "exposed-modules"]));
        if mods.is_empty() {
            continue;
        }
        fresh_names.insert(name.to_string());
        fresh_mods.extend(mods.into_iter().map(|m| format!("{name}:{m}")));
    }
    let mut frozen_names = BTreeSet::new();
    let mut frozen_mods = BTreeSet::new();
    for (pkg, mods) in codeeraser::graph::ladder::hs_boot::BOOT {
        frozen_names.insert(pkg.to_string());
        frozen_mods.extend(mods.split_whitespace().map(|m| format!("{pkg}:{m}")));
    }
    diff("hs_boot packages", &frozen_names, &fresh_names);
    diff("hs_boot modules", &frozen_mods, &fresh_mods);
}

/// The v2 token grammar: entries split on commas/whitespace, a
/// re-export reads `Name from pkg-id:Orig` — keep Name, skip the
/// annotation pair, refuse any token still holding ':'.
fn exposed_modules(field: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for tok in field.replace("exposed-modules:", " ").split_whitespace() {
        let tok = tok.trim_end_matches(',');
        if skip {
            skip = false;
            continue;
        }
        if tok == "from" {
            skip = true;
            continue;
        }
        if tok.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && !tok.contains(':') {
            out.push(tok.to_string());
        }
    }
    out
}
