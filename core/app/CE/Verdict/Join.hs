-- | The three-signal join verdict lattice (design §6.3), PURE: legs
-- in, (verdict, legsMask, reasonBits) out. Built one batch ahead of
-- its wire (M5-3h) and hooked to verdict/1 by the score batch (3i) —
-- the M5-2a graph stub → 2g replacement pattern. The similarity leg
-- speaks the wire's own vocabulary: (simKind, num, den), judged
-- against the OWNING family's threshold (clone 85/100, docdup
-- 80/100) by cross-multiplication — the 3h token-count floor was the
-- pre-wire approximation and died with the hookup.
--
-- Verdict codes are the design table's own numbering:
-- 0 report_only / 1 merge_candidate / 2 delete_candidate /
-- 3 churn_hotspot. Only Tier F gates: a row whose graph legs are
-- Nothing (unit tier — import granularity has no unit nodes) can
-- only be report_only, and its legsMask says so. Absence is never a
-- zero: a missing graph leg refuses to gate rather than pretending
-- indeg 0.
module CE.Verdict.Join
  ( Pos (..)
  , Legs (..)
  , Knobs (..)
  , bound
  , judge
  , legSim
  , legGraph
  , legChurn
  ) where

import CE.Clone.Cost (tsedDen, tsedNum)
import CE.Docdup.Cost (jaccardDen, jaccardNum)
import CE.Graph.Cost (entryMask)
import qualified CE.Verdict.Cost as Cost
import Data.Bits (bit, testBit, (.&.), (.|.))

-- | One side's graph position (the Position.hs row minus its echoed
-- index, reduced to the fields the lattice reads). On the verdict/1
-- wire pFlags is structurally 0 (file-granularity flags never cross;
-- entry-ness is implied by reachIn) — the field stays because the
-- lattice's RG10 guard and its battery exercise it.
data Pos = Pos
  { pIndeg :: Integer
  , pReach :: Integer
  , pFlags :: Integer
  , pScc :: Integer
  }

-- | One candidate pair's three legs. Similarity and churn are always
-- present (the pair exists BECAUSE similarity found it; churn zeros
-- are real zeros); the graph leg is Maybe because unit-tier rows
-- honestly lack it. lCochange is Maybe: Nothing = the pair sits
-- below the churn report's own table floor — unknown-small, which
-- never fires a verdict, rather than a fabricated zero.
data Legs = Legs
  { lSim :: (Integer, Integer, Integer)
  -- ^ (simKind, num, den): 0 t1t2 / 1 t3 / 2 docdup, the family's
  -- own score ratio.
  , lGraphA :: Maybe Pos
  , lGraphB :: Maybe Pos
  , lChurnA :: (Integer, Integer)
  , lChurnB :: (Integer, Integer)
  , lCochange :: Maybe Integer
  }

-- | The knobs travel as parameters (the 2g mechanism: the battery
-- perturbs fields; production binds constants at the boundary).
data Knobs = Knobs
  { kCloneNum :: Integer
  , kCloneDen :: Integer
  , kDupNum :: Integer
  , kDupDen :: Integer
  , kCochangeFloor :: Integer
  , kRewriteNum :: Integer
  , kRewriteDen :: Integer
  , kEntryMask :: Integer
  }

-- | The Cost binding the wire boundary passes. The similarity
-- thresholds and entryMask are REUSED from their owning families —
-- one authority each; a second copy would let two judgments disagree
-- about the same fact.
bound :: Knobs
bound =
  Knobs
    { kCloneNum = tsedNum
    , kCloneDen = tsedDen
    , kDupNum = jaccardNum
    , kDupDen = jaccardDen
    , kCochangeFloor = Cost.cochangeFloor
    , kRewriteNum = Cost.rewriteNum
    , kRewriteDen = Cost.rewriteDen
    , kEntryMask = entryMask
    }

-- | legsMask bits: which signals were actually present.
legSim, legGraph, legChurn :: Integer
legSim = 1
legGraph = 2
legChurn = 4

-- | (verdict, legsMask, reasonBits). The bit meanings live as the
-- per-bit table in CE.Verdict.Cost; bits record which conditions
-- HELD, the code is the lattice's conclusion, and the two travel
-- together so a two-leg firing can never hide.
judge :: Knobs -> Legs -> (Integer, Integer, Integer)
judge k l = (code, legsMask, reasons)
 where
  (kind, num, den) = lSim l
  -- the OWNING family's bar, cross-multiplied: t1t2/t3 share the
  -- clone threshold, docdup its own
  simOver = case kind of
    2 -> num * kDupDen k >= den * kDupNum k
    _ -> num * kCloneDen k >= den * kCloneNum k
  pair = (,) <$> lGraphA l <*> lGraphB l
  graphBoth = case pair of
    Just _ -> True
    Nothing -> False
  bothRef = maybe False (\(a, b) -> pIndeg a >= 1 && pIndeg b >= 1) pair
  sccDistinct = maybe False (\(a, b) -> pScc a /= pScc b) pair
  -- x is a dead flank of live partner y: unreferenced, unreachable,
  -- no entry bit, while y keeps indeg >= 1 (partner survives higher)
  deadV x y = pIndeg x == 0 && pReach x == 0 && (pFlags x .&. kEntryMask k) == 0 && pIndeg y >= 1
  flanks = maybe [] (\(a, b) -> [(a, b), (b, a)]) pair
  deadFlank = any (uncurry deadV) flanks
  -- RG10 lives in the CONDITION, not a post-filter: a public flank
  -- never becomes delete-ready, and the guard bit says why
  deleteReady = any (\(x, y) -> deadV x y && not (testBit (pFlags x) 0)) flanks
  publicGuard = any (\(x, y) -> deadV x y && testBit (pFlags x) 0) flanks
  (apA, rwA) = lChurnA l
  (apB, rwB) = lChurnB l
  total = apA + rwA + apB + rwB
  rewriteHot = total > 0 && (rwA + rwB) * kRewriteDen k >= total * kRewriteNum k
  cochangeHot = maybe False (>= kCochangeFloor k) (lCochange l)
  gated = simOver && graphBoth
  code
    | gated && bothRef && sccDistinct = 1
    | gated && deleteReady = 2
    | gated && cochangeHot && rewriteHot = 3
    | otherwise = 0
  legsMask = legSim .|. (if graphBoth then legGraph else 0) .|. legChurn
  reasons =
    sum
      [ bit n
      | (n, held) <-
          [ (1 :: Int, simOver)
          , (2, graphBoth)
          , (3, bothRef)
          , (4, sccDistinct)
          , (5, deadFlank)
          , (6, publicGuard)
          , (7, cochangeHot)
          , (8, rewriteHot)
          ]
      , held
      ]
