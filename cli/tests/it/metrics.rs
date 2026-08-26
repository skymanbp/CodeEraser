//! Metric unit tests: hand-computed expectations on small samples,
//! one per launch language. Expected values derive from the metric
//! definitions (gocyclo-style CC; Sonar whitepaper CoC) — external
//! tool cross-checks (lizard / rust-code-analysis / gocyclo) land in
//! the M1 acceptance fixtures, not here. The rows ride the shared
//! table runner (common::run_metric_cases).

use crate::common;
use codeeraser::scan::lang::Lang;

const CASES: &[common::MetricCase] = &[
    common::MetricCase {
        lang: Lang::Python,
        src: "\
def f(a, b):
    if a and b:
        for i in range(10):
            if i > 5:
                return i
    return 0
",
        fns: 1,
        checks: &[
            (0, "lines", 6, "def through final return"),
            (0, "params", 2, "a, b"),
            (0, "cc", 5, "1 + if + and + for + if"),
            (0, "coc", 7, "if +1, and +1, for +2, if +3"),
            (0, "nesting", 3, "if > for > if"),
        ],
    },
    common::MetricCase {
        lang: Lang::TypeScript,
        src: "\
function f(a: number, b: number): number {
  if (a > 0 && b > 0) {
    return a + b;
  } else if (a > 0 || b > 0) {
    return 1;
  }
  return 0;
}
",
        fns: 1,
        checks: &[
            (0, "cc", 5, "1 + if + && + if + ||"),
            (0, "coc", 4, "if +1, && +1, else-if flat +1, || +1"),
            (0, "params", 2, "a, b"),
        ],
    },
    common::MetricCase {
        lang: Lang::Rust,
        src: "\
fn f(a: i32) -> i32 {
    match a {
        0 => 0,
        1 => 1,
        _ => {
            if a > 10 && a < 100 { 2 } else { 3 }
        }
    }
}
",
        fns: 1,
        checks: &[
            (0, "cc", 6, "1 + 3 match arms + if + &&"),
            (0, "coc", 5, "match +1, if +2 nested, && +1, else +1"),
            (0, "nesting", 2, "match > if"),
        ],
    },
    common::MetricCase {
        lang: Lang::Go,
        src: "\
func f(a int, b int) int {
\tif a > 0 && b > 0 {
\t\treturn a + b
\t} else if a > 0 || b > 0 {
\t\treturn 1
\t} else {
\t\treturn 2
\t}
}
",
        fns: 1,
        checks: &[
            (0, "cc", 5, "1 + if + && + if(else-if) + ||"),
            (0, "coc", 5, "if +1, && +1, else-if +1, || +1, else +1"),
            (0, "params", 2, "a, b"),
        ],
    },
    // a && b && c = one run; a || b && c || d = three runs (Sonar
    // rule 4).
    common::MetricCase {
        lang: Lang::Python,
        src: "\
def same(a, b, c):
    return a and b and c

def mixed(a, b, c, d):
    return a or b and c or d
",
        fns: 2,
        checks: &[
            (0, "coc", 1, "like operators count once"),
            (1, "coc", 3, "each alternation counts"),
        ],
    },
    // M1 cross-check finding: comprehension clauses are branch paths
    // (lizard/radon count them in CC); CoC stays 0 (declarative).
    common::MetricCase {
        lang: Lang::Python,
        src: "\
def f(xs):
    return [x for x in xs if x > 0]
",
        fns: 1,
        checks: &[
            (0, "cc", 3, "1 + for_in_clause + if_clause"),
            (0, "coc", 0, "no cognitive penalty for comprehensions"),
        ],
    },
    // M1 cross-check finding: `expr?` is an implicit early-return
    // branch (rust-code-analysis counts it; ban.rs check 21 vs 17 ==
    // 4x `?`).
    common::MetricCase {
        lang: Lang::Rust,
        src: "\
fn f(s: &str) -> Result<i32, std::num::ParseIntError> {
    let a: i32 = s.parse()?;
    Ok(a + 1)
}
",
        fns: 1,
        checks: &[(0, "cc", 2, "1 + try_expression")],
    },
];

#[test]
fn metric_hand_computed_samples() {
    common::run_metric_cases(CASES);
}

/// Bespoke: units are found by NAME (extraction order of sibling
/// arrows is not part of the contract) — the inner arrow's ternary
/// must not leak into the outer unit's CC.
#[test]
fn nested_standalone_fn_not_double_counted() {
    let src = "\
const outer = (xs: number[]) => {
  if (xs.length > 0) {
    return xs.map((x) => (x > 0 ? x : -x));
  }
  return [];
};
";
    let m = common::measure_units(Lang::TypeScript, src);
    assert_eq!(m.len(), 2, "outer arrow + inner arrow are separate units");
    let outer = m.iter().find(|f| f.name == "outer").expect("outer");
    let inner = m.iter().find(|f| f.name == "(anonymous)").expect("inner");
    assert_eq!(outer.cc, 2);
    assert_eq!(inner.cc, 2, "1 + ternary");
}
