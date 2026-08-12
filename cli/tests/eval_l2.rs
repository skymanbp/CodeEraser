//! M4-7 L2 evaluation — THE BAR. Runs the real pipeline
//! (fourclass::batch over a live ce-core link) on every commit of the
//! frozen slice and freezes the gates in commit-l2-v1.json:
//!
//! 1. cross recall — per-file hits cover the reviewed cross GT
//!    (366 out / 181 in), misses = 0 beyond the reviewed below-floor
//!    register (M5-1d: true relocated lines with no >=2-distinct
//!    contiguous companion, itemized per line — the miss-side mirror
//!    of the extras ledger; empty on self and requests);
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
//! M5-1c: corpus-aware throughout — CE_SLICE_* retargets generation,
//! the gate covers every FROZEN corpus (requests pending: EVAL-SET),
//! the coincidence table derives from each labels doc's corrections
//! (verified set-equal to the old hardcoded self table).
//!
//! Run: CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!      cargo test --test eval_l2 -- --ignored --nocapture

mod eval_commit_review;
mod eval_l2_parts;
mod eval_l2_register;
mod eval_support;

use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::classify_batch;
use eval_l2_parts as parts;
use eval_support::{
    by_sha, core_link, corpus, corpus_doc_pairs, each_sample, load, sample_pair, u64s,
};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Both class splits must sum to the same numstat on each side.
fn assert_conserved(sha: &str, a: &[u64], b: &[u64]) {
    assert_eq!(a[0] + a[1], b[0] + b[1], "{sha}: added conserved");
    assert_eq!(a[2] + a[3], b[2] + b[3], "{sha}: removed conserved");
}

fn commit_row(link: &mut Link, s: &Value, labels: &Value) -> (Value, Vec<Value>) {
    let (sha, texts) = parts::commit_texts(s, labels);
    let sha = sha.as_str();
    let (l1, l2) = parts::live_pair(sha, &texts, link);
    let (c1, c2) = (parts::counts_of(&l1), parts::counts_of(&l2));
    for (a, b) in c1.iter().zip(&c2) {
        assert!(b[1] >= a[1] && b[3] >= a[3], "{sha}: monotonicity");
        assert_conserved(sha, a, b);
    }
    let delta = parts::delta_lines(&texts, &l1, &l2);
    let gt = parts::cross_gt(sha);
    let rows = parts::cross_rows(sha, &texts, &gt, &delta);
    let ledger = parts::extras_ledger(sha, &texts, &gt, &delta);
    // The GT pair rows travel inside texts (pair_texts clones them).
    let gt_rows: Vec<Value> = texts.iter().map(|t| t.2.clone()).collect();
    let pairs: Vec<Value> = gt_rows
        .iter()
        .zip(c1.iter().zip(&c2))
        .map(|(gp, (a, b))| {
            json!({"before": gp["before"], "after": gp["after"],
                   "gt": eval_support::gt_counts(gp),
                   "l1": a.to_vec(), "l2": b.to_vec()})
        })
        .collect();
    let relocations = eval_l2_register::relocations_json(&l2, &gt_rows);
    let moved_units = eval_l2_register::moved_units(&l1, &l2);
    (
        json!({"sha": sha, "pairs": pairs, "cross": rows, "relocations": relocations,
               "moved_units": moved_units}),
        ledger,
    )
}

