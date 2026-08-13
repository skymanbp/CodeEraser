//! The frozen-universe instrument skeleton — ONE binding for the
//! slice (eval_graph.rs) and t3 (eval_t3_universe.rs) families: the
//! pinned-tree walker, the nine-key doc envelope, the gate opening
//! and the working-tree drift walk. Extracted when the repo's own
//! ratchet caught the second family re-instantiating the first's
//! skeleton token for token (the twelfth bite) — each family keeps
//! only its row/summary semantics.

use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::classify_path;

/// One walked file: repo-relative path, lang code, and the text the
/// detectors see (utf8-lossy of the git blob).
pub type WalkedFile = (String, &'static str, String);

/// The doc stem of one corpus in a family: "graph-slice-zod" /
/// "graph-slice" (self). Shared by generators and cross-family
/// lookups so file naming can never fork.
pub fn doc_stem(family: &str, name: &Option<String>) -> String {
    match name {
        Some(n) => format!("{family}-{n}"),
        None => family.into(),
    }
}

/// The instrument walk of one pinned tree in the CE_SLICE_REPO-
/// selected corpus — the single-corpus generators' entry.
pub fn walk_tree(tip: &str) -> (Vec<WalkedFile>, BTreeMap<&'static str, u64>) {
    let repo = std::env::var("CE_SLICE_REPO").ok();
    walk_tree_in(repo.as_deref(), tip)
}

/// The walk with an explicit repository — the multi-corpus pool walk
/// reads several pinned corpora inside one process and cannot ride
/// the env var (the graph-sample precedent). ONE walker for every
/// universe family (a second walk could freeze two different trees
/// under one tip).
pub fn walk_tree_in(
    repo: Option<&str>,
    tip: &str,
) -> (Vec<WalkedFile>, BTreeMap<&'static str, u64>) {
    let mut files = Vec::new();
    let mut excluded: BTreeMap<&'static str, u64> = BTreeMap::new();
    // -z: NUL-terminated, unquoted — non-ASCII paths must not arrive
    // shell-escaped (core.quotePath would corrupt them). --full-tree:
    // the tests run with cwd cli/, and without it ls-tree emits
    // cwd-relative paths while `show rev:path` resolves from the repo
    // root (the M5-1c-ii lesson, recurred on the slice's first run).
    let listing = super::git_in(
        repo,
        &["ls-tree", "-r", "--full-tree", "--name-only", "-z", tip],
    );
    for path in listing.split('\0').filter(|p| !p.is_empty()) {
        match classify_path(path) {
            Ok(code) => {
                let text = super::git_in(repo, &["show", &format!("{tip}:{path}")]);
                files.push((path.to_string(), code, text));
            }
            Err(category) => *excluded.entry(category).or_insert(0) += 1,
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    (files, excluded)
}

/// The parts one instrument derives its own way; the envelope around
/// them is shared.
pub struct UniverseParts {
    pub constants: Value,
    pub summary: Value,
    pub excluded: BTreeMap<&'static str, u64>,
    pub files: Vec<Value>,
}

/// Rows + envelope parts from one walk. The per-family inputs are the
/// row builder, the frozen constants and the scorer; the walk→rows→
/// summary assembly is shared shape (the ratchet caught it aligned
/// across both generators).
pub fn universe_parts(
    walked: &[WalkedFile],
    excluded: BTreeMap<&'static str, u64>,
    row: impl Fn(&str, &str, &str) -> Value,
    constants: Value,
    summarize: fn(&[Value]) -> Value,
) -> UniverseParts {
    let files: Vec<Value> = walked.iter().map(|(p, c, t)| row(p, c, t)).collect();
    UniverseParts {
        constants,
        summary: summarize(&files),
        excluded,
        files,
    }
}

/// Assemble one frozen-universe doc — nine keys, one binding (the doc
/// shape is a cross-family contract: the t3 gate anchors path→sha256
/// straight into its slice sibling).
pub fn universe_doc(
    schema: &str,
    method: &str,
    name: &Option<String>,
    tip: &str,
    p: UniverseParts,
) -> Value {
    json!({
        "schema": schema,
        "corpus": {"name": name, "tip": tip},
        "scope": {"extensions": super::SCOPE_EXTS, "excludes": super::SCOPE_EXCLUDES},
        "constants": p.constants,
        "generated_from": super::generated_from(),
        "method": method,
        "summary": p.summary,
        "excluded": p.excluded,
        "files": p.files,
    })
}

/// Write a frozen-universe doc to its family-derived path.
pub fn write_universe(family: &str, name: &Option<String>, doc: &Value) {
    let path = super::eval_doc(&doc_stem(family, name));
    super::write_doc(&path, doc, &format!("{path} written"));
}

/// The shared opening of every universe gate: iterate the frozen docs
/// of one family (set anchored to FROZEN_CORPORA) through a per-doc
/// check.
pub fn each_frozen_doc(family: &str, mut check: impl FnMut(&str, &Value)) {
    for path in super::assert_frozen_corpus_set(family) {
        let doc = super::load(&path);
        check(&path, &doc);
    }
}

/// The frozen-doc envelope every per-corpus universe doc carries:
/// summary re-derived by the family's own scorer (G1), frozen
/// constants, frozen scope, well-formed exclusion ledger, pinned
/// full-OID tip, embedded corpus name matching the file name, rows
/// sorted and duplicate-free. Returns the corpus name.
pub fn assert_doc_envelope(
    path: &str,
    doc: &Value,
    family: &str,
    scorer: fn(&[Value]) -> Value,
    constants: fn() -> Value,
) -> Option<String> {
    let files = doc["files"].as_array().expect("files");
    assert_eq!(doc["summary"], scorer(files), "{path}: summary drifted");
    assert_eq!(doc["constants"], constants(), "{path}: constants drifted");
    assert_eq!(
        doc["scope"]["extensions"],
        json!(super::SCOPE_EXTS),
        "{path}"
    );
    assert_eq!(
        doc["scope"]["excludes"],
        json!(super::SCOPE_EXCLUDES),
        "{path}"
    );
    for (category, n) in doc["excluded"].as_object().expect("excluded") {
        assert!(
            ["excluded_prefix", "variant_extension", "other_extension"]
                .contains(&category.as_str())
                && n.as_u64().is_some_and(|v| v > 0),
            "{path}: malformed excluded row {category}"
        );
    }
    let tip = doc["corpus"]["tip"].as_str().expect("tip");
    assert!(
        tip.len() == 40 && tip.chars().all(|c| c.is_ascii_hexdigit()),
        "{path}: tip is not a pinned full OID"
    );
    let name = doc["corpus"]["name"].as_str().map(str::to_string);
    assert_eq!(
        name,
        super::doc_suffix(path, family),
        "{path}: embedded corpus name does not match the file name"
    );
    for pair in files.windows(2) {
        assert!(
            pair[0]["path"].as_str() < pair[1]["path"].as_str(),
            "{path}: rows unsorted or duplicated"
        );
    }
    name
}

/// Sum of a JSON object's u64 values — the map-total idiom every
/// per-key summary gate re-derives (site totals, survivor strata).
pub fn sum_obj(v: &Value) -> u64 {
    v.as_object()
        .expect("object")
        .values()
        .map(|x| x.as_u64().expect("u64"))
        .sum()
}

/// The working-tree drift walk both self gates share: every frozen
/// row whose working-tree file still carries the frozen sha256 is
/// handed to `check` as (row, path, lang code, text); returns how
/// many verified. Content-changed files are skipped — that is
/// editing, not drift; a semantics change re-pins the tip and
/// re-freezes.
pub fn each_frozen_match(doc: &Value, mut check: impl FnMut(&Value, &str, &str, &str)) -> usize {
    let mut verified = 0;
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let Ok(bytes) = std::fs::read(format!("../{path}")) else {
            continue;
        };
        // the frozen identity is git-blob text (LF); the Windows CI
        // runner checks out with autocrlf=true, so normalize before
        // comparing — first CI run of the slice gate was 0-row
        // vacuous on windows-latest for exactly this reason
        let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
        if super::content_sha(&text) != row["sha256"].as_str().expect("sha256") {
            continue;
        }
        check(row, path, row["lang"].as_str().expect("lang"), &text);
        verified += 1;
    }
    verified
}
