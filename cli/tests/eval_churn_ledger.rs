//! M5-3h churn-attribution ledger (3h exit row): five blind auditors
//! independently reimplemented the attribution spec (pairing, minimal
//! diff, unit segmentation, nth, owner, rewrite) and derived the
//! per-unit ledger for the 40 most recent qualifying commits at the
//! pinned tip. The frozen ledger is THEIRS (14 commits adjudicated to
//! the product's Myers under the equal-minimal-alignment conservation
//! certificate, both views preserved in the doc); the product must
//! match it row for row — conservation alone cannot catch systematic
//! misattribution (wrong nth order, outermost owner, nth-keyed
//! rewrite), which is exactly why this instrument exists.
//!
//! Gates: `ledger_structure_is_sound` runs everywhere (doc-internal
//! consistency); the replay and conservation gates need the real git
//! histories (self at the pinned tip; corpora under .ce-eval), so
//! they are ignored in CI and run locally:
//!   cargo test --test eval_churn_ledger -- --ignored --nocapture

mod common;
mod eval_support;

use codeeraser::churn;
use codeeraser::fourclass::session;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn doc() -> Value {
    eval_support::load(&eval_support::eval_doc("churn-ledger"))
}

/// (path, key, nth, appended, rewrote) from one frozen row array —
/// the row IS the tuple, so serde does the shape checking.
fn frozen_row(v: &Value) -> (String, String, i64, usize, usize) {
    serde_json::from_value(v.clone()).expect("row [path,key,nth,appended,rewrote]")
}

fn product_row(u: &churn::UnitRow) -> (String, String, i64, usize, usize) {
    (u.path.clone(), u.key.clone(), u.nth, u.appended, u.rewrote)
}

/// Validate one frozen commit (hex sha; rows sorted, identity-unique,
/// non-vacuous) and return (agent, rows, appended, rewrote).
fn checked_commit(c: &Value) -> (String, u64, u64, u64) {
    let sha = c["sha"].as_str().expect("sha");
    assert!(
        sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "full hex sha: {sha}"
    );
    let rows: Vec<_> = c["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(frozen_row)
        .collect();
    let (mut ap, mut rw) = (0u64, 0u64);
    let ids: Vec<_> = rows
        .iter()
        .map(|(p, k, n, a, r)| {
            assert!(a + r >= 1, "vacuous row in {sha}");
            ap += *a as u64;
            rw += *r as u64;
            (p.clone(), k.clone(), *n)
        })
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids, sorted, "{sha}: rows sorted and identity-unique");
    let agent = c["agent"].as_str().expect("agent").to_string();
    (agent, rows.len() as u64, ap, rw)
}

#[test]
fn ledger_structure_is_sound() {
    let d = doc();
    assert_eq!(d["schema"], "ce.churn-ledger/1");
    let commits = d["commits"].as_array().expect("commits");
    assert_eq!(commits.len(), 40, "the 40 qualifying commits");
    assert_eq!(
        commits[0]["sha"], d["tip"],
        "log order starts at the pinned tip"
    );
    let mut seats: BTreeMap<String, usize> = BTreeMap::new();
    let (mut rows_n, mut appended, mut rewrote) = (0u64, 0u64, 0u64);
    let mut shas: Vec<String> = Vec::new();
    for c in commits {
        shas.push(c["sha"].as_str().expect("sha").to_string());
        let (agent, n, ap, rw) = checked_commit(c);
        *seats.entry(agent).or_default() += 1;
        rows_n += n;
        appended += ap;
        rewrote += rw;
    }
    shas.sort();
    shas.dedup();
    assert_eq!(shas.len(), 40, "40 distinct commits");
    let want: BTreeMap<String, usize> = ["A", "B", "C", "D", "E"]
        .iter()
        .map(|a| (a.to_string(), 8))
        .collect();
    assert_eq!(seats, want, "five auditors, eight commits each");
    assert_eq!(d["totals"]["rows"].as_u64(), Some(rows_n));
    assert_eq!(d["totals"]["appended"].as_u64(), Some(appended));
    assert_eq!(d["totals"]["rewrote"].as_u64(), Some(rewrote));
    assert!(rows_n > 0, "an all-empty ledger audits nothing");
}

