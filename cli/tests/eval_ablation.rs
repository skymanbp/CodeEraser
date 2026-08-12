//! M5-1c-ii shadow ablation — the decision instrument behind the L2
//! upgrade (user decisions 2026-08-11, ccm #470): requests L2 broke
//! the invention gate (2 stations / 4 lines on a black/isort commit),
//! and hand measurement showed the menu variants failing to separate
//! there while a content-quality floor did. Before any core change,
//! every candidate is measured OFFLINE on both frozen corpora:
//!
//! - baseline — an exact Rust mirror of the core's judgment, proven
//!   per commit against the live ce-core delta (fidelity assert);
//! - quality / freq / chain / flow — site filters (formalizations in
//!   eval_ablation_parts::variants and EVAL-SET.md);
//! - phase3_edge — the F4 width probe (deletion-side attribution
//!   with an anchored-edge requirement).
//!
//! The self bars (recall 547/547, zero invention) are hard: a variant
//! row that breaks them is disqualified BY the matrix, and the
//! baseline row must keep them by construction. FPR replay is
//! untouched — this instrument never enters the production path.
//!
//! Run: CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!      cargo test --test eval_ablation -- --ignored --nocapture

mod eval_ablation_parts;
mod eval_commit_review;
mod eval_l2_parts;
mod eval_support;

use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::leftovers;
use eval_ablation_parts::variants::{self, Variant};
use eval_ablation_parts::{self as shadow, ledgers};
use eval_l2_parts as parts;
use eval_support::{by_sha, core_link, corpus_doc_pairs, eval_doc, load, u64s};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// Everything one commit contributes to the matrix.
struct Commit {
    sha: String,
    texts: parts::Texts,
    sent: Vec<(
        codeeraser::fourclass::batch::Side,
        codeeraser::fourclass::batch::Side,
    )>,
    blocks: Vec<shadow::ShadowBlock>,
    base: shadow::Moved,
    gt: parts::CrossGt,
    has_cross: bool,
    ctx: variants::Ctx,
}

#[derive(Default)]
struct Acc {
    sums: BTreeMap<&'static str, variants::Row>,
    rows: Vec<Value>,
    kills: Vec<Value>,
    width: Vec<Value>,
    gt_out: u64,
    gt_in: u64,
    equal: u64,
}

/// Run the live pipeline and the baseline shadow on one commit and
/// assert they agree — the per-commit fidelity proof.
fn commit_shadow(link: &mut Link, s: &Value, labels: &Value) -> Commit {
    let (sha, texts) = parts::commit_texts(s, labels);
    let (l1, l2) = parts::live_pair(&sha, &texts, link);
    let sent = leftovers(&parts::pair_inputs(&texts), &l1.pairs);
    let (blocks, capped) = shadow::sites(&sent);
    assert!(!capped, "{sha}: bucket cap tripped");
    let base = shadow::moved(&sent, &blocks, false);
    let live = parts::delta_lines(&texts, &l1, &l2);
    assert_eq!(shadow::delta(&texts, &base), live, "{sha}: shadow != live");
    let gt = parts::cross_gt(&sha);
    let has_cross = parts::has_cross(by_sha(labels).get(sha.as_str()));
    let ctx = variants::Ctx::build(&sha, &texts, &blocks);
    Commit {
        sha,
        texts,
        sent,
        blocks,
        base,
        gt,
        has_cross,
        ctx,
    }
}

/// Score every variant on one commit; all-zero commits leave no row.
fn commit_row(c: &Commit, acc: &mut Acc) {
    acc.equal += 1;
    for ((side, _), g) in &c.gt.counts {
        *(if side == "out" {
            &mut acc.gt_out
        } else {
            &mut acc.gt_in
        }) += g;
    }
    let mut vjson = serde_json::Map::new();
    let mut nonzero = false;
    for (name, v) in variants::ALL {
        let blocks = variants::filter(v, c.blocks.clone(), &c.ctx);
        let m = shadow::moved(&c.sent, &blocks, v == Variant::Phase3Edge);
        let row = variants::metrics(&shadow::delta(&c.texts, &m), &c.gt, c.has_cross);
        nonzero |= !row.is_zero();
        acc.sums.entry(name).or_default().add(&row);
        vjson.insert(name.into(), row.to_json());
        if v == Variant::Quality {
            acc.kills
                .extend(ledgers::kill_ledger(&c.sha, &c.texts, &c.blocks, &c.ctx));
        }
        if v == Variant::Phase3Edge {
            acc.width.extend(ledgers::width_ledger(
                &c.sha,
                &c.texts,
                &c.gt,
                &c.base.outs,
                &m.outs,
            ));
        }
    }
    if nonzero {
        acc.rows.push(json!({"sha": c.sha, "variants": vjson}));
    }
}

