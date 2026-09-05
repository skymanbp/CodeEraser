-- | The M4 judgment rules over the four-class split (plan §4.3,
-- intent-free by design). One rule ships in M4: STACKING — the "old
-- implementation kept, a fresh copy written next to it" shape. It
-- fires only on the conjunction of three signals, each cheap to
-- refute on a normal edit:
--   novel mass    — at least stackingNovelFloor novel lines, and
--   little removal — deletions under novel/stackingRatio, and
--   a duplicated unit — at least stackingNovelFloor of those novel
--                       lines lie INSIDE the span of a unit key
--                       newly duplicated on the after side (the
--                       `dupSpans` rows the aligner ships, 7.0.0).
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

-- | (pair index, rule name) per firing rule. `novel` is the pair's
-- post-reclassification novel LINES and `deleted` its deleted count,
-- supplied by the caller (leftover lists here are pre-delta and would
-- overcount). Since 7.0.0 (O47) the mass weighed against the floor is
-- the novel lines inside a newly duplicated unit's span: a fresh copy
-- written beside the old one puts its lines there, while twenty novel
-- lines elsewhere in a file that happens to gain a duplicate key are
-- ordinary editing — the 6.x rule joined them and fired. The removal
-- ratio still reads the whole edit: stacking removes almost nothing.
suspicions :: [(Pair, [Int], Int)] -> [(Int, String)]
suspicions rows =
  [ (pIdx p, "stacking")
  | (p, novel, deleted) <- rows
  , let inDup = length [l | l <- novel, any (\(_, s, e) -> s <= l && l <= e) (pDupSpans p)]
  , inDup >= stackingNovelFloor
  , deleted * stackingRatio < length novel
  ]
