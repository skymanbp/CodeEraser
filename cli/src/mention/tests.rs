//! The tokenizer and binary-rule legs of the sealed criterion (§2's
//! frozen witnesses, K24's tokenizer half, K39's text cases). Each
//! case is `(extension, text) ⇒ tokens in emission order`; the JS-arm
//! cases pair one JS extension with one union extension over the SAME
//! text, so the arm is the only thing that moves.

use super::token::{FOLD_MIN_CHARS, dedup_suffixed, emit, fold, segments, whole_run_only};
use super::walk::decode;

/// The fold gate's segmenter, on its own frozen witnesses: `_`
/// boundaries, a camel rise, an all-caps run as ONE segment (so
/// `HTTPServer` is two and `RULES` one), a digit-led tail.
#[test]
fn the_segmenter_counts_underscore_and_camel_boundaries() {
    for (name, want) in [
        ("scan_row_cap", 3),
        ("PyProject", 2),
        ("HTTPServer", 2),
        ("RULES", 1),
        ("Level", 1),
        ("_2abc_defg", 2),
    ] {
        assert_eq!(segments(name), want, "{name}");
    }
}

fn toks(file: &str, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    emit(text, whole_run_only(file), &mut |t| out.push(t.to_string()));
    out
}

/// The three witnesses the criterion froze, plus the reference
/// probe's edge rows: `$` in both alphabets, the script split never
/// minting a bare `graph`, a single `$` piece, and the digit-led drop.
#[test]
fn the_frozen_witnesses_emit_exactly_the_sealed_tokens() {
    assert_eq!(toks("a.md", "$ZodString"), ["$ZodString", "ZodString"]);
    assert_eq!(toks("a.ts", "$ZodString"), ["$ZodString"]);
    assert_eq!(
        toks("a.md", "调用$graph函数"),
        ["调用$graph函数", "$graph", "调用", "graph函数"]
    );
    assert_eq!(toks("a.ts", "调用$graph函数"), ["调用$graph函数", "$graph"]);
    assert_eq!(toks("a.md", "ключ$"), ["ключ$", "$", "ключ"]);
    assert_eq!(
        toks("a.md", "调用graph_report函数"),
        ["调用graph_report函数", "graph_report"]
    );
    assert_eq!(toks("a.rs", "$foo"), ["$foo", "foo"]);
    assert_eq!(toks("a.md", "$1"), ["$1"]);
    assert_eq!(toks("a.md", "_1x"), ["_1x"]);
    assert_eq!(toks("a.md", "2FA"), ["FA"]);
    assert_eq!(toks("a.md", "1a$b"), ["a$b", "a", "b"]);
    assert_eq!(toks("a.md", "中2abcdefg"), ["中2abcdefg"]);
    assert_eq!(toks("a.rs", "_2abc_defg"), ["_2abc_defg"]);
}

/// The union arm's reason to exist (K24): a shell script's `$name` and
/// a Haskell `f$g` keep the bare names; runs are cut on the same
/// alphabet whatever the file, and prose around them is ordinary.
#[test]
fn the_union_arm_keeps_the_dollar_free_names() {
    assert_eq!(
        toks("Makefile", "exec $ce_entry_main --x"),
        ["exec", "$ce_entry_main", "ce_entry_main", "x"]
    );
    assert_eq!(
        toks("M.hs", "putStrLn$fmtRow"),
        ["putStrLn$fmtRow", "putStrLn", "fmtRow"]
    );
    assert_eq!(
        toks("a.md", "see `zod_string` (v3)"),
        ["see", "zod_string", "v3"]
    );
}

/// The `$` arm is an extension table — lower-cased lookup, no
/// extension is union, every JS-family spelling is whole-run — and
/// the K41 witness shape is a base, one `$`, then digits only. One
/// `(input, verdict)` table per predicate.
#[test]
fn extension_arm_and_suffix_witness_tables() {
    let arm = [
        ("a.ts", true),
        ("a.TS", true),
        ("a.tsx", true),
        ("a.mts", true),
        ("a.cts", true),
        ("a.js", true),
        ("a.mjs", true),
        ("a.cjs", true),
        ("a.jsx", true),
        ("a.vue", true),
        ("a.svelte", true),
        ("Makefile", false),
        (".gitignore", false),
        ("a.md", false),
        ("a.rs", false),
        ("a.hs", false),
        ("a.sh", false),
        ("a.astro", false),
        ("a.html", false),
    ];
    for (file, whole) in arm {
        assert_eq!(whole_run_only(file), whole, "{file}");
    }
    assert!(
        ["name$1", "x$12", "Zod$0"]
            .iter()
            .all(|r| dedup_suffixed(r))
    );
    assert!(
        !["$1", "name$", "a$b1", "name", "a$1$b"]
            .iter()
            .any(|r| dedup_suffixed(r))
    );
}

/// The fold key filters `_`, `-` AND `$` and lower-cases; the length
/// gate is seven LITERAL characters, judged before the filter.
#[test]
fn fold_filters_the_three_separators_and_the_gate_is_literal_length() {
    let keys = [
        ("$ZodString", "zodstring"),
        ("zod_string", "zodstring"),
        ("Kebab-Case", "kebabcase"),
        ("_2abc_defg", "2abcdefg"),
    ];
    assert!(keys.iter().all(|(token, key)| fold(token) == *key));
    assert_eq!(FOLD_MIN_CHARS, 7);
    assert_eq!(
        "$ZodStr".chars().count(),
        7,
        "fills; its fold `zodstr` is not re-judged"
    );
    assert_eq!(
        "__init__".chars().count(),
        8,
        "fills although the fold is 4"
    );
}

/// The caps are explicit numbers of the pass and the face carries its
/// schema id — the two facts the operator's window states.
#[test]
fn caps_and_face_identity_are_stated() {
    assert_eq!(super::FILE_TOKEN_CAP, 65_536);
    assert_eq!(super::TABLE_ROW_CAP, 4_194_304);
    let doc = super::face::report_json(&super::Stats::default());
    assert!(
        doc.contains("\"schema\":\"ce.mentions-report/0.1.0\""),
        "{doc}"
    );
    assert!(doc.contains("\"mention_rev\":1"), "{doc}");
}

/// The binary rule: BOMs decode, an early NUL is git's verdict, a NUL
/// past byte 8000 keeps the file (`contracts/VERSIONING.md`'s case),
/// and a stray byte is lossy rather than fatal.
#[test]
fn decode_follows_the_bom_then_gits_nul_rule() {
    let le: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain(
            "héllo graph_report"
                .encode_utf16()
                .flat_map(u16::to_le_bytes),
        )
        .collect();
    assert_eq!(decode(&le).as_deref(), Some("héllo graph_report"));
    let be: Vec<u8> = [0xFE, 0xFF]
        .into_iter()
        .chain("x".encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    assert_eq!(decode(&be).as_deref(), Some("x"));
    assert_eq!(decode(b"early\0nul"), None);
    let mut late = vec![b'a'; 8100];
    late.push(0);
    assert!(decode(&late).is_some(), "NUL at 8100 is past the window");
    let mut edge = vec![b'a'; 7999];
    edge.push(0);
    assert_eq!(decode(&edge), None, "NUL at byte 7999 is inside the window");
    assert_eq!(decode(b"caf\xe9 ok").as_deref(), Some("caf\u{fffd} ok"));
}
