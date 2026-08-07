//! Explicitly-recorded divergence cases (plan §6 M1: "分歧 case（短路、
//! 装饰器、可选链）显式收录不回避"). Short-circuit runs live in
//! tests/metrics.rs; this file pins ce's stance on the rest, with the
//! whitepaper (v1.7) citation for each decision.

use codeeraser::scan::{functions, lang::Lang, metrics, spec};

mod common;

struct Scores {
    name: String,
    cc: u32,
    coc: u32,
}

fn measure(lang: Lang, src: &str) -> Vec<Scores> {
    let sp = spec::spec(lang);
    let tree = common::parse(lang, src);
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .into_iter()
        .map(|u| Scores {
            name: u.name,
            cc: metrics::cyclo::measure(u.node, src.as_bytes(), sp),
            coc: metrics::cognitive::measure(u.node, src.as_bytes(), sp).score,
        })
        .collect()
}

/// Decorators (whitepaper p.15, Appendix A): Sonar scores the METHOD
/// AGGREGATE — a_decorator = 1 (exception: inner def at nesting 0),
/// not_a_decorator = 2 (inner if pays +1 nesting). ce's unit model
/// (lizard-aligned) measures every def standalone from nesting 0, so
/// both hosts are 0 and both inners are 1 — the decorator exception is
/// moot by construction. Divergence recorded, not hidden.
#[test]
fn python_decorator_unit_split_stance() {
    let src = "\
def a_decorator(a, b):
    def inner(func):
        if condition:
            print(b)
        func()
    return inner

def not_a_decorator(a, b):
    my_var = a*b
    def inner(func):
        if condition:
            print(b)
        func()
    return inner
";
    let m = measure(Lang::Python, src);
    assert_eq!(m.len(), 4, "two hosts + two nested defs");
    for s in &m {
        let expect = if s.name == "inner" { 1 } else { 0 };
        assert_eq!(s.coc, expect, "{}: standalone-unit scoring", s.name);
    }
}

/// Null-coalescing / optional chaining (whitepaper p.6 "Ignore
/// shorthand"): CoC ignores both. ce's CC stance: `??` IS a branch
/// (two evaluation paths — counted, cc_operators); `?.` is not counted
/// in M1 (short-circuit member access, no comparator counts it).
#[test]
fn typescript_nullish_and_optional_chaining() {
    let src = "\
function orDefault(a: number | null, b: number): number {
  return a ?? b;
}
function pick(a: { b?: number } | null): number | undefined {
  return a?.b;
}
";
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].cc, 2, "?? counts in CC");
    assert_eq!(m[0].coc, 0, "?? ignored in CoC (p.6)");
    assert_eq!(m[1].cc, 1, "?. not counted in CC (M1 stance)");
    assert_eq!(m[1].coc, 0, "?. ignored in CoC (p.6)");
}

/// Labeled jumps (whitepaper p.8): `continue LABEL` is a fundamental
/// +1; a plain continue is free; `break <value>` (Rust) must not count
/// — its child is an expression, not a label.
#[test]
fn rust_labeled_jumps() {
    let src = "\
fn labeled() {
    'outer: for _ in 0..3 {
        for _ in 0..3 {
            continue 'outer;
        }
    }
}
fn plain() {
    for _ in 0..3 {
        for _ in 0..3 {
            continue;
        }
    }
}
fn break_with_value() -> i32 {
    loop {
        break 1;
    }
}
";
    let m = measure(Lang::Rust, src);
    assert_eq!(m.len(), 3);
    assert_eq!(m[0].coc, 4, "for +1, for +2, continue 'outer +1");
    assert_eq!(m[1].coc, 3, "plain continue is free");
    assert_eq!(m[2].coc, 1, "break-with-value is not a labeled jump");
}

#[test]
fn typescript_labeled_jump() {
    let src = "\
function drain(): void {
  outer: for (;;) {
    for (;;) {
      continue outer;
    }
  }
}
";
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 4, "for +1, for +2, continue outer +1");
}

/// Rust let-chains (2024 edition): the `&&` between let-conditions are
/// anonymous tokens with no `operator` field, so they were invisible
/// to both metrics until the chain_kinds mechanism (attack-review
/// finding — ce's own code uses this idiom). CC: one +1 per joint;
/// CoC: the chain is one operator run.
#[test]
fn rust_let_chain_counts() {
    let src = "\
fn f(x: Option<i32>, y: Option<i32>) -> i32 {
    if let Some(a) = x
        && let Some(b) = y
        && a > 0
    {
        a + b
    } else {
        0
    }
}
";
    let m = measure(Lang::Rust, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].cc, 4, "1 + if + two anonymous && joints");
    assert_eq!(m[0].coc, 3, "if +1, chain run +1, else +1");
}

/// Python `else` on for/while/try: ce scores the whitepaper hybrid
/// flat +1 — the clause is a real branch in reading flow. The
/// whitepaper (Java-centric) is silent on loop/try else; stance
/// recorded, not hidden.
#[test]
fn python_loop_and_try_else_stance() {
    let src = "\
def search(xs):
    for x in xs:
        if x:
            break
    else:
        return None
    try:
        return x
    except ValueError:
        return None
    else:
        pass
";
    let m = measure(Lang::Python, src);
    assert_eq!(m.len(), 1);
    // for +1, if +2, for-else +1, except +1, try-else +1 = 6
    assert_eq!(m[0].coc, 6, "loop/try else each pay flat +1");
}

#[test]
fn go_goto_counts() {
    let src = "\
func retry() {
	i := 0
L:
	i++
	if i < 3 {
		goto L
	}
}
";
    let m = measure(Lang::Go, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 2, "if +1, goto +1");
}
