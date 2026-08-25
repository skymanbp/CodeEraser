//! `ce erase` end-to-end against the contract's own acceptance list
//! (docs/reference/erase.md §acceptance): the three classes plan and
//! apply on a fixture tree with a converging re-run, a live T2 twin
//! is never planned, refusals (dirty worktree / drifted hash /
//! non-repo) fire BY NAME without touching anything, and two plans
//! over one tree are byte-identical. The self-repo zero-row gate is
//! CI's `ce erase .. --check` leg, not this file.

mod common;

use codeeraser::erase;
use common::gitio::git;
use common::{commit_all, core_bin, tmp};
use std::path::Path;

/// 50+ admitted words, byte-identical in both hosts — the verbatim
/// full-segment class.
const PARA: &str = "alpha beta gamma delta epsilon zeta eta theta iota kappa \
lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega one two \
three four five six seven eight nine ten eleven twelve thirteen fourteen \
fifteen sixteen seventeen eighteen nineteen twenty apple pear plum grape \
melon berry stone river cloud";

/// ~58 normalized tokens, one whole unit — the T1 side.
const FN_T1: &str = "def compute_total(values):
    values = list(values)
    count = len(values) + 3
    total = 0
    for item in values:
        if item > 3:
            total = total + item * 2
        else:
            total = total - 1
    result = total * 7 + count
    print(result)
    return result
";

/// The same shape with renamed identifiers and shifted literals —
/// T2 under normalization, byte-different: the advisory class.
const FN_T2: &str = "def sum_amounts(entries):
    entries = list(entries)
    width = len(entries) + 4
    acc = 0
    for row in entries:
        if row > 5:
            acc = acc + row * 6
        else:
            acc = acc - 2
    outcome = acc * 8 + width
    print(outcome)
    return outcome
";

fn seed(dir: &Path) {
    let w = |name: &str, text: String| std::fs::write(dir.join(name), text).expect(name);
    w(
        "README.md",
        format!("# fixture\n\n[doc2](doc2.md)\n\n{PARA}\n"),
    );
    w("doc2.md", format!("# second\n\n{PARA}\n"));
    w("orphan.md", "an unlinked note nobody references\n".into());
    w("__main__.py", format!("import t2\n\n{FN_T1}"));
    w("copy.py", FN_T1.into());
    w("t2.py", FN_T2.into());
    git(dir, &["init", "-q"]);
    commit_all(dir, "seed");
}

fn row<'a>(p: &'a erase::Plan, class: &str, path: &str) -> &'a erase::Row {
    p.rows
        .iter()
        .find(|r| r.class == class && r.path == path)
        .unwrap_or_else(|| panic!("no {class} row for {path}"))
}

/// One named refusal: apply errs with `needle` and touches nothing
/// (the orphan is every fixture's canary file).
fn refuses(dir: &Path, core: &str, plan: &erase::Plan, needle: &str) {
    let err = erase::apply_plan(dir, None, core, plan).expect_err(needle);
    assert!(err.to_string().contains(needle), "{err:#}");
    assert!(dir.join("orphan.md").exists(), "refusal touched nothing");
}

#[test]
fn plan_names_all_classes_and_is_deterministic() {
    let dir = tmp("erase-plan");
    seed(&dir);
    let core = core_bin();
    let p = erase::plan(&dir, None, &core).expect("plan");
    assert_eq!(p.counts.eraseable, 2, "orphan + doc2 span");
    assert!(row(&p, "dead_file", "orphan.md").eraseable);
    // copy.py declares `def compute_total(...)` — an export surface, so
    // its dead verdict is unref_public and RG10 forbids acting on it
    // (6.1.0). Before the firewall reached this face the plan proposed
    // deleting a file whose public function nothing in the tree calls,
    // which is the ordinary shape of a library. orphan.md declares
    // nothing and stays eraseable, so the guard is a firewall and not
    // a mute.
    let copy = row(&p, "dead_file", "copy.py");
    assert!(!copy.eraseable);
    assert_eq!(copy.reason, "public_surface");
    let doc = row(&p, "verbatim_doc", "doc2.md");
    assert!(
        doc.eraseable && doc.span.is_some(),
        "span row, not the file"
    );
    // the live T2 twin is named, never planned (acceptance row 3)
    let twin = row(&p, "t1_twin", "t2.py");
    assert!(!twin.eraseable);
    assert_eq!(twin.reason, "bytes_differ");
    // acceptance row 4: two plans over one tree are byte-identical
    let again = erase::plan(&dir, None, &core).expect("plan again");
    assert_eq!(
        erase::render::report_json(&p).to_string(),
        erase::render::report_json(&again).to_string(),
        "determinism pinned"
    );
    // the diff face renders every eraseable row and carries provenance
    let diff = erase::render::diff(&dir, &p).expect("diff");
    for needle in [
        "+++ /dev/null",
        "## dead_file",
        "## verbatim_doc",
        "fnv1a64:",
    ] {
        assert!(diff.contains(needle), "diff face missing {needle}");
    }
}

#[test]
fn apply_refuses_by_name_then_converges() {
    let dir = tmp("erase-apply");
    seed(&dir);
    let core = core_bin();
    let plan = erase::plan(&dir, None, &core).expect("plan");
    // dirty worktree refuses, nothing touched
    std::fs::write(dir.join("junk.txt"), "wip").expect("junk");
    refuses(&dir, &core, &plan, "worktree not clean");
    std::fs::remove_file(dir.join("junk.txt")).expect("rm junk");
    // a drifted target refuses by name (clean tree, changed bytes)
    let mut doc2 = std::fs::read_to_string(dir.join("doc2.md")).expect("doc2");
    doc2.push_str("\ntrailing edit\n");
    std::fs::write(dir.join("doc2.md"), doc2).expect("drift");
    commit_all(&dir, "drift");
    refuses(&dir, &core, &plan, "content changed since planning");
    // a fresh plan applies, converges, and leaves the audit trail
    let plan = erase::plan(&dir, None, &core).expect("re-plan");
    let n = erase::apply_plan(&dir, None, &core, &plan).expect("apply");
    assert_eq!(n, 2);
    assert!(!dir.join("orphan.md").exists());
    // the export surface survives the apply, not just the plan (6.1.0)
    assert!(dir.join("copy.py").exists(), "RG10 holds through apply");
    let doc2 = std::fs::read_to_string(dir.join("doc2.md")).expect("doc2");
    assert!(!doc2.contains("alpha beta gamma"), "span spliced out");
    assert!(doc2.contains("# second"), "the rest of the file survives");
    let log = std::fs::read_to_string(dir.join(".ce/erase-log.ndjson")).expect("log");
    assert_eq!(log.lines().count(), 2, "one record per applied row");
    assert!(log.lines().all(|l| l.contains("ce.erase-log/0.1.0")));
    let after = erase::plan(&dir, None, &core).expect("post-plan");
    assert_eq!(after.counts.eraseable, 0, "converged");
}

#[test]
fn apply_outside_a_repo_refuses() {
    let dir = tmp("erase-norepo"); // tmp()'s .git is an empty anchor dir, not a repository
    std::fs::write(
        dir.join("orphan.md"),
        "an unlinked note nobody references\n",
    )
    .expect("md");
    let core = core_bin();
    let plan = erase::plan(&dir, None, &core).expect("plan");
    assert!(plan.counts.eraseable >= 1, "the orphan plans");
    refuses(&dir, &core, &plan, "not a git repository");
}
