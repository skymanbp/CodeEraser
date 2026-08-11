//! M4-7 L2 evaluation — THE BAR. Runs the real pipeline
//! (fourclass::batch over a live ce-core link) on every commit of the
//! frozen slice and freezes the gates in commit-l2-v1.json:
//!
//! 1. cross recall — per-file hits cover the reviewed cross GT
//!    (366 out / 181 in), misses = 0;
//! 2. coincidence gate — on each reviewed correction file the
//!    predicted cross count EQUALS the GT count: with per-file misses
//!    at zero, an exact count leaves no room for the excluded line;
//! 3. invention gate — zero cross predictions on commits whose GT has
//!    no cross moves;
//! 4. monotonicity + L1 identity — L2 >= L1 on moved with conserved
//!    sums per pair (asserted at generation), and single-pair batches
//!    reproduce l1-v1.json on all 200 samples (separate ignored test);
//! 5. extras ledger — every predicted-above-GT file itemized with
//!    line content (GT's blocks-mode floor under-marks sub-block
//!    moves; an allowance in bulk would be a hole, a ledger is not);
//! 6. determinism — reversed pair order yields identical per-file
//!    deltas (asserted during generation);
//! 7. cost-model sensitivity — pinned Haskell-side (Spec.hs
//!    costModel: the floor tracks the site cost).
//!
//! Run: CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!      cargo test --test eval_l2 -- --ignored --nocapture

mod eval_commit_review;
mod eval_l2_parts;
mod eval_l2_register;
mod eval_support;

use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::{PairInput, classify_batch};
use eval_l2_parts as parts;
use eval_support::{by_sha, each_sample, gt_pairs, load, u64s};
use serde_json::{Value, json};

/// Both class splits must sum to the same numstat on each side.
fn assert_conserved(sha: &str, a: &[u64], b: &[u64]) {
    assert_eq!(a[0] + a[1], b[0] + b[1], "{sha}: added conserved");
    assert_eq!(a[2] + a[3], b[2] + b[3], "{sha}: removed conserved");
}

fn core_link() -> Link {
    let bin = std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    );
    Link::open(&bin).expect("ce-core link").0
}

fn commit_row(link: &mut Link, s: &Value, labels: &Value) -> (Value, Vec<Value>) {
    let sha = s["sha"].as_str().expect("sha");
    let by = by_sha(labels);
    let gt_rows = gt_pairs(s, &by);
    let texts = parts::pair_texts(sha, gt_rows);
    let l1 = parts::run_batch(&texts, None);
    let l2 = parts::run_batch(&texts, Some(link));
    assert!(l2.degraded.is_none(), "{sha}: degraded {:?}", l2.degraded);
    let (c1, c2) = (parts::counts_of(&l1), parts::counts_of(&l2));
    for (a, b) in c1.iter().zip(&c2) {
        assert!(b[1] >= a[1] && b[3] >= a[3], "{sha}: monotonicity");
        assert_conserved(sha, a, b);
    }
    let delta = parts::delta_lines(&texts, &l1, &l2);
    let gt = parts::cross_gt(sha);
    let rows = parts::cross_rows(sha, &texts, &gt, &delta);
    let ledger = parts::extras_ledger(sha, &texts, &gt, &delta);
    let pairs: Vec<Value> = gt_rows
        .iter()
        .zip(c1.iter().zip(&c2))
        .map(|(gp, (a, b))| {
            json!({"before": gp["before"], "after": gp["after"],
                   "gt": eval_support::gt_counts(gp),
                   "l1": a.to_vec(), "l2": b.to_vec()})
        })
        .collect();
    let relocations = eval_l2_register::relocations_json(&l2, gt_rows);
    let moved_units = eval_l2_register::moved_units(&l1, &l2);
    (
        json!({"sha": sha, "pairs": pairs, "cross": rows, "relocations": relocations,
               "moved_units": moved_units}),
        ledger,
    )
}

/// Determinism: reversed pair order must yield the same per-file
/// delta on the largest relocation commit.
fn assert_deterministic(link: &mut Link, sha: &str, labels: &Value) {
    let by = by_sha(labels);
    let slice = load("../contracts/eval/commit-slice-v1.json");
    let s = by_sha(&slice)[sha];
    let texts = parts::pair_texts(sha, gt_pairs(s, &by));
    let forward = parts::delta_lines(
        &texts,
        &parts::run_batch(&texts, None),
        &parts::run_batch(&texts, Some(link)),
    );
    let mut reversed = texts.clone();
    reversed.reverse();
    let backward = parts::delta_lines(
        &reversed,
        &parts::run_batch(&reversed, None),
        &parts::run_batch(&reversed, Some(link)),
    );
    assert_eq!(forward, backward, "{sha}: order-dependent delta");
}

