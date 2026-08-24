-- | The scan-family grading knobs (ADR-008 P3) — the graded verdict
-- table itself, the DSL's second table form (design booklet §4):
-- one row per metric code, (code, warnDefault, failDefault), fail 0
-- = no hard line exists for that metric. The numbers are the plan
-- §4.1 dogfood thresholds — the SAME defaults
-- cli/src/config.rs::Thresholds declares (provenance: ESLint
-- max-lines=300, Sonar S104=750/S138=75, ESLint fn=50, Pylint
-- max-args=5, Sonar S3776 CoC=15, lizard CC=15). ce.toml is the
-- source, these are the DEFAULTS (the 27b9bc2 pattern), and the
-- reply's grade echo pins the Rust mirror.
module CE.Scan.Cost (conforms, goLang, gradeTable, gradeWith, scanRowCap) where

-- | Codes are frozen positions (the wire.rs edge-code discipline):
--   0 file-lines   1 fn-lines   2 fn-params   3 cyclomatic
--   4 cognitive    5 nesting    6 fn-naming (value 0/1, warn 0;
--   derived from the naming facts when they ride — conforms below)
gradeTable :: [(Integer, Integer, Integer)]
gradeTable =
  [ (0, 300, 750)
  , (1, 50, 75)
  , (2, 5, 0)
  , (3, 15, 0)
  , (4, 15, 0)
  , (5, 4, 0)
  , (6, 0, 0)
  ]

-- | One measured value against one (warn, fail) row: 2 = fail (a
-- hard line exists and is exceeded), 1 = warn, 0 = clean —
-- strictly-greater on both lines (the report.rs comparisons,
-- restated by their OWNER; the Rust mirror is proven equal by the
-- product's whole-report ensure, never trusted).
gradeWith :: (Integer, Integer) -> Integer -> Integer
gradeWith (warn, failLine) v
  | failLine > 0 && v > failLine = 2
  | v > warn = 1
  | otherwise = 0

-- | Row ceiling (the verdictRowCap magnitude anchor: rows are
-- (metric, value) pairs, several per function — 524288 covers a
-- ~75k-function tree). Over-cap answers a complete degraded reply
-- that FAILS (the P1 posture: a gate that could not judge must
-- never pass).
scanRowCap :: Integer
scanRowCap = 524288

-- | The fn-naming judgment (2.30.0, ADR-008 batch-7 slice 14): the
-- convention verdict over the five wire facts [lang, style, upper,
-- under, test] — style 1 (snake) forbids uppercase, style 2
-- (mixedCaps) forbids underscores except the go-vet test-name
-- shapes, and that exemption is gated on Go's OWN lang code: the
-- same facts under TypeScript or Haskell stay a violation (the
-- leak the Rust-side predicate carried). Style 0 = no convention
-- (Markdown, non-identifier subjects). Shape is enforced by
-- Scan.hs's validator; cli scan/metrics/naming.rs::conforms is the
-- pinned mirror the whole-report ensure proves equal.
conforms :: [Integer] -> Bool
conforms row = case row of
  [lang, style, upper, under, test]
    | style == 1 -> upper == 0
    | style == 2 -> under == 0 || (lang == goLang && test == 1)
    | otherwise -> True
  _ -> error "naming shape enforced by violation"

-- | Go's frozen Lang wire code (cli scan/lang.rs LANGS order).
goLang :: Integer
goLang = 4
