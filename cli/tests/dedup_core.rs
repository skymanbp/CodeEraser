//! Dedup hot-path core tests: T2 normalization invariance and the
//! winnowing correctness bounds (SIGMOD'03: every match of >= t
//! tokens is caught; nothing below the k-gram noise floor is).

use codeeraser::dedup::{Params, tokens, winnow};
use codeeraser::scan::{lang::Lang, spec};
use std::collections::BTreeSet;

mod common;

fn toks(lang: Lang, src: &str) -> Vec<u64> {
    let sp = spec::spec(lang);
    let tree = common::parse(lang, src);
    tokens::tokenize(tree.root_node(), sp)
        .iter()
        .map(|t| t.hash)
        .collect()
}

fn fp_set(tokens: &[u64], p: Params) -> BTreeSet<u64> {
    winnow::fingerprints(tokens, p)
        .into_iter()
        .map(|f| f.hash)
        .collect()
}

/// T2 clone: renamed identifiers, different literals, an extra
/// comment — the normalized streams must be identical.
#[test]
fn t2_normalization_invariance() {
    let a = "\
fn total(items: &[i32], factor: i32) -> i32 {
    let mut acc = 0;
    for item in items {
        acc += item * factor + 10;
    }
    acc
}
";
    let b = "\
fn sum_up(xs: &[i32], scale: i32) -> i32 {
    // completely different comment
    let mut out = 0;
    for x in xs {
        out += x * scale + 9999;
    }
    out
}
";
    assert_eq!(toks(Lang::Rust, a), toks(Lang::Rust, b));
}

/// Whole strings collapse to one LIT each; f-string interpolation
/// keeps its `{ID}` structure.
#[test]
fn python_string_collapse() {
    let a = "def f(x):\n    return \"a\" + f\"b{x}c\"\n";
    let b = "def g(y):\n    return \"zzzzz\" + f\"wwww{y}vv\"\n";
    assert_eq!(toks(Lang::Python, a), toks(Lang::Python, b));
}

/// Different operators are different streams — normalization must not
/// erase structure.
#[test]
fn structure_is_preserved() {
    let a = "def f(a, b):\n    return a + b\n";
    let b = "def f(a, b):\n    return a * b\n";
    assert_ne!(toks(Lang::Python, a), toks(Lang::Python, b));
}

/// Attack-review D3 regression: a literal STATEMENT after a literal
/// must stay a separate token — lexical-adjacency merging swallowed
/// Python attribute docstrings and TS ASI directives entirely.
#[test]
fn literal_statement_not_swallowed() {
    let with_doc = "X = \"abc\"\n\"\"\"Attribute docstring.\"\"\"\nY = 1\n";
    let without = "X = \"abc\"\nY = 1\n";
    assert_ne!(
        toks(Lang::Python, with_doc),
        toks(Lang::Python, without),
        "the docstring statement must appear in the stream"
    );
    let ts_directive = "const a = 'x'\n'use strict'\nconst b = 2\n";
    let ts_plain = "const a = 'x'\nconst b = 2\n";
    assert_ne!(
        toks(Lang::TypeScript, ts_directive),
        toks(Lang::TypeScript, ts_plain)
    );
}

/// Attack-review nit-14 regression: the Rust lifetime tick `'` is not
/// a string delimiter — `&'a str` signatures must differ from `&str`.
#[test]
fn rust_lifetime_is_not_a_literal() {
    let with_lt = "fn f<'a>(x: &'a str) -> &'a str { x }\n";
    let without = "fn f(x: &str) -> &str { x }\n";
    let a = toks(Lang::Rust, with_lt);
    let b = toks(Lang::Rust, without);
    assert_ne!(a, b, "lifetime ticks must not collapse into LIT");
}

/// Guarantee bound: a shared run of exactly t = window + kgram - 1
/// tokens embedded in otherwise-disjoint sequences must share at
/// least one fingerprint.
#[test]
fn winnowing_guarantee_threshold() {
    let p = Params::default();
    let t = p.window + p.kgram - 1;
    let shared: Vec<u64> = (10_000..10_000 + t as u64).collect();
    let mut a: Vec<u64> = (1..40).collect();
    a.extend(&shared);
    a.extend(2_000..2_040u64);
    let mut b: Vec<u64> = (5_000..5_070).collect();
    b.extend(&shared);
    b.extend(7_000..7_020u64);
    let common: Vec<u64> = fp_set(&a, p)
        .intersection(&fp_set(&b, p))
        .copied()
        .collect();
    assert!(!common.is_empty(), "t-token match must share a fingerprint");
}

/// Noise floor: a shared run shorter than one k-gram can never
/// produce a shared fingerprint.
#[test]
fn winnowing_noise_floor() {
    let p = Params::default();
    let shared: Vec<u64> = (10_000..10_010).collect(); // 10 < kgram 25
    let mut a: Vec<u64> = (1..100).collect();
    a.extend(&shared);
    let mut b: Vec<u64> = (5_000..5_100).collect();
    b.extend(&shared);
    let common: Vec<u64> = fp_set(&a, p)
        .intersection(&fp_set(&b, p))
        .copied()
        .collect();
    assert!(common.is_empty(), "sub-kgram overlap must stay invisible");
}

/// T1 end-to-end through the real tokenizer: identical TS files give
/// identical fingerprint sets.
#[test]
fn t1_identical_files_match() {
    let src = "\
export function toRegexp(antPattern: string, sep: string): string {
  let sb = \"^\";
  let i = antPattern.startsWith(\"/\") ? 1 : 0;
  while (i < antPattern.length) {
    const ch = antPattern.charAt(i);
    if (ch === \"*\") {
      sb += \".*\";
    } else if (ch === \"?\") {
      sb += \".\";
    } else {
      sb += ch;
    }
    i++;
  }
  return sb;
}
";
    let p = Params::default();
    let a = toks(Lang::TypeScript, src);
    assert!(a.len() >= p.window + p.kgram - 1, "sample long enough");
    assert_eq!(fp_set(&a, p), fp_set(&toks(Lang::TypeScript, src), p));
    assert!(!fp_set(&a, p).is_empty());
}
