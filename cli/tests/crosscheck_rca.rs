//! M1 acceptance harness: full-file Rust cyclomatic cross-check against
//! rust-code-analysis-cli via its JSON output path (the only reliable
//! machine-readable mode on Windows — see crosscheck DIVERGENCES.md).
//!
//! Ignored by default: needs `rust-code-analysis-cli` (pinned 0.0.25,
//! `cargo install --locked`) on PATH plus the pinned fixtures.
//! Run:  cargo test --test crosscheck_rca -- --ignored --nocapture
//!
//! RCA JSON semantics (probed 2026-08-07):
//! - `-o <outdir>` joins outdir with the input's RELATIVE path, giving
//!   `<outdir>/<rel>.json`; drive-absolute inputs fail silently. So both
//!   paths here are relative, with cwd pinned to the repo root.
//! - a space's `cyclomatic.sum` aggregates ALL descendant spaces; a
//!   function's own CC is `sum - Σ(direct child space sums)`
//!   (ban.rs `check`: 29 - 8 = 21 = ce, hand-verified).

use codeeraser::scan::{functions, lang::Lang, metrics, spec};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// (start_line, end_line) → (name, own cyclomatic complexity)
type FnMap = BTreeMap<(u64, u64), (String, u32)>;

const FIXTURES: &[&str] = &[
    "contracts/fixtures/crosscheck/rust/crates__ignore__src__walk.rs",
    "contracts/fixtures/crosscheck/rust/crates__printer__src__hyperlink__mod.rs",
    "contracts/fixtures/crosscheck/rust/crates__printer__src__lib.rs",
    "contracts/fixtures/crosscheck/rust/crates__printer__src__util.rs",
    "contracts/fixtures/crosscheck/rust/crates__regex__src__ban.rs",
];

/// Divergences attributed and retained in DIVERGENCES.md — must stay in
/// sync with that file. Format: (fixture, start_line, rca_cc, ce_cc).
const ATTRIBUTED: &[(&str, u64, u32, u32)] = &[];

const OUT_DIR: &str = "target/rca-harness";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

/// The own-CC extraction rule was probed against 0.0.25 specifically;
/// refuse to compare against a drifted tool version.
fn assert_rca_version() {
    let out = Command::new("rust-code-analysis-cli")
        .arg("--version")
        .output()
        .expect("rust-code-analysis-cli not runnable (install: cargo install --locked rust-code-analysis-cli)");
    let v = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        v.contains("0.0.25"),
        "RCA version drifted from the SOURCES.md pin (0.0.25): {v}"
    );
}

fn rca_fns(root: &Path, rel: &str) -> FnMap {
    // RCA rejects a missing outdir ("must be a directory") but creates
    // the nested <rel> subdirs itself — pre-create only the top level.
    std::fs::create_dir_all(root.join(OUT_DIR)).expect("create outdir");
    let json_path = root.join(OUT_DIR).join(format!("{rel}.json"));
    // stale output would mask a silent RCA failure — remove first
    let _ = std::fs::remove_file(&json_path);
    let status = Command::new("rust-code-analysis-cli")
        .current_dir(root)
        .args(["-m", "-O", "json", "-o", OUT_DIR, "-p", rel])
        .status()
        .expect("spawn rust-code-analysis-cli");
    assert!(status.success(), "RCA exited nonzero for {rel}");
    let text = std::fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("RCA produced no output {}: {e}", json_path.display()));
    let unit: Value = serde_json::from_str(&text).expect("parse RCA JSON");
    let mut out = FnMap::new();
    collect_rca(&unit, &mut out);
    out
}

/// Recursively record every `function` space with its own (non-aggregated)
/// cyclomatic value. Children of any space kind are subtracted: `sum`
/// aggregates through struct/impl/mod spaces alike.
fn collect_rca(space: &Value, out: &mut FnMap) {
    let no_children = Vec::new();
    let children = space["spaces"].as_array().unwrap_or(&no_children);
    if space["kind"].as_str() == Some("function") {
        let sum = space["metrics"]["cyclomatic"]["sum"]
            .as_f64()
            .expect("cyclomatic.sum");
        let child_sum: f64 = children
            .iter()
            .filter_map(|c| c["metrics"]["cyclomatic"]["sum"].as_f64())
            .sum();
        let key = (
            space["start_line"].as_u64().expect("start_line"),
            space["end_line"].as_u64().expect("end_line"),
        );
        let name = space["name"].as_str().unwrap_or("?").to_string();
        out.insert(key, (name, (sum - child_sum).round() as u32));
    }
    for c in children {
        collect_rca(c, out);
    }
}

fn ce_fns(root: &Path, rel: &str) -> FnMap {
    let src = std::fs::read(root.join(rel)).expect("read fixture");
    let sp = spec::spec(Lang::Rust);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&Lang::Rust.grammar().expect("grammar"))
        .expect("set_language");
    let tree = parser.parse(&src, None).expect("parse fixture");
    functions::extract(tree.root_node(), &src, sp)
        .into_iter()
        .map(|u| {
            let cc = metrics::cyclo::measure(u.node, &src, sp);
            ((u.start_line as u64, u.end_line as u64), (u.name, cc))
        })
        .collect()
}

#[derive(Default)]
struct Tally {
    units: usize,
    agree: usize,
    attributed: usize,
    problems: Vec<String>,
}

fn is_attributed(rel: &str, start: u64, rca_cc: u32, ce_cc: u32) -> bool {
    ATTRIBUTED
        .iter()
        .any(|&(f, l, r, c)| rel.ends_with(f) && l == start && r == rca_cc && c == ce_cc)
}

/// Set-diff one fixture: exact unit alignment both ways, then values.
fn diff_file(rel: &str, rca: &FnMap, ce: &FnMap, t: &mut Tally) {
    for (k, (name, _)) in rca {
        if !ce.contains_key(k) {
            t.problems
                .push(format!("{rel}: only-RCA {name} @{}-{}", k.0, k.1));
        }
    }
    for (k, (name, _)) in ce {
        if !rca.contains_key(k) {
            t.problems
                .push(format!("{rel}: only-ce {name} @{}-{}", k.0, k.1));
        }
    }
    for (k, (name, rca_cc)) in rca {
        let Some((_, ce_cc)) = ce.get(k) else { continue };
        t.units += 1;
        if rca_cc == ce_cc {
            t.agree += 1;
        } else if is_attributed(rel, k.0, *rca_cc, *ce_cc) {
            t.attributed += 1;
        } else {
            t.problems
                .push(format!("{rel}: {name} @{} rca={rca_cc} ce={ce_cc}", k.0));
        }
    }
}

#[test]
#[ignore = "acceptance harness: needs rust-code-analysis-cli on PATH"]
fn rust_cc_full_sweep_vs_rca() {
    assert_rca_version();
    let root = repo_root();
    let mut t = Tally::default();
    for rel in FIXTURES {
        diff_file(rel, &rca_fns(&root, rel), &ce_fns(&root, rel), &mut t);
    }
    println!(
        "Rust CC vs rust-code-analysis 0.0.25: {}/{} agree, {} attributed (DIVERGENCES.md)",
        t.agree, t.units, t.attributed
    );
    assert_eq!(
        t.attributed,
        ATTRIBUTED.len(),
        "attributed-divergence allowlist is stale — sync with DIVERGENCES.md"
    );
    assert!(
        t.problems.is_empty(),
        "unattributed divergences:\n{}",
        t.problems.join("\n")
    );
}
