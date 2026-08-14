//! Sonar Cognitive Complexity whitepaper v1.7 (2023-08-29) worked
//! examples, ported to the launch languages (plan §6 M1: "CoC 过
//! Sonar 白皮书共通例题"). Every expected value below is read from the
//! whitepaper's margin annotations at the cited page, not re-derived.
//! The rows ride the shared table runner (common::run_metric_cases);
//! each row's why strings carry the page citations.

use codeeraser::scan::lang::Lang;

mod common;

const CASES: &[common::MetricCase] = &[
    // p.5 + p.10 sumOfPrimes: Go port keeps the labeled jump.
    common::MetricCase {
        lang: Lang::Go,
        src: "\
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
",
        fns: 1,
        checks: &[
            (
                0,
                "coc",
                7,
                "p.10 margin: for +1, for +2, if +3, continue OUT +1",
            ),
            (0, "cc", 4, "p.5 margin total"),
        ],
    },
    // p.5 + p.10 getWords: a switch and all its cases = one structural
    // increment; CC 4 (default adds no branch — gocyclo v0.6.0 rule).
    common::MetricCase {
        lang: Lang::Go,
        src: "\
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
",
        fns: 1,
        checks: &[
            (0, "coc", 1, "p.10 margin: one structural increment"),
            (0, "cc", 4, "p.5 margin: default not counted"),
        ],
    },
    // p.8 second operator example: the negated parenthesized sub-chain
    // starts a NEW sequence — pins ce's paren-bounded run model.
    common::MetricCase {
        lang: Lang::TypeScript,
        src: "\
function f(a: boolean, b: boolean, c: boolean): boolean {
  if (a && !(b && c)) {
    return true;
  }
  return false;
}
",
        fns: 1,
        checks: &[(0, "coc", 3, "p.8 margin: if +1, && +1, && +1")],
    },
    // p.9 myMethod: try is transparent (no increment, no nesting).
    common::MetricCase {
        lang: Lang::Python,
        src: "\
def my_method():
    try:
        if condition1:
            for i in range(10):
                while condition2:
                    pass
    except (ValueError, TypeError):
        if condition2:
            pass
",
        fns: 1,
        checks: &[(
            0,
            "coc",
            9,
            "p.9 margin: if +1, for +2, while +3, catch +1, if +2",
        )],
    },
    // p.9 myMethod2: a lambda increments nothing but raises nesting —
    // Go func-literal port (absorbed unit).
    common::MetricCase {
        lang: Lang::Go,
        src: "\
func myMethod2() {
	r := func() {
		if condition1 {
		}
	}
	_ = r
}
",
        fns: 1,
        checks: &[(0, "coc", 2, "p.9 margin: lambda nests, if pays +2")],
    },
    // p.19 toRegexp (Appendix C): ternary at depth 0, mixed operator
    // runs, flat else-if with children at chain level, depth-3 ifs.
    common::MetricCase {
        lang: Lang::TypeScript,
        src: r#"
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
"#,
        fns: 1,
        checks: &[(0, "coc", 20, "p.19 margin total")],
    },
];

#[test]
fn whitepaper_worked_examples() {
    common::run_metric_cases(CASES);
}
