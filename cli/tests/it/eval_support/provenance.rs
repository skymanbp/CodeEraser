//! Ancestry-gate primitives (G13/T-G13): "sampled before audited
//! before scored" as CHECKED git facts. ONE binding for the graph
//! and dedup provenance gates — these are deliberately the only
//! gates that run git, and a shallow clone refuses loudly instead of
//! passing vacuously (CI checks out fetch-depth: 0 for exactly this).

pub fn require_full_history() {
    let shallow = git_in(Some(".."), &["rev-parse", "--is-shallow-repository"]);
    assert_eq!(
        shallow.trim(),
        "false",
        "shallow clone: ancestry is uncheckable — fetch-depth: 0 (see ci.yml)"
    );
}

/// First commit that introduced `path` (repo-relative).
pub fn intro_commit(path: &str) -> String {
    let log = git_in(Some(".."), &["log", "--reverse", "--format=%H", "--", path]);
    log.lines()
        .next()
        .unwrap_or_else(|| panic!("{path}: never committed"))
        .to_string()
}

/// merge-base --is-ancestor: the exit code IS the answer, so this one
/// call bypasses git_in's success assertion on purpose.
pub fn is_ancestor(ancestor: &str, descendant: &str) -> bool {
    std::process::Command::new("git")
        .args([
            "-C",
            "..",
            "merge-base",
            "--is-ancestor",
            ancestor,
            descendant,
        ])
        .status()
        .expect("git")
        .success()
}

/// --is-ancestor is REFLEXIVE (review F6): two artifacts frozen in
/// one commit would pass the plain form and defeat the very ordering
/// the gate certifies — strictness is the claim, so assert it.
pub fn is_strict_ancestor(ancestor: &str, descendant: &str) -> bool {
    ancestor != descendant && is_ancestor(ancestor, descendant)
}

/// Per-corpus artifact intro commits from a path template — the
/// audit-table list both provenance families build.
pub fn corpus_intros(template: &dyn Fn(&str) -> String, corpora: &[&str]) -> Vec<String> {
    corpora.iter().map(|c| intro_commit(&template(c))).collect()
}

/// One ordering leg: every listed intro strictly descends from the
/// anchor commit.
pub fn assert_all_postdate(anchor: &str, intros: &[String], what: &str) {
    for intro in intros {
        assert!(is_strict_ancestor(anchor, intro), "{what}");
    }
}

/// The scoring leg: every listed doc's generated_from.commit
/// strictly descends from every audit intro — GT froze before any
/// score existed.
pub fn assert_docs_postdate_audits(audits: &[String], stems: &[String], what: &str) {
    for stem in stems {
        let doc = super::load(&super::eval_doc(stem));
        let commit = doc["generated_from"]["commit"].as_str().expect("commit");
        for audit in audits {
            assert!(is_strict_ancestor(audit, commit), "{stem}: {what}");
        }
    }
}

/// The sample→audit→scoring ordering as ONE walk (both provenance
/// families run exactly this pair of legs; the subtree and resolver
/// legs stay with the family): every audit table strictly descends
/// from the sample freeze, every precision doc strictly descends
/// from every audit table.
pub fn assert_audit_scoring_legs(
    sample_path: &str,
    audit_template: &dyn Fn(&str) -> String,
    corpora: &[&str],
    precision_family: &str,
    tag: &str,
) {
    require_full_history();
    let sample = intro_commit(sample_path);
    let audits = corpus_intros(audit_template, corpora);
    assert_all_postdate(
        &sample,
        &audits,
        &format!("audit table does not strictly descend from the sample freeze ({tag})"),
    );
    let stems: Vec<String> = super::FROZEN_CORPORA
        .iter()
        .map(|n| super::doc_stem(precision_family, &n.map(str::to_string)))
        .collect();
    assert_docs_postdate_audits(
        &audits,
        &stems,
        &format!("scored before the audit froze ({tag})"),
    );
}

/// Every file currently under `subtree` must have been introduced
/// STRICTLY after `anchor` — the full-scan blind-window leg: nothing
/// judge-shaped may predate the frozen draw, wherever it landed.
pub fn assert_subtree_postdates(anchor: &str, subtree: &str) {
    let tree = git_in(Some(".."), &["ls-files", "--", subtree]);
    let mut any = false;
    for path in tree.lines().filter(|p| !p.is_empty()) {
        any = true;
        assert!(
            is_strict_ancestor(anchor, &intro_commit(path)),
            "{path}: landed at or before the anchor freeze (G13)"
        );
    }
    assert!(any, "{subtree}: empty subtree makes the gate vacuous");
}

/// Run git in `repo` (None = the enclosing repository), success AND
/// empty stderr asserted — a git warning on the success path is a
/// silently degraded result (the retired slice generators' lesson).
/// Lives here since the 2026-08-24 cleanup: the provenance gates and
/// graph_provenance's ladder-freshness leg are its last consumers.
pub fn git_in(repo: Option<&str>, args: &[&str]) -> String {
    let mut cmd = std::process::Command::new("git");
    if let Some(repo) = repo {
        cmd.arg("-C").arg(repo);
    }
    let out = cmd.args(args).output().expect("git");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "git {args:?}: {stderr}");
    assert!(stderr.trim().is_empty(), "git {args:?} warned: {stderr}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}
