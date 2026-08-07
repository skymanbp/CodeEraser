//! Metric unit tests: hand-computed expectations on small samples,
//! one per launch language. Expected values derive from the metric
//! definitions (gocyclo-style CC; Sonar whitepaper CoC) — external
//! tool cross-checks (lizard / rust-code-analysis / gocyclo) land in
//! the M1 acceptance fixtures, not here.

use codeeraser::scan::{functions, lang::Lang, metrics, spec};

struct Measured {
    name: String,
    lines: usize,
    params: usize,
    cc: u32,
    coc: u32,
    nesting: u32,
}

fn measure(lang: Lang, src: &str) -> Vec<Measured> {
    let sp = spec::spec(lang);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar().expect("grammar"))
        .expect("set_language");
    let tree = parser.parse(src, None).expect("parse");
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .into_iter()
        .map(|u| {
            let cog = metrics::cognitive::measure(u.node, src.as_bytes(), sp);
            Measured {
                name: u.name,
                lines: u.end_line - u.start_line + 1,
                params: u.params,
                cc: metrics::cyclo::measure(u.node, src.as_bytes(), sp),
                coc: cog.score,
                nesting: cog.max_nesting,
            }
        })
        .collect()
}

#[test]
fn python_nested() {
    let src = "\
def f(a, b):
    if a and b:
        for i in range(10):
            if i > 5:
                return i
    return 0
";
    let m = measure(Lang::Python, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].name, "f");
    assert_eq!(m[0].lines, 6);
    assert_eq!(m[0].params, 2);
    // CC: 1 + if + and + for + if
    assert_eq!(m[0].cc, 5);
    // CoC: if(+1) and(+1) for(+2) if(+3)
    assert_eq!(m[0].coc, 7);
    assert_eq!(m[0].nesting, 3);
}

#[test]
fn typescript_else_if_chain() {
    let src = "\
function f(a: number, b: number): number {
  if (a > 0 && b > 0) {
    return a + b;
  } else if (a > 0 || b > 0) {
    return 1;
  }
  return 0;
}
";
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 1);
    // CC: 1 + if + && + if + ||
    assert_eq!(m[0].cc, 5);
    // CoC: if(+1) &&(+1) else-if(flat +1) ||(+1) — no double count
    assert_eq!(m[0].coc, 4);
    assert_eq!(m[0].params, 2);
}

#[test]
fn rust_match_with_guard() {
    let src = "\
fn f(a: i32) -> i32 {
    match a {
        0 => 0,
        1 => 1,
        _ => {
            if a > 10 && a < 100 { 2 } else { 3 }
        }
    }
}
";
    let m = measure(Lang::Rust, src);
    assert_eq!(m.len(), 1);
    // CC: 1 + 3 match arms + if + &&
    assert_eq!(m[0].cc, 6);
    // CoC: match(+1) if(+2, nested) &&(+1) else(+1)
    assert_eq!(m[0].coc, 5);
    assert_eq!(m[0].nesting, 2);
}

#[test]
fn go_else_chain_and_mixed_operators() {
    let src = "\
func f(a int, b int) int {
\tif a > 0 && b > 0 {
\t\treturn a + b
\t} else if a > 0 || b > 0 {
\t\treturn 1
\t} else {
\t\treturn 2
\t}
}
";
    let m = measure(Lang::Go, src);
    assert_eq!(m.len(), 1);
    // CC: 1 + if + && + if(else-if) + ||
    assert_eq!(m[0].cc, 5);
    // CoC: if(+1) &&(+1) else-if(flat +1) ||(+1) else(field-else +1)
    assert_eq!(m[0].coc, 5);
    assert_eq!(m[0].params, 2);
}

#[test]
fn boolean_operator_runs() {
    // a && b && c = one run; a || b && c || d = three runs (Sonar rule 4)
    let src = "\
def same(a, b, c):
    return a and b and c

def mixed(a, b, c, d):
    return a or b and c or d
";
    let m = measure(Lang::Python, src);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].coc, 1, "like operators count once");
    assert_eq!(m[1].coc, 3, "each alternation counts");
}

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
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 2, "outer arrow + inner arrow are separate units");
    let outer = m.iter().find(|f| f.name == "outer").expect("outer");
    let inner = m.iter().find(|f| f.name == "(anonymous)").expect("inner");
    // outer: if only — the inner arrow's ternary must not leak in
    assert_eq!(outer.cc, 2);
    assert_eq!(inner.cc, 2, "1 + ternary");
}
