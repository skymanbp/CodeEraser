//! Pair-layer extension-verify tests. The synthetic-stream cases pin
//! extend() semantics exactly; the pipeline cases replay the two
//! round-1 jscpd misses (DEDUP-CALIBRATION.md) as regressions.

use codeeraser::dedup::{Params, index::Instance, pairs, tokens};
use codeeraser::scan::lang::Lang;

fn tok(hash: u64, line: usize) -> tokens::Token {
    tokens::Token {
        hash,
        start_line: line,
        end_line: line,
    }
}

fn synth(values: &[u64]) -> Vec<tokens::Token> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| tok(*v, i + 1))
        .collect()
}

fn anchor(file: &str, hash: u64, start_tok: usize) -> Instance {
    Instance {
        hash,
        file: file.into(),
        start_tok,
        start_line: start_tok + 1,
        end_line: start_tok + 1,
    }
}

/// Attack-review D1 regression: an anchor whose stored token offset
/// exceeds the live stream (file shrank after indexing) is skipped
/// and counted — never an out-of-bounds panic.
#[test]
fn stale_anchor_skipped_not_panicked() {
    let stream: Vec<u64> = (1..60).collect();
    let mut streams = pairs::Streams::new();
    streams.insert("a.rs".into(), synth(&stream));
    streams.insert("b.rs".into(), synth(&stream));
    // offset 61 is beyond the 59-token stream
    let instances = vec![anchor("a.rs", 9, 61), anchor("b.rs", 9, 10)];
    let found = pairs::clone_blocks(&instances, &streams, Params::default().guarantee());
    assert!(found.blocks.is_empty());
    assert_eq!(found.stale_skipped, 1, "stale anchor counted, not fatal");
}

/// Attack-review D4 regression: a hash shared by MORE than HOT_CAP
/// locations chains adjacent pairs instead of vanishing — 70 copies
/// of the same content must still be reported, not zeroed.
#[test]
fn hot_group_chains_instead_of_vanishing() {
    let shared: Vec<u64> = (1_000..1_060).collect();
    let mut streams = pairs::Streams::new();
    let mut instances = Vec::new();
    for i in 0..70 {
        let name = format!("f{i:02}.rs");
        streams.insert(name.clone(), synth(&shared));
        instances.push(anchor(&name, 7, 5));
    }
    let found = pairs::clone_blocks(&instances, &streams, Params::default().guarantee());
    assert_eq!(found.hot_chained, 1, "one hot hash group chained");
    assert_eq!(found.blocks.len(), 69, "adjacent chain covers all copies");
}

/// Shared run of 30 tokens (>= kgram noise floor, < t=50): the anchor
/// exists but extension verification must reject it.
#[test]
fn sub_threshold_run_rejected() {
    let shared: Vec<u64> = (1_000..1_030).collect();
    let mut a: Vec<u64> = (1..41).collect();
    a.extend(&shared);
    a.extend(2_000..2_010u64);
    let mut b: Vec<u64> = (500..510).collect();
    b.extend(&shared);
    b.extend(3_000..3_040u64);
    let mut streams = pairs::Streams::new();
    streams.insert("a.rs".into(), synth(&a));
    streams.insert("b.rs".into(), synth(&b));
    let instances = vec![anchor("a.rs", 42, 40), anchor("b.rs", 42, 10)];
    let found = pairs::clone_blocks(&instances, &streams, Params::default().guarantee());
    assert!(found.blocks.is_empty(), "30 < t must not be reported");
}

/// Shared run of exactly 60 tokens: reported with the EXACT verified
/// length, regardless of where in the run the anchor sits.
#[test]
fn verified_run_has_exact_length() {
    let shared: Vec<u64> = (1_000..1_060).collect();
    let mut a: Vec<u64> = (1..41).collect();
    a.extend(&shared);
    let mut b: Vec<u64> = (500..520).collect();
    b.extend(&shared);
    b.extend(3_000..3_010u64);
    let mut streams = pairs::Streams::new();
    streams.insert("a.rs".into(), synth(&a));
    streams.insert("b.rs".into(), synth(&b));
    // anchor deep inside the run: extension must recover the full 60
    let instances = vec![anchor("a.rs", 7, 55), anchor("b.rs", 7, 35)];
    let found = pairs::clone_blocks(&instances, &streams, Params::default().guarantee());
    assert_eq!(found.blocks.len(), 1);
    assert_eq!(found.blocks[0].tokens, 60, "maximal exact extension");
}

/// Round-1 miss #1 regression (models.py 837<->847 shape): the same
/// stanza twice back-to-back must be reported as the two adjacent
/// disjoint halves, not killed by a self-overlap filter.
#[test]
fn adjacent_self_repeat_detected() {
    let stanza = "\
fn work(input: &[i64], limit: i64) -> i64 {
    let mut total = 0;
    for value in input {
        if *value > limit {
            total += value * 2 + 7;
        } else {
            total -= value / 3;
        }
    }
    total
}
";
    let doubled = format!("{stanza}{stanza}");
    let p = Params::default();
    let stream = tokens::stream(doubled.as_bytes(), Lang::Rust).expect("stream");
    let period = stream.len() / 2;
    assert!(period >= p.guarantee(), "stanza long enough for the test");
    let hashes: Vec<u64> = stream.iter().map(|t| t.hash).collect();
    let mut streams = pairs::Streams::new();
    streams.insert("f.rs".into(), stream);
    let instances: Vec<Instance> = codeeraser::dedup::winnow::fingerprints(&hashes, p)
        .into_iter()
        .map(|f| anchor("f.rs", f.hash, f.start))
        .collect();
    let found = pairs::clone_blocks(&instances, &streams, p.guarantee());
    assert_eq!(found.blocks.len(), 1, "one maximal adjacent pair");
    let b = &found.blocks[0];
    assert_eq!(b.tokens, period, "capped at the anchor gap");
    assert!(b.a_end <= b.b_start, "ranges stay disjoint");
}

/// Round-1 miss #2 regression (walk.rs boilerplate shape): three T2
/// copies in one file. A fully periodic sequence has exactly TWO
/// maximal runs — one per offset class (P and 2P): left-maximality
/// pulls the would-be P2<->P3 pair onto the P1<->P2 run, which the
/// dedupe collapses. No fragmentation, no cross-region chaining.
#[test]
fn three_copies_yield_offset_class_runs() {
    let stanza = |seed: u32| {
        format!(
            "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
        )
    };
    let src = format!("{}{}{}", stanza(1), stanza(2), stanza(3));
    let p = Params::default();
    let stream = tokens::stream(src.as_bytes(), Lang::Rust).expect("stream");
    let period = stream.len() / 3;
    assert!(period >= p.guarantee(), "stanza long enough for the test");
    let hashes: Vec<u64> = stream.iter().map(|t| t.hash).collect();
    let mut streams = pairs::Streams::new();
    streams.insert("f.rs".into(), stream);
    let instances: Vec<Instance> = codeeraser::dedup::winnow::fingerprints(&hashes, p)
        .into_iter()
        .map(|f| anchor("f.rs", f.hash, f.start))
        .collect();
    let found = pairs::clone_blocks(&instances, &streams, p.guarantee());
    assert_eq!(found.blocks.len(), 2, "one maximal run per offset class");
    for b in &found.blocks {
        assert_eq!(b.tokens, period, "len capped at one period (disjoint)");
        assert!(b.a_end <= b.b_start, "disjoint ranges");
        assert_eq!(b.a_start, found.blocks[0].a_start, "both anchor at P1");
    }
}
