-- aeson 2.x object keys are Key, not String; string literals for
-- (.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The extent law's facts per file, read from the verdict request
-- (plan v2.35 score amendment), and the reply it answers. Every
-- selection is here: each function's excess share over its ceiling, which file
-- is dead (unreachable: the pos row's last column 0 - the whole set,
-- no in-degree test), which reference line stays inside its file's
-- import cycle (the source a cycle member by the score's own
-- predicate, the target in the same SCC, a self reference only where
-- the floor admits a one-file cycle), which file the size family
-- sizes (a prose-only file has no size term), which file is
-- rewrite-heavy and by how much. The fold over the units is
-- CE.Verdict.Extent.
module CE.Verdict.ExtentFacts (Located (..), located, scored, units) where

import CE.Verdict.Cost (violCostNeutral)
import CE.Verdict.Extent (Unit, extentScore, layered, ownShares, tenths, totalLines, unit)
import CE.Verdict.Score
import CE.Verdict.Wire (VerdictReq (..))
import Data.Aeson (object, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M
import Data.Ratio ((%))

-- | The universe size and the four tables, validated together
-- (CE.Verdict.ExtentRows).
data Located = Located
  { lUniverse :: Integer
  , lSpans :: [[Integer]]
  , lDups :: [[Integer]]
  , lRefs :: [[Integer]]
  , lProse :: [[Integer]]
  }

-- | The request's tables (present on every request the contract
-- admits, CE.Verdict.ExtentRows).
located :: VerdictReq -> Located
located req = Located (toInteger (length (reqTier req))) (get spans) (get dups) (get refs) (get prose)
 where
  (spans, dups, refs, prose) = reqExtent req
  get = maybe [] id

-- | (score, the seven axis rows = each axis's own share, the reply's
-- `extent` pair): the extent law. The score is floored on the exact E;
-- the reply carries E as `tenths` (CE.Verdict.Extent) and L.
scored :: ScoreKnobs -> Maybe Integer -> [[Integer]] -> Facts -> Located -> (Integer, [(Integer, Integer)], [Pair])
scored k soft weights f loc = (extentScore scale kk e l, ownShares scale us, ["extent" .= object ["tenths" .= tenths e, "lines" .= l]])
 where
  us = units k soft f loc
  e = layered [(c, w) | [c, w] <- effectiveWeights k weights] us
  l = totalLines us
  scale = sScoreScale k
  kk = sViolCost k % violCostNeutral

-- | One unit per universe file (0 .. n-1), then one per size row
-- outside the universe (only its size term applies).
units :: ScoreKnobs -> Maybe Integer -> Facts -> Located -> [Unit]
units k soft f loc = [file u | u <- [0 .. fromInteger (lUniverse loc) - 1]] <> outside
 where
  global = maybe (sSizeCeil k) id soft
  softOf row = M.findWithDefault global (classOf row, 0) (fClassKnobs f)
  rows = zip (fCont f) (lSpans loc)
  sized = IM.fromList [(fromInteger u, (v, Just (softOf row))) | (row@(_ : 0 : v : _), (u : _)) <- rows, u >= 0]
  -- read by the docdup family alone: no size term (S absent)
  prose = IM.fromList [(fromInteger u, (l, Nothing)) | [u, l] <- lProse loc]
  -- every universe file owns one of the two (CE.Verdict.ExtentRows)
  sizeOf u = IM.findWithDefault (0, Nothing) u (IM.union sized prose)
  outside = [unit v (Just (softOf row)) [] [] 0 | (row@(_ : 0 : v : _), (u : _)) <- rows, u < 0]
  named = IM.fromListWith (<>) (located' k f loc (fst . sizeOf))
  funcs = IM.fromListWith (<>) [(fromInteger u, [(lo, hi, over row c)]) | (row@(_ : 1 : c : _), [u, lo, hi]) <- rows, u >= 0]
  ceilingOf row = M.findWithDefault (sCocCeil k) (classOf row, 1) (fClassKnobs f)
  -- the share over the ceiling, computed only above it (c = 0 is the
  -- commonest value); 0 at and below, the forest keeps the function
  over row c = if c > ceilingOf row then 1 - ceilingOf row % c else 0
  churn = IM.fromList (churnExcess k f (fst . sizeOf))
  file u =
    let (len, s) = sizeOf u
     in unit len s (IM.findWithDefault [] u named) (IM.findWithDefault [] u funcs) (IM.findWithDefault 0 u churn)

-- | The located lines by file: duplicate member spans (kind 0 / 1 on
-- the clone axis, 2 on the docdup axis), every line of a dead file,
-- the reference lines inside the source's cycle (a self reference is
-- a cycle of length 1: judged only at a floor of 1).
located' :: ScoreKnobs -> Facts -> Located -> (Int -> Integer) -> [(Int, [(Integer, Integer, Integer)])]
located' k f loc linesOf =
  [(fromInteger u, [(if kind <= 1 then 2 else 3, lo, hi)]) | [u, lo, hi, kind] <- lDups loc]
    <> [(fromInteger u, [(4, 1, linesOf (fromInteger u))]) | [u, _, _, _, 0] <- fPos f]
    <> [(fromInteger src, [(6, line, line)]) | [src, dst, line] <- lRefs loc, inCycle src dst]
 where
  members = IS.fromList [fromInteger u | (u : _) <- cycleRows k f]
  scc = IM.fromList [(fromInteger u, s) | [u, _, s, _, _] <- fPos f]
  inCycle src dst =
    IS.member (fromInteger src) members
      && IM.lookup (fromInteger dst) scc == IM.lookup (fromInteger src) scc
      && (src /= dst || sCycleFloor k <= 1)

-- | lines * (1 - t * (rw + ap) / rw) for a rewrite-heavy file, t the
-- rewrite threshold; a heavy row with rw = 0 exists only at t = 0,
-- where the share is the whole file.
churnExcess :: ScoreKnobs -> Facts -> (Int -> Integer) -> [(Int, Rational)]
churnExcess k f linesOf =
  [ (fromInteger u, len * share)
  | [u, rw, ap] <- churnRows k f
  , let len = linesOf (fromInteger u) % 1
  , let share = if rw == 0 then 1 else 1 - (sRewriteNum k % sRewriteDen k) * ((rw + ap) % rw)
  ]
