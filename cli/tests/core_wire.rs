//! Cross-language wire goldens: the fixture files under
//! contracts/fixtures/ alternate request line / expected reply line
//! and are consumed byte-for-byte by BOTH this test and the Haskell
//! suite (core/test/Spec.hs) — drift on either side reddens both.
//! Byte comparison is sound because the freeze pins
//! `aeson +ordered-keymap` (deterministic key order).
//!
//! Requires a built ce-core: the gate must not silently skip, so a
//! missing CE_CORE_BIN is a loud failure, not an ignore.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

fn fixture_pairs(path: &str) -> Vec<(String, String)> {
    let raw = std::fs::read_to_string(path).expect(path);
    let lines: Vec<&str> = raw.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len() % 2,
        0,
        "{path}: fixtures are request/reply pairs"
    );
    lines
        .chunks(2)
        .map(|c| (c[0].to_string(), c[1].to_string()))
        .collect()
}

/// One core process answers every fixture pair in order — this also
/// exercises the persistent-link shape (N requests, one child).
#[test]
fn wire_goldens_roundtrip() {
    let mut child = Command::new(core_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn ce-core");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut out = BufReader::new(child.stdout.take().expect("stdout"));
    for file in [
        "../contracts/fixtures/handshake/hello-ok.ndjson",
        "../contracts/fixtures/handshake/wire-errors.ndjson",
        "../contracts/fixtures/fourclass/golden.ndjson",
        "../contracts/fixtures/graph/golden.ndjson",
        "../contracts/fixtures/clone/golden.ndjson",
        "../contracts/fixtures/docdup/golden.ndjson",
        "../contracts/fixtures/verdict/golden.ndjson",
        "../contracts/fixtures/scan/golden.ndjson",
        "../contracts/fixtures/structure/golden.ndjson",
        "../contracts/fixtures/trend/golden.ndjson",
    ] {
        for (n, (request, expected)) in fixture_pairs(file).into_iter().enumerate() {
            writeln!(stdin, "{request}").expect("write");
            stdin.flush().expect("flush");
            let mut reply = String::new();
            out.read_line(&mut reply).expect("read");
            assert_eq!(reply.trim_end(), expected, "{file} pair {}", n + 1);
        }
    }
    drop(stdin); // EOF => core exits
    let status = child.wait().expect("wait");
    assert!(status.success(), "core exit: {status}");
}

/// The corelink client against the real core: capabilities arrive,
/// a supported round-trip echoes id/type, and an unsupported request
/// comes back as a visible desync error (the L1-fallback trigger,
/// A9f), not a hang or a wrong answer.
#[test]
fn corelink_open_and_desync() {
    let (mut link, reply) = codeeraser::corelink::Link::open(&core_bin()).expect("open");
    assert!(reply.accept, "hello accepted");
    // the FULL version welds here, not just the negotiated major —
    // clearance review: §1's "pinned by the shared fixture" claim was
    // false for the Rust constant, which could advance alone with
    // every test green
    assert_eq!(
        reply.proto,
        codeeraser::corelink::PROTO,
        "client PROTO and core proto drifted"
    );
    // every declared family, one loop (fourclass/2 = M5-1c anchor
    // shape; graph/1 = M5-2a; clone/docdup/verdict = M5-3a; scan/1
    // = ADR-008 P3; structure/1 = M6 S2; trend/1 = M7.5b) — the
    // per-cap assert ladder tripped the self-ratchet at row eight
    for cap in [
        "hello",
        "fourclass/2",
        "graph/1",
        "clone/1",
        "docdup/1",
        "verdict/1",
        "scan/1",
        "structure/1",
        "trend/1",
    ] {
        assert!(link.has(cap), "capability {cap} declared");
    }
    let ok = link
        .request("fourclass", serde_json::json!({"pairs": []}))
        .expect("empty batch round-trips");
    assert_eq!(ok["moved"], serde_json::json!([]));
    let err = link
        .request("mystery", serde_json::json!({}))
        .expect_err("unsupported type must not produce a result");
    // the refusal surfaces the core's NAMED code since 2.8.0 (review
    // C4: "desync" used to swallow every error/contract message)
    assert!(err.contains("unknown_type"), "named refusal, got: {err}");
}

/// ADR-008 (first step + P4): EMPTY knob tables make the core judge
/// with its Cost.hs defaults, and the FULL echo must equal the set
/// pinned here — ONE assertion that reddens when either side moves
/// alone (audit D2 for the 300/15 pair; P4 extends the gate to the
/// whole knob face). The ceilings half reads ce.toml's
/// `Thresholds::default()` — the config IS its source; the P4 knobs
/// have no config default (absent key = core default), so this
/// literal object is their pinned mirror, the drift check itself.
#[test]
fn knob_default_drift_gate() {
    let t = codeeraser::config::Thresholds::default();
    let (mut link, _) = codeeraser::corelink::Link::open(&core_bin()).expect("open");
    let reply = link
        .request(
            "verdict",
            serde_json::json!({
                "sim": [], "pos": [], "tier": [], "churn": [], "cochange": [],
                "continuous": [], "discrete": [], "baseline": null,
                "weights": [], "floor": null, "ceilings": [],
                "thresholds": [], "tolerance": []
            }),
        )
        .expect("verdict round-trip");
    assert_eq!(
        reply["knobs"],
        serde_json::json!({
            "sizeCeil": t.file_lines_warn, "cocCeil": t.cognitive_warn,
            "deadIndegCeil": 0, "rewriteNum": 50, "rewriteDen": 100,
            "cochangeFloor": 2, "violCost": 10, "defaultWeight": 1,
            "scoreScale": 1000, "tolNum": 102, "tolDen": 100, "tolAbs": 10,
        }),
        "core Cost.hs defaults drifted from the pinned knob face"
    );
}

/// A two-pair batch in the cross-pair shape every bucket test uses:
/// one pair pure removal, one pair pure addition (Rust lang).
fn removal_addition_batch<'a>(
    removed: &'a str,
    added: &'a str,
) -> [codeeraser::fourclass::batch::PairInput<'a>; 2] {
    use codeeraser::fourclass::batch::PairInput;
    use codeeraser::scan::lang::Lang;
    [
        PairInput {
            before: removed,
            after: "",
            lang: Lang::Rust,
        },
        PairInput {
            before: "",
            after: added,
            lang: Lang::Rust,
        },
    ]
}

