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
use codeeraser::fourclass::batch::{leftovers, request_body};
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
    below_floor: u64,
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
    // Codex review F1: the moved-map assert cannot pin site
    // DECOMPOSITION (one union, many block sets) and the variants
    // filter BLOCKS — replay the exact wire request and require the
    // shadow's sites to equal the core's reply blocks verbatim.
    if sent.iter().any(|(r, a)| !r.is_empty() || !a.is_empty()) {
        let body = request_body(&parts::pair_inputs(&texts), &sent);
        let reply = link.request("fourclass", body).expect("fourclass");
        let mine: Vec<Value> = blocks
            .iter()
            .map(|b| json!([b.from_pair, b.from_lines, b.to_pair, b.to_lines]))
            .collect();
        assert_eq!(
            json!(mine),
            reply["blocks"],
            "{sha}: shadow sites != core blocks"
        );
    }
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
    acc.below_floor += c.gt.waived.values().map(|v| v.len() as u64).sum::<u64>();
    let mut vjson = serde_json::Map::new();
    let mut nonzero = false;
    for (name, v) in variants::ALL {
        let blocks = variants::filter(v, c.blocks.clone(), &c.ctx);
        let m = shadow::moved(&c.sent, &blocks, v == Variant::Phase3Edge);
        let mut row = variants::metrics(&shadow::delta(&c.texts, &m), &c.gt, c.has_cross);
        row.0[7] = (c.blocks.len() - blocks.len()) as u64;
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
    structure; on every commit the BASELINE shadow must equal the live \
    ce-core delta AND the shadow's site set must equal the core's reply \
    blocks verbatim (both asserted at generation — the mirror and its \
    DECOMPOSITION are proven, not trusted). Variants re-judge the same \
    input, each an exact predicate, not its menu family: quality = a site \
    needs ONE evidence line with >= 20 alphanumeric chars (single-anchor \
    rule; an aggregate-20 floor would NOT separate the requests invention, \
    7+16=23); freq = hard base-tree uniqueness gate (one evidence line \
    with tree frequency 1; slice langs, memory/ excluded); chain = per \
    pair-edge max-lines subset with strictly increasing source and \
    destination STARTS (the most lenient non-crossing form); flow = greedy \
    whole-block destination-line exclusivity (not a global matching); \
    phase3_edge = deletion-side attribution requires an anchored edge \
    (attack review F4 width probe). Rows: per variant [count_hits, \
    count_misses, identity_misses, invention_lines, extras_files, \
    extras_lines, pred_lines, blocks_dropped]; all-zero commits omitted \
    (the gate conserves the GT total against the labels anchor, so a \
    dropped nonzero row cannot hide). Reviewed below-floor lines (the \
    corpus's register of true relocations with no >=2-distinct \
    contiguous companion) leave every variant's hit/miss ledger alike: \
    hits + misses == cross GT - register. Tail row: quality kill \
    ledger + phase3 width ledger.";

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
            let mut summary = json!({
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
            // absent when the corpus's below-floor register is empty
            if acc.below_floor > 0 {
                summary["below_floor"] = json!(acc.below_floor);
            }
            let mut rows = std::mem::take(&mut acc.rows);
            rows.push(json!({
                "kill_ledger": std::mem::take(&mut acc.kills),
                "width_ledger": std::mem::take(&mut acc.width),
            }));
            (summary, rows)
        },
    );
}

/// Diagnostic itemization, NOT a gate: per (side, file) divergence
/// between the baseline shadow's delta and the reviewed cross GT,
/// with line content — the corpus-freeze review view (misses,
/// identity misses and extras are only counts in the matrix).
#[test]
#[ignore] // needs the pinned window's git history + a built ce-core (CE_CORE_BIN)
fn dump_cross_divergence() {
    let labels = load(&eval_support::corpus().doc("labels"));
    let slice = load(&eval_support::corpus().doc("slice"));
    let mut link = core_link();
    for s in slice["commits"].as_array().expect("commits") {
        let c = commit_shadow(&mut link, s, &labels);
        let delta = shadow::delta(&c.texts, &c.base);
        let mut keys: Vec<_> = c.gt.counts.keys().chain(delta.keys()).collect();
        keys.sort();
        keys.dedup();
        for key in keys {
            let g = c.gt.counts.get(key).copied().unwrap_or(0);
            let pred = delta.get(key).cloned().unwrap_or_default();
            let want = c.gt.lines.get(key);
            let missing: Vec<usize> = want
                .map(|w| w.iter().copied().filter(|l| !pred.contains(l)).collect())
                .unwrap_or_default();
            if pred.len() as u64 == g && missing.is_empty() {
                continue;
            }
            println!(
                "== {} {key:?} gt {g} pred {} missing {missing:?}",
                &c.sha[..9],
                pred.len()
            );
            let text = parts::side_text(&c.texts, &key.0, &key.1);
            let lines: Vec<&str> = text.lines().collect();
            let shown = pred
                .iter()
                .map(|l| (' ', *l))
                .chain(missing.iter().map(|l| ('-', *l)));
            for (mark, l) in shown {
                let content = lines.get(l - 1).unwrap_or(&"?").trim();
                println!("  {mark}{l}: {content}");
            }
        }
    }
}

/// CI gate, no git needed, every corpus (hardened per Codex review
/// F2 — a resummed doc is not integrity): row shas must be unique
/// members of the frozen slice; every variant must CONSERVE the
/// labels-anchored GT total (hits + misses == cross GT, so a zeroed
/// or deleted row breaks the sum against an EXTERNAL anchor); the
/// baseline column keeps the L2 bars and the quality column keeps
/// the frozen verdict (zero misses, zero invention, every corpus).
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
    let (_, bf) = eval_support::anchor_to_labels(s, &labels, "below_floor");
    check_rows_membership(&slice, rows);
    check_sums(s, rows, bf);
    check_ledgers(s, tail);
    let base = u64s(&s["variants"]["baseline"]);
    assert_eq!(base[1], 0, "baseline cross misses");
    assert_eq!(base[2], 0, "baseline identity misses");
    if doc_path == eval_doc("commit-ablation") {
        assert_eq!(base[3], 0, "self corpus: baseline invention");
    }
    let quality = u64s(&s["variants"]["quality"]);
    assert_eq!(&quality[1..4], [0, 0, 0], "quality verdict pins");
}

fn check_rows_membership(slice: &Value, rows: &[Value]) {
    let shas: std::collections::HashSet<&str> = slice["commits"]
        .as_array()
        .expect("slice")
        .iter()
        .map(|c| c["sha"].as_str().expect("sha"))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for r in rows {
        let sha = r["sha"].as_str().expect("row sha");
        assert!(shas.contains(sha), "{sha}: row outside the slice");
        assert!(seen.insert(sha), "{sha}: duplicate row");
    }
}

fn check_sums(s: &Value, rows: &[Value], below_floor: u64) {
    let gt = s["cross_gt_out"].as_u64().unwrap() + s["cross_gt_in"].as_u64().unwrap();
    for (name, _) in variants::ALL {
        let mut sum = vec![0u64; 8];
        for r in rows {
            for (i, v) in u64s(&r["variants"][name]).iter().enumerate() {
                sum[i] += v;
            }
        }
        assert_eq!(sum, u64s(&s["variants"][name]), "{name}: summary drifted");
        // reviewed below-floor lines leave every variant's ledger
        assert_eq!(
            sum[0] + sum[1],
            gt - below_floor,
            "{name}: GT total not conserved"
        );
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