/// Determinism: reversed pair order must yield the same per-file
/// delta on the corpus's largest cross commit — derived from the
/// labels totals, not pinned (self elects 4822d04ff, 262 lines).
fn assert_deterministic(link: &mut Link, labels: &Value) {
    let sha = labels["commits"]
        .as_array()
        .expect("commits")
        .iter()
        .max_by_key(|r| {
            r["cross_file"]["out"].as_u64().unwrap() + r["cross_file"]["in"].as_u64().unwrap()
        })
        .expect("labels rows")["sha"]
        .as_str()
        .expect("sha");
    let slice = load(&corpus().doc("slice"));
    let (_, texts) = parts::commit_texts(by_sha(&slice)[sha], labels);
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

/// Attack review F2: the extras ledger is FROZEN — a regeneration may
/// not introduce rows absent from the committed doc. New extras mean
/// new above-GT predictions; they must be REVIEWED, then blessed
/// explicitly with CE_ACCEPT_EXTRAS=1, never absorbed silently.
fn assert_extras_frozen(ledger: &[Value]) {
    if std::env::var("CE_ACCEPT_EXTRAS").as_deref() == Ok("1") {
        return;
    }
    // A corpus's first freeze has no committed doc: everything is new
    // and must go through the same review-then-bless flow.
    let path = corpus().doc("l2");
    let old = match std::fs::exists(&path).expect("probe") {
        false => Vec::new(),
        true => {
            let committed = load(&path);
            let rows = committed["commits"].as_array().expect("commits");
            rows.last().expect("ledger row")["extras_ledger"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        }
    };
    let fresh: Vec<&Value> = ledger.iter().filter(|e| !old.contains(e)).collect();
    assert!(
        fresh.is_empty(),
        "unreviewed NEW extras (review, then CE_ACCEPT_EXTRAS=1): {fresh:#?}"
    );
}

/// The corpus's labels-doc stem for the method text.
fn labels_stem() -> String {
    let path = corpus().doc("labels");
    let name = path.rsplit('/').next().expect("file name");
    name.trim_end_matches(".json").to_string()
}

#[test]
#[ignore] // needs the pinned window's git history + a built ce-core (CE_CORE_BIN)
fn generate_commit_l2() {
    let labels = load(&corpus().doc("labels"));
    let mut link = core_link();
    let mut ledger: Vec<Value> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    eval_support::generate_commit_doc(
        "l2",
        "ce.eval-commit-l2/1.0.0",
        &format!(
            "real pipeline: fourclass::batch::classify_batch over a live \
             ce-core link, per slice commit; gt = {}; cross \
             GT per file re-derived by the labels machinery (partition + \
             reviewed corrections). Gates: cross misses = 0 at LINE \
             IDENTITY for files without a reviewed correction, count \
             level on the corrected files, zero beyond the reviewed \
             below-floor register (true relocated lines with no \
             >=2-distinct contiguous companion — structurally below \
             destFloor, itemized per line); coincidence files exact; zero \
             invention on non-cross commits; L2>=L1 monotone with \
             conserved sums (asserted at generation); extras itemized \
             with content AND frozen (a new above-GT row fails the \
             generator until reviewed and blessed); reversed-order \
             determinism (asserted at generation); cost-model \
             sensitivity pinned in core/test/Spec.hs.",
            labels_stem()
        ),
        |slice| {
            for s in slice["commits"].as_array().expect("commits") {
                let (row, mut extra) = commit_row(&mut link, s, &labels);
                rows.push(row);
                ledger.append(&mut extra);
            }
            assert_extras_frozen(&ledger);
            assert_deterministic(&mut link, &labels);
            let mut doc_rows = std::mem::take(&mut rows);
            let misses = eval_l2_register::register_misses(&doc_rows);
            assert!(misses.is_empty(), "relocation register misses: {misses:?}");
            let bad = eval_l2_register::edge_violations(&doc_rows);
            assert!(bad.is_empty(), "invented relocation edges: {bad:?}");
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
        let inputs = sample_pair(&sample);
        let b = classify_batch(&inputs, None);
        let got = parts::counts_of(&b)[0];
        let row = rows.iter().find(|r| r["id"] == id).expect("row");
        assert_eq!(got.to_vec(), u64s(&row["pred"]), "{id}: drifted from L1");
    });
    println!("200/200 single-pair batches reproduce l1-v1.json");
}

/// CI gate, no git needed, every corpus: summary re-derives; cross
/// GT anchors to the frozen labels totals; misses zero; coincidence
/// files exact; invention zero; per-pair L2 conserves the L1 totals
/// (the judgment invariant — monotone reclassification moves lines
/// between classes, never creates or drops them).
#[test]
fn commit_l2_consistent() {
    let labels_docs: HashMap<_, _> = corpus_doc_pairs("labels").into_iter().collect();
    for (slice_path, doc_path) in eval_support::corpus_doc_pairs_frozen("l2") {
        check_corpus(&labels_docs[&slice_path], &doc_path);
    }
}

fn check_corpus(labels_path: &str, doc_path: &str) {
    let doc = load(doc_path);
    let labels = load(labels_path);
    let all = doc["commits"].as_array().expect("commits");
    let (tail, rows) = all.split_last().expect("rows");
    let ledger = tail["extras_ledger"].as_array().expect("ledger");
    let s = &doc["summary"];
    assert_eq!(*s, parts::summarize(rows, ledger), "summary drifted");
    assert_eq!(s["cross_misses"], 0, "cross recall gate");
    // Anchors + the below-floor ledger: hits + waived == cross GT.
    let (gt, bf) = eval_support::anchor_to_labels(s, &labels, "below_floor_lines");
    assert_eq!(
        s["cross_hits"].as_u64().unwrap() + bf,
        gt,
        "cross ledger conservation"
    );
    let misses = eval_l2_register::register_misses(rows);
    assert!(misses.is_empty(), "relocation register misses: {misses:?}");
    let bad = eval_l2_register::edge_violations(rows);
    assert!(bad.is_empty(), "invented relocation edges: {bad:?}");
    check_rows(rows, &labels);
}

/// The labels row's reviewed-correction (file, cross-side) set —
/// corrections record "added"/"removed", cross rows say "in"/"out".
/// Replaces the old hardcoded self-corpus table: same data, one
/// source, every corpus.
fn correction_files(labels_row: &Value) -> Vec<(String, &'static str)> {
    labels_row["corrections"]
        .as_array()
        .expect("corrections")
        .iter()
        .map(|c| {
            let file = c["file"].as_str().expect("file").to_string();
            let side = match c["side"].as_str().expect("side") {
                "added" => "in",
                _ => "out",
            };
            (file, side)
        })
        .collect()
}

fn check_rows(rows: &[Value], labels: &Value) {
    let by = by_sha(labels);
    for r in rows {
        let sha = r["sha"].as_str().expect("sha");
        let lrow = by.get(sha);
        let has_cross = parts::has_cross(lrow);
        let coincidence = lrow.map(|l| correction_files(l)).unwrap_or_default();
        for c in r["cross"].as_array().expect("cross") {
            let (g, p) = parts::gt_pred(c);
            if !has_cross {
                assert_eq!(p, 0, "{sha}: invention on a non-cross commit");
            }
            let hit = coincidence
                .iter()
                .any(|(f, sd)| c["file"] == *f && c["side"] == *sd);
            if hit {
                assert_eq!(p, g, "{sha}: coincidence file must be exact: {c}");
            }
        }
        // Conservation holds between L1 and L2 — the judgment's own
        // invariant. GT-vs-pipeline totals may legally diverge: the
        // GT carrier is git's hunk arithmetic, the pipeline is our
        // minimal Myers, and the two edit scripts differ on prose
        // rewrites (requests 26466b54 README.md: 46/56 vs 41/51,
        // both sides off by the same 5 — the 28d537dd class). The
        // gt column stays as data; pairs_exact tracks its alignment.
        for p in r["pairs"].as_array().expect("pairs") {
            assert_conserved(sha, &u64s(&p["l1"]), &u64s(&p["l2"]));
        }
    }
}