/// The product cap's motivating shapes stay UN-degraded (Codex
/// review C4: previously nothing pinned them, so reverting to the
/// old per-side occurrence count would have passed every test).
/// Both are real ripgrep shapes the per-side form mislabeled:
/// b9de003f8 adds 66 identical lines with no removed partner
/// (product 0); 082245dad pairs 5 removals with 118 additions
/// (product 590 < 64^2 despite one side > 64).
#[test]
fn product_cap_spares_one_sided_and_small_product_shapes() {
    use codeeraser::fourclass::batch::classify_batch;
    let (mut link, _) = codeeraser::corelink::Link::open(&core_bin()).expect("open");
    let pile = "#[inline(always)]\n".repeat(66);
    let one_sided = removal_addition_batch("", &pile);
    let out = classify_batch(&one_sided, Some(&mut link));
    assert_eq!(out.degraded, None, "one-sided pile must not degrade");
    let removed = "#[derive(Debug)]\n".repeat(5);
    let added = "#[derive(Debug)]\n".repeat(118);
    let small_product = removal_addition_batch(&removed, &added);
    let out = classify_batch(&small_product, Some(&mut link));
    assert_eq!(out.degraded, None, "small-product bucket must not degrade");
}

/// Attack-review F3: a bucket-cap reply may still carry the partial
/// blocks the core derived from uncapped hashes — the client must
/// return PURE L1 plus the visible reason, never partial L2.
#[test]
fn degraded_reply_keeps_l1_pure() {
    use codeeraser::fourclass::batch::classify_batch;
    // 65 identical removals x 65 identical additions of one content =
    // pairing product 4225 > 64^2, tripping the per-hash work budget
    // (one-sided or small-product piles are exempt — the product is
    // the work), while alpha/beta form a block the core still derives
    // alongside it.
    let flood = "let flood_line = 1;\n".repeat(65);
    let before_a = format!("{flood}let alpha = 2;\nlet beta = 3;\n");
    let after_b = format!("{flood}let alpha = 2;\nlet beta = 3;\n");
    let inputs = removal_addition_batch(&before_a, &after_b);
    let l1 = classify_batch(&inputs, None);
    let (mut link, _) = codeeraser::corelink::Link::open(&core_bin()).expect("open");
    let l2 = classify_batch(&inputs, Some(&mut link));
    assert_eq!(l2.degraded.as_deref(), Some("bucket_cap"), "cap visible");
    assert!(l2.relocations.is_empty(), "no partial relocations");
    for (a, b) in l1.pairs.iter().zip(&l2.pairs) {
        assert_eq!(
            (a.counts.added_novel, a.counts.removed_deleted),
            (b.counts.added_novel, b.counts.removed_deleted),
            "degraded batch is bitwise L1"
        );
        assert_eq!(a.moved.len(), b.moved.len(), "no partial moved delta");
    }
}
