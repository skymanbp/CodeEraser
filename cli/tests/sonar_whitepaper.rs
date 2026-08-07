//! Sonar Cognitive Complexity whitepaper v1.7 (2023-08-29) worked
//! examples, ported to the launch languages (plan §6 M1: "CoC 过
//! Sonar 白皮书共通例题"). Every expected value below is read from the
//! whitepaper's margin annotations at the cited page, not re-derived.

use codeeraser::scan::{functions, lang::Lang, metrics, spec};

mod common;

struct Scores {
    cc: u32,
    coc: u32,
}

fn measure(lang: Lang, src: &str) -> Vec<Scores> {
    let sp = spec::spec(lang);
    let tree = common::parse(lang, src);
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .into_iter()
        .map(|u| Scores {
            cc: metrics::cyclo::measure(u.node, src.as_bytes(), sp),
            coc: metrics::cognitive::measure(u.node, src.as_bytes(), sp).score,
        })
        .collect()
}

/// p.5 + p.10 `sumOfPrimes`: CoC 7 (for +1, for +2, if +3, labeled
/// `continue OUT` +1), Cyclomatic 4. Go port keeps the labeled jump.
#[test]
fn p10_sum_of_primes() {
    let src = "\
func sumOfPrimes(max int) int {
	total := 0
OUT:
	for i := 1; i <= max; i++ {
		for j := 2; j < i; j++ {
			if i%j == 0 {
				continue OUT
			}
		}
		total += i
	}
	return total
}
";
    let m = measure(Lang::Go, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 7, "whitepaper p.10 margin total");
    assert_eq!(m[0].cc, 4, "whitepaper p.5 margin total");
}

/// p.5 + p.10 `getWords`: a switch and all its cases = one structural
/// increment, CoC 1; CC 4 (p.5 margin — default adds no branch; the
/// same rule as gocyclo v0.6.0's "ignore default case").
#[test]
fn p10_get_words() {
    let src = "\
func getWords(number int) string {
	switch number {
	case 1:
		return \"one\"
	case 2:
		return \"a couple\"
	case 3:
		return \"a few\"
	default:
		return \"lots\"
	}
}
";
    let m = measure(Lang::Go, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 1, "whitepaper p.10 margin total");
    assert_eq!(m[0].cc, 4, "whitepaper p.5 margin: default not counted");
}

/// p.8 second operator example: `if (a && !(b && c))` = 3 — the
/// negated parenthesized sub-chain starts a NEW sequence even though
/// the operator is the same. Pins ce's paren-bounded run model.
#[test]
fn p8_negated_paren_starts_new_run() {
    let src = "\
function f(a: boolean, b: boolean, c: boolean): boolean {
  if (a && !(b && c)) {
    return true;
  }
  return false;
}
";
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 3, "whitepaper p.8 margin: if +1, && +1, && +1");
}

/// p.9 `myMethod`: try is transparent (no increment, no nesting);
/// if +1, for +2, while +3, catch +1, if +2 = 9. Python try/except port.
#[test]
fn p9_my_method_try_catch() {
    let src = "\
def my_method():
    try:
        if condition1:
            for i in range(10):
                while condition2:
                    pass
    except (ValueError, TypeError):
        if condition2:
            pass
";
    let m = measure(Lang::Python, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 9, "whitepaper p.9 margin total");
}

/// p.9 `myMethod2`: a lambda increments nothing but raises nesting, so
/// the if inside it costs +2. Go func-literal port (absorbed unit).
#[test]
fn p9_lambda_raises_nesting() {
    let src = "\
func myMethod2() {
	r := func() {
		if condition1 {
		}
	}
	_ = r
}
";
    let m = measure(Lang::Go, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 2, "whitepaper p.9 margin total");
}

/// p.19 `toRegexp` (Appendix C): the full else-if chain example,
/// margin sum 20 — exercises ternary at depth 0, mixed operator runs,
/// flat else-if with children at chain level, nested ifs to depth 3.
#[test]
fn p19_to_regexp() {
    let src = r#"
function toRegexp(antPattern: string, directorySeparator: string): string {
  const escapedDirectorySeparator = "\\" + directorySeparator;
  let sb = "^";
  let i = antPattern.startsWith("/") ||
      antPattern.startsWith("\\") ? 1 : 0;
  while (i < antPattern.length) {
    const ch = antPattern.charAt(i);
    if (SPECIAL_CHARS.indexOf(ch) !== -1) {
      sb += "\\" + ch;
    } else if (ch === "*") {
      if (i + 1 < antPattern.length
          && antPattern.charAt(i + 1) === "*") {
        if (i + 2 < antPattern.length
            && isSlash(antPattern.charAt(i + 2))) {
          sb += "(?:.*" + escapedDirectorySeparator + "|)";
          i += 2;
        } else {
          sb += ".*";
          i += 1;
        }
      } else {
        sb += "[^" + escapedDirectorySeparator + "]*?";
      }
    } else if (ch === "?") {
      sb += "[^" + escapedDirectorySeparator + "]";
    } else if (isSlash(ch)) {
      sb += escapedDirectorySeparator;
    } else {
      sb += ch;
    }
    i++;
  }
  sb += "$";
  return sb;
}
"#;
    let m = measure(Lang::TypeScript, src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].coc, 20, "whitepaper p.19 margin total");
}
