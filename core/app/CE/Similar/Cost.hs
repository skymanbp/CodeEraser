-- | The similar family's constants (plan v2.29 step 5; ADR-008 sixth
-- instalment): the same-role conjunction and its floors, repatriated
-- from cli/src/similar/bm25.rs where the ROI instrument carried them
-- as its declared mirror. Rust measures nine integers per candidate —
-- the distinct spelled terms shared per channel [N,P,C,D,S,L], the
-- shape-equality bit, and the BM25 score as a fraction; WHICH
-- candidates play the query's role, and in what order they stand, is
-- judgment and lives here. An advisor: no floor here is a condition
-- bit, nothing here reaches `ce check` (booklet 13's posture).
module CE.Similar.Cost (
  similarCap,
  rowWidth,
  roleMinName,
  roleMinCallee,
  roleMinNameShape,
  isRole,
  ratio,
) where

import Data.Ratio ((%))

-- | Table ceiling: query terms and candidate rows together. A unit's
-- bag runs to a few hundred terms and the measuring side sends its
-- top-k (k = 5 in the instrument); 65536 is far above any honest
-- request. Over-cap answers a complete degraded reply.
similarCap :: Integer
similarCap = 65536

-- | The candidate row: [nHit, pHit, cHit, dHit, sHit, lHit, shapeEqual,
-- bm25Num, bm25Den].
rowWidth :: Int
rowWidth = 9

-- | First arm: at least this many shared name terms AND at least this
-- many shared callee terms — a unit that is called the same and calls
-- the same (the `stacking` three-signal precedent, not audit/1's
-- single-bit disjunction). The floors are role-prefixed because the
-- docs_consts gate binds booklet 14's `minName` chip to
-- CE.Tombstone.Cost by bare name and requires that binding unique.
roleMinName, roleMinCallee :: Integer
roleMinName = 1
roleMinCallee = 1

-- | Second arm: at least this many shared name terms with the shape
-- (arity, return, kind) equal — two names in common and the same
-- signature shape, without a shared callee.
roleMinNameShape :: Integer
roleMinNameShape = 2

-- | One row's verdict. The floors are what the two oracle generations
-- measured the precision of (docs/EVAL-SET-SIMILAR.md); the holdout
-- retest kept this exact form.
isRole :: [Integer] -> Bool
isRole [n, _, c, _, _, _, shape, _, _] =
  (n >= roleMinName && c >= roleMinCallee) || (n >= roleMinNameShape && shape == 1)
isRole _ = False -- unreachable behind famOffence; refuse, never convict

-- | One row's score as an exact rational — the measuring side's
-- fixed-point numerator over its denominator; this side never learns
-- the fixed-point width and never rounds.
ratio :: [Integer] -> Rational
ratio row = case row of
  [_, _, _, _, _, _, _, num, den] -> num % den
  _ -> 0 -- unreachable behind famOffence
