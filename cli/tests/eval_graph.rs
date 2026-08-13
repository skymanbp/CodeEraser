//! M5-2 graph-slice instrument: the frozen SITE universe per corpus
//! (design brief docs/reviews/2026-08-12-m5-2-graph-design.md §5).
//! The universe is sites, not edges — frozen BEFORE any resolver
//! exists, so the resolver can never choose its own precision
//! denominator; the falsification constants (min_per_lang,
//! r0_share_trigger) are written into the doc before any measurement
//! exists.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP):
//!   cargo test --test eval_graph -- --ignored --nocapture

mod eval_support;

use codeeraser::graph::sites::detect;
use eval_support::{
    content_sha, eval_doc, generated_from, git_run, kind_counts, lang_of, load, write_doc,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Frozen graph-universe scope: canonical extensions only (variant
/// suffixes stay out on every corpus — the COMMIT_SCOPE argument:
/// one frozen scope keeps corpora comparable), minus machine-local
/// memory/. The crosscheck fixture islands are deliberately IN scope
/// even though ce.toml excludes them from the product walk: their
/// imports have no in-corpus target, so they are the designed-in
/// negative control (design §5, judge defect D2).
const SCOPE_EXTS: [&str; 5] = ["go", "md", "py", "rs", "ts"];
const SCOPE_EXCLUDES: [&str; 2] = ["memory/", "cli/memory/"];

/// The frozen corpus set — a deleted or renamed slice doc reddens CI
/// instead of blinding it (design G10; Opus review: the first gate
/// held no per-corpus anchor at all). Sorted; None = the self doc.
const FROZEN_CORPORA: [Option<&str>; 5] = [
    None,
    Some("cobra"),
    Some("requests"),
    Some("ripgrep"),
    Some("zod"),
];

/// Pre-registered falsification constants (design §5), one binding
/// for generator AND gate — duplicated literals were the exact
/// throat-drift shape this file polices elsewhere (Opus review).
fn constants() -> Value {
    json!({"min_per_lang": 15, "r0_share_trigger": 0.80})
}

/// The self universe: pinned at the commit holding the CURRENT
/// detector, so regeneration is reproducible regardless of later
/// history. A detector change bumps this pin and re-freezes the
/// slice (design RG3 — first fired by the M5-2b-iii hardening batch,
/// which replaced the 2b-i pin eb5fe24).
const GRAPH_SELF_TIP: &str = "60f73e3bea7681721a2f572e64788948a17830f6";

/// (corpus name, pinned tree OID). Self unless CE_SLICE_REPO points
/// elsewhere; external corpora must name themselves and pin their
/// tip (rev-parsed to a full OID — a movable rev would make the doc
/// unreproducible).
fn graph_corpus() -> (Option<String>, String) {
    let get = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    if std::env::var("CE_SLICE_REPO").is_err() {
        return (None, GRAPH_SELF_TIP.into());
    }
    let name = get("CE_GRAPH_NAME").expect("CE_SLICE_REPO needs CE_GRAPH_NAME");
    let tip = get("CE_GRAPH_TIP").expect("CE_SLICE_REPO needs CE_GRAPH_TIP (pinned universe)");
    let full = git_run(
        &["rev-parse", "--verify", &format!("{tip}^{{commit}}")],
        false,
    );
    (Some(name), full.trim().to_string())
}

/// Scope test for one tree path: Ok(lang code) or the itemized
/// exclusion category the doc reports.
fn classify_path(path: &str) -> Result<&'static str, &'static str> {
    if SCOPE_EXCLUDES.iter().any(|p| path.starts_with(p)) {
        return Err("excluded_prefix");
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return Err("other_extension");
    };
    if stem.is_empty() {
        return Err("other_extension"); // dotfiles (.gitignore, …)
    }
    match ext {
        "go" => Ok("go"),
        "md" => Ok("md"),
        "py" => Ok("py"),
        "rs" => Ok("rs"),
        "ts" => Ok("ts"),
        "tsx" | "mts" | "cts" | "markdown" => Err("variant_extension"),
        _ => Err("other_extension"),
    }
}

/// One inventory row: content identity (sha256 of the utf8-lossy
/// text the detector saw — the instrument's identity, not git's blob
/// id) plus per-kind site counts.
fn file_row(tip: &str, path: &str, code: &'static str) -> Value {
    let content = git_run(&["show", &format!("{tip}:{path}")], false);
    let kinds = kind_counts(&detect(&content, lang_of(code)));
    json!({"path": path, "sha256": content_sha(&content), "lang": code, "sites": kinds})
}

/// Re-derivable from the rows alone — the CI gate re-runs this exact
/// function (the G1 discipline: generator and gate share one scorer).
fn summarize(files: &[Value]) -> Value {
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = 0;
    for f in files {
        for (kind, n) in f["sites"].as_object().expect("sites") {
            let n = n.as_u64().expect("count");
            let key = format!("{}/{kind}", f["lang"].as_str().expect("lang"));
            *by.entry(key).or_insert(0) += n;
            total += n;
        }
    }
    json!({"files": files.len(), "total_sites": total, "sites_by": by})
}

#[test]
#[ignore] // needs the corpus repository (git show at the pinned tip)
fn generate_graph_slice() {
    let (name, tip) = graph_corpus();
    let mut files = Vec::new();
    let mut excluded: BTreeMap<&str, u64> = BTreeMap::new();
    // -z: NUL-terminated, unquoted — non-ASCII paths must not arrive
    // shell-escaped (core.quotePath would corrupt them). --full-tree:
    // the test runs with cwd cli/, and without it ls-tree emits
    // cwd-relative paths while `show rev:path` resolves from the repo
    // root (the M5-1c-ii lesson, recurred here on first run).
    let listing = git_run(
        &["ls-tree", "-r", "--full-tree", "--name-only", "-z", &tip],
        false,
    );
    for path in listing.split('\0').filter(|p| !p.is_empty()) {
        match classify_path(path) {
            Ok(code) => files.push(file_row(&tip, path, code)),
            Err(category) => *excluded.entry(category).or_insert(0) += 1,
        }
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let doc = json!({
        "schema": "ce.eval-graph-slice/1.0.0",
        "corpus": {"name": name, "tip": tip},
        "scope": {"extensions": SCOPE_EXTS, "excludes": SCOPE_EXCLUDES},
        "constants": constants(),
        "generated_from": generated_from(),
        "method": "site universe of the pinned tree: every in-scope file \
                   (canonical extensions minus machine-local memory/; the \
                   crosscheck islands deliberately in scope as the negative \
                   control), inventoried with the sha256 of the text the \
                   detector saw and its resolution-free per-kind site counts \
                   (graph::sites — grammar kind tables only, no path ever \
                   consulted). Frozen before any resolver exists; the \
                   falsification constants are pre-registered here, before \
                   any measurement.",
        "summary": summarize(&files),
        "excluded": excluded,
        "files": files,
    });
    let stem = match &name {
        Some(n) => format!("graph-slice-{n}"),
        None => "graph-slice".into(),
    };
    let path = eval_doc(&stem);
    write_doc(&path, &doc, &format!("{path} written"));
}

/// CI gate, no git, every frozen slice: the summary re-derives from
/// the rows via the generator's own scorer; constants and scope are
/// the frozen ones; the tip is a pinned full OID; rows are sorted
/// and duplicate-free; the embedded corpus name matches the file
/// name; and the frozen docs jointly cover all five languages with
/// sites (D2-4 — the 2b exit criterion).
#[test]
fn graph_slice_consistent() {
    let docs = eval_support::frozen_docs("graph-slice");
    let mut names: Vec<Option<String>> = docs
        .iter()
        .map(|p| eval_support::doc_suffix(p, "graph-slice"))
        .collect();
    names.sort(); // file-name order interleaves the self doc
    let expected: Vec<Option<String>> = FROZEN_CORPORA
        .iter()
        .map(|n| n.map(str::to_string))
        .collect();
    assert_eq!(names, expected, "frozen corpus set drifted (G10)");
    let mut lang_sites: BTreeMap<String, u64> = BTreeMap::new();
    for path in &docs {
        let doc = load(path);
        check_slice(path, &doc, &mut lang_sites);
    }
    for lang in SCOPE_EXTS {
        assert!(
            lang_sites.get(lang).copied().unwrap_or(0) > 0,
            "no {lang} sites across frozen slices (D2-4)"
        );
    }
}

/// The detector and the frozen self universe must not drift apart
/// silently (design RG3 made a CI fact — Opus review: a spec.rs or
/// md.rs change used to invalidate all five docs with zero red
/// signal). Every frozen row whose working-tree file still carries
/// the frozen sha256 is re-detected: per-kind counts must match, and
/// every detected spec must be a substring of its source line (the
/// 2b exit criterion, previously asserted only on toy fixtures).
/// Content-changed files are skipped — that is editing, not detector
/// drift; a detector change re-pins GRAPH_SELF_TIP and re-freezes.
#[test]
fn self_universe_tracks_detector() {
    let doc = load(&eval_doc("graph-slice"));
    let mut verified = 0;
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let Ok(bytes) = std::fs::read(format!("../{path}")) else {
            continue;
        };
        // the frozen identity is git-blob text (LF); the Windows CI
        // runner checks out with autocrlf=true, so normalize before
        // comparing — first CI run of this gate was 0-row vacuous on
        // windows-latest for exactly this reason
        let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
        if content_sha(&text) != row["sha256"].as_str().expect("sha256") {
            continue;
        }
        let sites = detect(&text, lang_of(row["lang"].as_str().expect("lang")));
        let lines: Vec<&str> = text.lines().collect();
        for s in &sites {
            // the site line is the STATEMENT HEAD: a multi-line TS
            // import carries its full specifier on a later line of
            // the same statement (2c/2d review F1 — 14 frozen zod
            // sites), so the anti-invention property is "spec within
            // the statement window", not "spec on the head line"
            let end = (s.line + 15).min(lines.len());
            assert!(
                lines[s.line - 1..end].iter().any(|l| l.contains(&s.spec)),
                "{path}:{}: spec {:?} not within its statement window",
                s.line,
                s.spec
            );
        }
        assert_eq!(
            row["sites"],
            json!(kind_counts(&sites)),
            "{path}: detector drifted from the frozen universe"
        );
        verified += 1;
    }
    // ordinary churn shrinks the verifiable set between freezes;
    // this floor only guards against the gate going vacuous
    assert!(verified >= 25, "drift gate near-vacuous: {verified} rows");
}

fn check_slice(path: &str, doc: &Value, lang_sites: &mut BTreeMap<String, u64>) {
    let files = doc["files"].as_array().expect("files");
    assert_eq!(doc["summary"], summarize(files), "{path}: summary drifted");
    assert_eq!(doc["constants"], constants(), "{path}: constants drifted");
    for (category, n) in doc["excluded"].as_object().expect("excluded") {
        assert!(
            ["excluded_prefix", "variant_extension", "other_extension"]
                .contains(&category.as_str())
                && n.as_u64().is_some_and(|v| v > 0),
            "{path}: malformed excluded row {category}"
        );
    }
    assert_eq!(doc["scope"]["extensions"], json!(SCOPE_EXTS), "{path}");
    assert_eq!(doc["scope"]["excludes"], json!(SCOPE_EXCLUDES), "{path}");
    let tip = doc["corpus"]["tip"].as_str().expect("tip");
    assert!(
        tip.len() == 40 && tip.chars().all(|c| c.is_ascii_hexdigit()),
        "{path}: tip is not a pinned full OID"
    );
    let name = doc["corpus"]["name"].as_str().map(str::to_string);
    assert_eq!(
        name,
        eval_support::doc_suffix(path, "graph-slice"),
        "{path}: embedded corpus name does not match the file name"
    );
    for pair in files.windows(2) {
        assert!(
            pair[0]["path"].as_str() < pair[1]["path"].as_str(),
            "{path}: rows unsorted or duplicated"
        );
    }
    for (langkind, n) in doc["summary"]["sites_by"].as_object().expect("sites_by") {
        let lang = langkind.split('/').next().expect("lang/kind");
        *lang_sites.entry(lang.to_string()).or_insert(0) += n.as_u64().expect("n");
    }
}
