-- | The M4 judgment rules over the four-class split (plan §4.3,
-- intent-free by design). One rule ships in M4: STACKING — the "old
-- implementation kept, a fresh copy written next to it" shape. It
-- fires only on the conjunction of three signals, each cheap to
-- refute on a normal edit:
--   novel mass    — at least stackingNovelFloor novel lines, and
--   little removal — deletions under novel/stackingRatio, and
--   a duplicated unit — a unit key newly duplicated on the after
--                       side (the `dup` hashes the aligner ships).
-- The FPR gate (contracts/eval) measures exactly this conjunction
-- over the real-edit corpus; the constants live here so the gate has
-- one target.
module CE.FourClass.Verdict
  ( stackingNovelFloor
  , stackingRatio
  , suspicions
  ) where

import CE.FourClass.Wire (Pair (..))

-- | Minimum novel lines before stacking is worth raising. Below
-- this, even a true duplicate is a nit, not a stack.
stackingNovelFloor :: Int
stackingNovelFloor = 20

-- | Deletions must stay under novel/ratio — editing-in-place removes
-- roughly what it adds; stacking removes almost nothing.
stackingRatio :: Int
stackingRatio = 10

-- | (pair index, rule name) per firing rule. `novel`/`deleted` are
-- the pair's post-reclassification counts, supplied by the caller
-- (leftover lists here are pre-delta and would overcount).
suspicions :: [(Pair, Int, Int)] -> [(Int, String)]
suspicions rows =
  [ (pIdx p, "stacking")
  | (p, novel, deleted) <- rows
  , not (null (pDup p))
  , novel >= stackingNovelFloor
  , deleted * stackingRatio < novel
  ]
