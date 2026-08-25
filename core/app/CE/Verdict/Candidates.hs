-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The join-candidate face of a verdict reply: one row per sim pair,
-- judged through the SAME effective Join knobs the score read, so the
-- two judgments share one authority. Split from CE.Verdict at the
-- E01 300-line core gate when the rulepack fence (5.1.0) arrived —
-- the leg maps were already "the candidates' concern alone", which is
-- what makes this a seam rather than a slice.
module CE.Verdict.Candidates (candidates) where

import CE.Graph.Cost (exportVisBit, publicFlagBit)
import qualified CE.Graph.Dead as Dead
import CE.Verdict.Join (Knobs, Legs (..), Pos (..), confidence, judge)
import CE.Verdict.Wire (VerdictReq (..))
import Data.Bits (bit)
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M

-- | Join-candidate rows, one per sim row (split from result at the
-- E01 line — the leg maps are the candidates' concern alone). The
-- effective Join knobs arrive from the same thresholds table the
-- score reads, so the two judgments share one authority.
candidates :: Knobs -> VerdictReq -> [[Integer]]
candidates jk req =
  [ -- the 6th column is the leg-agreement confidence (2.33.0, H4)
    [u, v, code, bits, mask, confidence mask bits]
  | row@(u : v : _) <- reqSim req
  , let (code, mask, bits) = judge jk (legsOf row)
  ]
 where
  -- pFlags carries the EXPORT axis and nothing else. Entry-ness
  -- rides reachIn (an entry seeds the reach set, so it is never a
  -- dead flank), which is why the pos row has no flags column and
  -- needs none. Bit 0 is the other half, and until 6.1.0 nothing
  -- here could set it: the graph face had the fact from 4.1.0 while
  -- the lattice's RG10 guard sat inert, so `delete` could be
  -- proposed for an exported flank — the one thing the four-way
  -- verdict code exists to prevent. WHICH bit means exported is the
  -- graph family's judgment and is reused, never re-decided.
  exported = Dead.exportedNodes exportVisBit (reqSymbols req)
  flagsOf u
    | IS.member (fromInteger u) exported = bit (fromInteger publicFlagBit)
    | otherwise = 0
  posMap =
    M.fromList
      [ (u, Pos indeg reachIn (flagsOf u) sccId)
      | [u, indeg, _outdeg, sccId, _sccSize, reachIn] <- reqPos req
      ]
  churnMap = M.fromList [(u, (ap, rw)) | [u, rw, ap] <- reqChurn req]
  cochMap = M.fromList [((u, v), c) | [u, v, c] <- reqCochange req]
  legsOf row = case row of
    [u, v, kind, num, den] ->
      Legs
        { lSim = (kind, num, den)
        , lGraphA = M.lookup u posMap
        , lGraphB = M.lookup v posMap
        , lChurnA = M.findWithDefault (0, 0) u churnMap
        , lChurnB = M.findWithDefault (0, 0) v churnMap
        , lCochange = M.lookup (u, v) cochMap
        }
    _ -> error "sim row shape enforced by violation"