const METHOD: &str = "shadow ablation: a Rust mirror of the core's L2 judgment \
    (Anchor sites + Provenance phases 2/3) over the exact leftovers() run \
    structure; the BASELINE shadow must equal the live ce-core delta on every \
    commit (asserted at generation — the mirror is proven, not trusted). \
    Variants re-judge the same input: quality = a site needs one evidence \
    line with >= 20 alphanumeric chars; freq = a site needs one evidence \
    line whose trimmed content is unique in the base tree (slice langs, \
    memory/ excluded); chain = per pair-edge maximum-lines subset with \
    strictly increasing source and destination starts; flow = destination- \
    line exclusivity, greedy by size then position; phase3_edge = deletion- \
    side attribution requires an anchored edge (attack review F4 width \
    probe). Rows: per variant [cross_hits, cross_misses, identity_misses, \
    invention_lines, extras_files, extras_lines, pred_lines]; all-zero \
    commits omitted. Tail row: quality kill ledger + phase3 width ledger.";

#[test]
#[ignore] // needs the pinned window's git history + a built ce-core (CE_CORE_BIN)
fn generate_commit_ablation() {
    let labels = load(&eval_support::corpus().doc("labels"));
    let mut link = core_link();
    let mut acc = Acc::default();
    eval_support::generate_commit_doc(
        "ablation",
        "ce.eval-commit-ablation/1.0.0",
        METHOD,
        |slice| {
            let commits = slice["commits"].as_array().expect("commits");
            for s in commits {
                let c = commit_shadow(&mut link, s, &labels);
                commit_row(&c, &mut acc);
            }
            let summary = json!({
                "commits": commits.len() as u64,
                "equivalence_commits": acc.equal,
                "cross_gt_out": acc.gt_out,
                "cross_gt_in": acc.gt_in,
                "variants": acc.sums.iter()
                    .map(|(k, v)| (k.to_string(), v.to_json()))
                    .collect::<serde_json::Map<_, _>>(),
                "quality_kills": acc.kills.len() as u64,
                "phase3_width": ledgers::width_summary(&acc.width),
            });
            let mut rows = std::mem::take(&mut acc.rows);
            rows.push(json!({
                "kill_ledger": std::mem::take(&mut acc.kills),
                "width_ledger": std::mem::take(&mut acc.width),
            }));
            (summary, rows)
        },
    );
}

/// CI gate, no git needed, every corpus: the summary re-derives from
/// the rows and ledgers; cross GT anchors to the frozen labels; the
/// baseline column keeps the L2 bars (misses zero everywhere,
/// invention zero on the self corpus).
#[test]
fn commit_ablation_consistent() {
    let labels_docs: HashMap<_, _> = corpus_doc_pairs("labels").into_iter().collect();
    for (slice_path, doc_path) in corpus_doc_pairs("ablation") {
        check_corpus(&slice_path, &labels_docs[&slice_path], &doc_path);
    }
}

fn check_corpus(slice_path: &str, labels_path: &str, doc_path: &str) {
    let (doc, slice, labels) = (load(doc_path), load(slice_path), load(labels_path));
    let all = doc["commits"].as_array().expect("commits");
    let (tail, rows) = all.split_last().expect("rows");
    let s = &doc["summary"];
    let n = slice["commits"].as_array().expect("slice").len() as u64;
    assert_eq!(s["commits"].as_u64(), Some(n), "commit coverage");
    assert_eq!(
        s["equivalence_commits"], s["commits"],
        "fidelity must cover every commit"
    );
    let ml = &labels["summary"]["moved_lines"];
    assert_eq!(s["cross_gt_out"], ml["cross_out"], "cross GT anchor");
    assert_eq!(s["cross_gt_in"], ml["cross_in"], "cross GT anchor");
    check_sums(s, rows);
    check_ledgers(s, tail);
    let base = u64s(&s["variants"]["baseline"]);
    assert_eq!(base[1], 0, "baseline cross misses");
    assert_eq!(base[2], 0, "baseline identity misses");
    if doc_path == eval_doc("commit-ablation") {
        assert_eq!(base[3], 0, "self corpus: baseline invention");
    }
}

fn check_sums(s: &Value, rows: &[Value]) {
    for (name, _) in variants::ALL {
        let mut sum = vec![0u64; 7];
        for r in rows {
            for (i, v) in u64s(&r["variants"][name]).iter().enumerate() {
                sum[i] += v;
            }
        }
        assert_eq!(sum, u64s(&s["variants"][name]), "{name}: summary drifted");
    }
}

fn check_ledgers(s: &Value, tail: &Value) {
    let kills = tail["kill_ledger"].as_array().expect("kill ledger");
    assert_eq!(
        s["quality_kills"].as_u64(),
        Some(kills.len() as u64),
        "kill count"
    );
    let width = tail["width_ledger"].as_array().expect("width ledger");
    assert_eq!(
        s["phase3_width"],
        ledgers::width_summary(width),
        "width summary"
    );
}