#[test]
#[ignore] // needs full git history + a built ce-core (CE_CORE_BIN)
fn generate_commit_l2() {
    let labels = load("../contracts/eval/commit-labels-v1.json");
    let mut link = core_link();
    let mut ledger: Vec<Value> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    eval_support::generate_commit_doc(
        "../contracts/eval/commit-l2-v1.json",
        "ce.eval-commit-l2/1.0.0",
        "real pipeline: fourclass::batch::classify_batch over a live \
         ce-core link, per slice commit; gt = commit-labels-v1; cross \
         GT per file re-derived by the labels machinery (partition + \
         reviewed corrections). Gates: cross misses = 0; coincidence \
         files exact; zero invention on non-cross commits; L2>=L1 \
         monotone with conserved sums (asserted at generation); \
         extras itemized with content (GT blocks-mode floor \
         under-marks sub-block moves); reversed-order determinism \
         (asserted at generation); cost-model sensitivity pinned in \
         core/test/Spec.hs.",
        |slice| {
            for s in slice["commits"].as_array().expect("commits") {
                let (row, mut extra) = commit_row(&mut link, s, &labels);
                rows.push(row);
                ledger.append(&mut extra);
            }
            assert_deterministic(
                &mut link,
                "4822d04ff0a944eba62fed71cb6ccd226a647e77",
                &labels,
            );
            let mut doc_rows = std::mem::take(&mut rows);
            let misses = eval_l2_register::register_misses(&doc_rows);
            assert!(misses.is_empty(), "relocation register misses: {misses:?}");
            let summary = parts::summarize(&doc_rows, &ledger);
            doc_rows.push(json!({"extras_ledger": std::mem::take(&mut ledger)}));
            (summary, doc_rows)
        },
    );
}

/// L2 must be bitwise L1 on single-pair batches — the structural
/// claim behind "no regression on the saturated 200-sample set",
/// verified against the committed l1-v1.json rather than trusted.
#[test]
#[ignore] // needs the local .ce-eval payloads (eval_extract output)
fn l1_reproduced_on_200_samples() {
    let l1 = load("../contracts/eval/l1-v1.json");
    let rows = l1["rows"].as_array().expect("rows");
    each_sample(rows, |id, sample| {
        let inputs = [PairInput {
            before: sample["before"].as_str().expect("before"),
            after: sample["after"].as_str().expect("after"),
            lang: eval_support::lang_of(sample["lang"].as_str().expect("lang")),
        }];
        let b = classify_batch(&inputs, None);
        let got = parts::counts_of(&b)[0];
        let row = rows.iter().find(|r| r["id"] == id).expect("row");
        assert_eq!(got.to_vec(), u64s(&row["pred"]), "{id}: drifted from L1");
    });
    println!("200/200 single-pair batches reproduce l1-v1.json");
}

/// CI gate, no git needed: summary re-derives; cross GT anchors to
/// the frozen labels totals; misses zero; coincidence files exact;
/// invention zero; per-pair L2 conserves the GT numstat.
#[test]
fn commit_l2_consistent() {
    let doc = load("../contracts/eval/commit-l2-v1.json");
    let labels = load("../contracts/eval/commit-labels-v1.json");
    let all = doc["commits"].as_array().expect("commits");
    let (tail, rows) = all.split_last().expect("rows");
    let ledger = tail["extras_ledger"].as_array().expect("ledger");
    let s = &doc["summary"];
    assert_eq!(*s, parts::summarize(rows, ledger), "summary drifted");
    let ml = &labels["summary"]["moved_lines"];
    assert_eq!(s["cross_gt_out"], ml["cross_out"], "cross GT anchor");
    assert_eq!(s["cross_gt_in"], ml["cross_in"], "cross GT anchor");
    assert_eq!(s["cross_misses"], 0, "cross recall gate");
    let misses = eval_l2_register::register_misses(rows);
    assert!(misses.is_empty(), "relocation register misses: {misses:?}");
    check_rows(rows, &labels);
}

fn check_rows(rows: &[Value], labels: &Value) {
    let by = by_sha(labels);
    let coincidence = [
        ("da8eb210b", "cli/tests/guard_hook.rs", "out"),
        ("da8eb210b", "cli/tests/mcp_precommit.rs", "in"),
        ("8d6b237a5", "cli/src/dedup/mod.rs", "out"),
        ("8d6b237a5", "cli/tests/dedup_check.rs", "in"),
        ("4822d04ff", "cli/tests/daemon_e2e.rs", "in"),
    ];
    for r in rows {
        let sha = r["sha"].as_str().expect("sha");
        let has_cross = by
            .get(sha)
            .map(|l| {
                l["cross_file"]["out"].as_u64().unwrap() + l["cross_file"]["in"].as_u64().unwrap()
                    > 0
            })
            .unwrap_or(false);
        for c in r["cross"].as_array().expect("cross") {
            let (g, p) = parts::gt_pred(c);
            if !has_cross {
                assert_eq!(p, 0, "{sha}: invention on a non-cross commit");
            }
            let hit = coincidence
                .iter()
                .any(|(s9, f, sd)| sha.starts_with(s9) && c["file"] == *f && c["side"] == *sd);
            if hit {
                assert_eq!(p, g, "{sha}: coincidence file must be exact: {c}");
            }
        }
        for p in r["pairs"].as_array().expect("pairs") {
            assert_conserved(sha, &u64s(&p["gt"]), &u64s(&p["l2"]));
        }
    }
}