/// git stdout in `repo` through the common spawn-and-assert throat —
/// the gates must never turn a broken history into a silent pass.
fn git(repo: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&common::git_out(repo, args)).into_owned()
}

/// The frozen ledger's own language universe: the 3h blind audit's
/// transcription spec named FIVE tree-sitter languages, so the
/// artifact's rows can only ever cover those — replay compares on
/// the artifact's scope, not on whatever the product speaks today.
/// An ALLOWLIST of the freeze-time set (never a deny of the newest
/// language): Haskell rows (3k) and any later language fall outside
/// automatically, and the frozen half must still match ROW FOR ROW
/// (frozen-not-product stays the systematic-misattribution alarm).
fn in_frozen_scope(path: &str) -> bool {
    use codeeraser::scan::lang::Lang;
    matches!(
        Lang::from_path(Path::new(path)),
        Some(Lang::Python | Lang::TypeScript | Lang::Tsx | Lang::Rust | Lang::Go | Lang::Markdown)
    )
}

#[test]
#[ignore] // needs the self repo's git history containing the pinned tip
fn product_matches_the_audited_ledger_row_for_row() {
    let d = doc();
    let root = Path::new("..");
    let tip = d["tip"].as_str().expect("tip");
    git(root, &["merge-base", "--is-ancestor", tip, "HEAD"]);
    let mut mismatches = Vec::new();
    for c in d["commits"].as_array().expect("commits") {
        let sha = c["sha"].as_str().expect("sha");
        let got: Vec<_> = churn::commit_ledger(root, sha)
            .iter()
            .filter(|u| in_frozen_scope(&u.path))
            .map(product_row)
            .collect();
        let want: Vec<_> = c["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(frozen_row)
            .collect();
        if got != want {
            let extra: Vec<_> = got.iter().filter(|r| !want.contains(r)).collect();
            let missing: Vec<_> = want.iter().filter(|r| !got.contains(r)).collect();
            mismatches.push(format!(
                "{sha}: product-not-frozen {extra:?} | frozen-not-product {missing:?}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of 40 commits disagree:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// Added-line count of one commit WITHOUT unit attribution: the same
/// pair_texts + classify surfaces the ledger consumes, but the count
/// never touches owner/nth/ledger code — a ledger that drops or
/// double-books lines cannot balance against this.
fn independent_added(repo: &Path, sha: &str) -> usize {
    session::commit_pairs(repo, sha)
        .unwrap_or_default()
        .iter()
        .filter_map(|pair| churn::pair_texts(repo, sha, pair))
        .map(|(_, before, after, lang)| {
            codeeraser::fourclass::classify(&before, &after, lang)
                .changed
                .added
                .len()
        })
        .sum()
}

#[test]
#[ignore] // needs the corpora clones under .ce-eval plus self history
fn per_corpus_conservation_over_recent_commits() {
    let corpora: [Option<String>; 5] = [
        None,
        Some("cobra".into()),
        Some("requests".into()),
        Some("ripgrep".into()),
        Some("zod".into()),
    ];
    for name in corpora {
        let repo = match eval_support::universe::corpus_repo(&name) {
            Some(p) => std::path::PathBuf::from(p),
            None => std::path::PathBuf::from(".."),
        };
        let tip = match &name {
            Some(_) => eval_support::universe::frozen_tip("t3-universe", &name).1,
            None => doc()["tip"].as_str().expect("tip").to_string(),
        };
        let log = git(
            &repo,
            &[
                "log",
                &tip,
                "--first-parent",
                "--no-merges",
                "-20",
                "--format=%H",
            ],
        );
        let label = name.as_deref().unwrap_or("self");
        for sha in log.split_whitespace() {
            let rows = churn::commit_ledger(&repo, sha);
            let summed: usize = rows.iter().map(|r| r.appended + r.rewrote).sum();
            let counted = independent_added(&repo, sha);
            assert_eq!(summed, counted, "{label} {sha}: ledger sum vs added count");
        }
        println!("conservation {label}: 20 commits balanced");
    }
}
