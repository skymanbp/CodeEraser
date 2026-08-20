//! Corpus resolution for the commit-slice instrument family: which
//! repository is under measurement and where each frozen doc family
//! lives. Split from mod.rs when the M5-1 external-validation
//! machinery pushed it past the file budget; the window machinery
//! (tip/base pinning, Corpus struct, SELF_UNIVERSE_TIP) retired with
//! the one-shot generators that read it (git history, 0c7c936 wave).

/// The active corpus name: None = this repository (the frozen self
/// slice). CE_SLICE_REPO (see colordiff::git_run) retargets at an
/// external corpus and then requires CE_SLICE_NAME (doc suffix) —
/// an unnamed corpus would overwrite the frozen self docs. The
/// retired CE_SLICE_TIP/BASE knobs are refused out loud: a silent
/// no-op would let a hand-run believe its window was pinned.
pub fn corpus_name() -> Option<String> {
    let get = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    assert!(
        get("CE_SLICE_TIP").is_none() && get("CE_SLICE_BASE").is_none(),
        "CE_SLICE_TIP/BASE retired with the window machinery (git history)"
    );
    let name = get("CE_SLICE_NAME");
    if get("CE_SLICE_REPO").is_none() {
        assert!(
            name.is_none(),
            "CE_SLICE_NAME makes no sense without CE_SLICE_REPO"
        );
        return None;
    }
    assert!(
        name.is_some(),
        "CE_SLICE_REPO needs a non-empty CE_SLICE_NAME: an unnamed \
         corpus would overwrite the frozen self slice"
    );
    name
}

/// Every committed slice doc paired with its `family` sibling. The
/// sibling MUST exist — a slice without its labels/baseline is an
/// unfinished freeze, and a gate that skips it would go silently
/// blind on a whole corpus.
pub fn corpus_doc_pairs(family: &str) -> Vec<(String, String)> {
    let pairs = doc_pairs(family, true);
    assert!(!pairs.is_empty(), "no committed slice docs");
    pairs
}

/// Like corpus_doc_pairs, but a missing sibling is a PENDING corpus,
/// not an error — for families that freeze later than the slice
/// (an L2 doc lands only once the bar passes on that corpus; the
/// requests doc stayed pending on the invention finding until the
/// anchor floor landed, see EVAL-SET). The self corpus sibling must
/// still exist: that bar is frozen.
pub fn corpus_doc_pairs_frozen(family: &str) -> Vec<(String, String)> {
    let pairs = doc_pairs(family, false);
    let self_doc = format!("../contracts/eval/commit-{family}-v1.json");
    assert!(
        pairs.iter().any(|(_, d)| *d == self_doc),
        "self {family} doc missing"
    );
    pairs
}

/// The corpus-name suffix of a frozen doc path for an arbitrary
/// document stem: "…/{stem}-ripgrep-v1.json" → Some("ripgrep"), the
/// self doc "…/{stem}-v1.json" → None. Shared by the commit families
/// (below) and the graph family (eval_graph.rs).
pub fn doc_suffix(path: &str, stem: &str) -> Option<String> {
    let file = path.rsplit('/').next().expect("file name");
    let mid = file
        .strip_prefix(stem)
        .and_then(|s| s.strip_suffix("-v1.json"))
        .unwrap_or_else(|| panic!("{path}: not a {stem} doc"));
    (!mid.is_empty()).then(|| mid.trim_start_matches('-').to_string())
}

/// doc_suffix for the commit families — how a gate iterating EVERY
/// corpus resolves the matching compiled review record (resolving
/// via the active corpus would silently read the wrong one — Codex
/// review C3 follow-up).
pub fn doc_corpus_name(path: &str, family: &str) -> Option<String> {
    doc_suffix(path, &format!("commit-{family}"))
}

/// A frozen-doc gate's prologue: the corpus name parsed from the
/// family doc's path plus both loaded documents (companion doc
/// first) — the shared opening of every per-corpus CI gate.
pub fn gate_docs(
    family: &str,
    companion: &str,
    doc_path: &str,
) -> (Option<String>, serde_json::Value, serde_json::Value) {
    (
        doc_corpus_name(doc_path, family),
        super::load(companion),
        super::load(doc_path),
    )
}

/// Frozen docs under contracts/eval whose file name starts with
/// `prefix` (and ends -v1.json), sorted — the ONE enumeration every
/// doc-family gate consumes (the dedup ratchet caught this loop's
/// third verbatim copy; the throat is the fix, not the third copy).
/// The built ce-core binary path (generators that spawn the product
/// driver need the PATH; link consumers use core_link).
pub fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

/// The live ce-core link every L2-family generator drives.
pub fn core_link() -> codeeraser::corelink::Link {
    codeeraser::corelink::Link::open(&core_bin())
        .expect("ce-core link")
        .0
}

pub fn frozen_docs(prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir("../contracts/eval").expect("eval dir") {
        let file = entry.expect("entry").file_name();
        let file = file.to_string_lossy();
        if file.starts_with(prefix) && file.ends_with("-v1.json") {
            out.push(format!("../contracts/eval/{file}"));
        }
    }
    out.sort();
    out
}

fn doc_pairs(family: &str, required: bool) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for path in frozen_docs("commit-slice") {
        let sibling = path.replace("commit-slice", &format!("commit-{family}"));
        match std::fs::exists(&sibling).expect("probe") {
            true => pairs.push((path, sibling)),
            false => assert!(!required, "{path}: missing {family} sibling {sibling}"),
        }
    }
    pairs
}
