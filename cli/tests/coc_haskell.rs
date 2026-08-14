//! M5-3k Haskell CoC battery: the whitepaper's PORTABLE worked
//! examples plus the stance pins for everything Haskell that S3776
//! never names — each why cites the page or the register entry
//! (contracts/coc-haskell-divergences.md). The unportable examples
//! (sumOfPrimes, toRegexp: loops and labeled jumps have no Haskell
//! form) are register entries D4, not silent omissions.

use codeeraser::scan::lang::Lang;

mod common;

const CASES: &[common::MetricCase] = &[
    // p.5 + p.10 getWords as a case expression.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
getWords :: Int -> String
getWords number = case number of
  1 -> \"one\"
  2 -> \"a couple\"
  3 -> \"a few\"
  _ -> \"lots\"
",
        fns: 1,
        checks: &[
            (0, "coc", 1, "p.10 margin: one structural increment"),
            (
                0,
                "cc",
                5,
                "every alternative incl. the total-case wildcard (register D2; \
                 gocyclo's default-skip not adopted)",
            ),
        ],
    },
    // p.8 second operator example: the negated parenthesized
    // sub-chain starts a NEW run (paren-bounded run model).
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
f :: Bool -> Bool -> Bool -> Bool
f a b c = if a && not (b && c) then True else False
",
        fns: 1,
        checks: &[(0, "coc", 3, "p.8 margin: if +1, && +1, && +1")],
    },
    // p.9 myMethod2: an anonymous lambda increments nothing but
    // raises nesting (register D6; Go func_literal port peer).
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
myMethod2 :: [Int] -> [Int]
myMethod2 xs = map (\\v -> if v > 0 then v else 0) xs
",
        fns: 1,
        checks: &[(0, "coc", 2, "p.9 margin: lambda nests, if pays +2")],
    },
    // Register D1: guards are elif-analogue hybrids — flat +1 per
    // guarded alternative (p.7), one CC decision per condition.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
classify :: Int -> String
classify n
  | n < 0 = \"neg\"
  | n == 0 = \"zero\"
  | n < 10 && n > 5 = \"mid\"
  | otherwise = \"big\"
",
        fns: 1,
        checks: &[
            (0, "coc", 5, "four guards flat +4 (D1), one && run +1"),
            (0, "cc", 6, "base 1 + four conditions + one &&"),
        ],
    },
    // Register D3: if-then-else is an EXPRESSION — the ternary
    // analogue; a case alternative's if pays the nesting penalty.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
depth :: Int -> Int -> Int
depth a b = case a of
  0 -> if b > 0 then 1 else 2
  _ -> 3
",
        fns: 1,
        checks: &[
            (
                0,
                "coc",
                3,
                "case +1, nested if +2 (D3: else costs nothing)",
            ),
            (0, "cc", 4, "base 1 + two alternatives + one conditional"),
        ],
    },
    // Register D7: one unit PER EQUATION, nth keys them apart.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
s :: Maybe Int -> Int -> Int
s (Just a) 0 = a
s Nothing _ = 9
",
        fns: 2,
        checks: &[
            (0, "params", 2, "patterns count = arity (D7)"),
            (1, "params", 2, "second equation, same key, nth 1"),
        ],
    },
    // Register D8: do-statement binds are NOT units (name-gated
    // kind) — main stays one unit and keeps its body's complexity.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
main :: IO ()
main = do
  x <- getLine
  if null x then putStrLn \"e\" else putStrLn x
",
        fns: 1,
        checks: &[(0, "coc", 1, "the do-bind is no unit boundary (D8)")],
    },
    // Register D8 flip side: a NAMED where-bind is its own unit
    // (Rust closure precedent) and its host does not double-pay.
    common::MetricCase {
        lang: Lang::Haskell,
        src: "\
p :: Int -> Int
p x = y
  where
    y = if x > 0 then x else 0
",
        fns: 2,
        checks: &[
            (0, "coc", 0, "host: the nested unit is measured apart"),
            (1, "coc", 1, "the where-bind pays its own if (D8)"),
        ],
    },
];

#[test]
fn haskell_whitepaper_and_stance_battery() {
    common::run_metric_cases(CASES);
}

/// Cross-language equivalence (3k acceptance): one shape — a guarded
/// early return over a mixed boolean chain — measures the SAME CoC in
/// all five grammar-backed languages. Haskell's mandatory `else`
/// costs nothing under the ternary stance (D3), exactly as the `:`
/// of a ternary costs nothing elsewhere.
#[test]
fn cross_language_equivalence_pins_one_shape_at_three() {
    let shapes: &[(Lang, &str)] = &[
        (
            Lang::TypeScript,
            "function g(a: boolean, b: boolean, c: boolean): number {\n  if (a && b || c) {\n    return 1;\n  }\n  return 0;\n}\n",
        ),
        (
            Lang::Python,
            "def g(a, b, c):\n    if a and b or c:\n        return 1\n    return 0\n",
        ),
        (
            Lang::Rust,
            "fn g(a: bool, b: bool, c: bool) -> i32 {\n    if a && b || c {\n        return 1;\n    }\n    0\n}\n",
        ),
        (
            Lang::Go,
            "func g(a, b, c bool) int {\n\tif a && b || c {\n\t\treturn 1\n\t}\n\treturn 0\n}\n",
        ),
        (
            Lang::Haskell,
            "g :: Bool -> Bool -> Bool -> Int\ng a b c = if a && b || c then 1 else 0\n",
        ),
    ];
    for (lang, src) in shapes {
        let m = common::measure_units(*lang, src);
        assert_eq!(m.len(), 1, "{lang:?}: one unit");
        assert_eq!(
            m[0].coc, 3,
            "{lang:?}: if +1 plus two operator runs — the languages must agree"
        );
    }
}
