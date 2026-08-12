-- | Phase 1 — site opening. Anchored blocks are maximal common
-- contiguous hash sub-sequences between a removed run of one pair and
-- an added run of a DIFFERENT pair, accepted iff their length clears
-- the derived floor (CE.FourClass.Cost.destFloor). Runs arrive from
-- the aligner (Wire): judgment consumes the run structure as given.
--
-- No exclusivity, no greedy claiming, no tie-break — required, not
-- lazy: de-duplication commits are many-to-one (one destination body
-- explains many removed copies), so accepted blocks are a union of
-- independently derived sets and the result has no dependence on
-- iteration order. Determinism is structural.
module CE.FourClass.Anchor
  ( bucketCap
  , sites
  ) where

import CE.FourClass.Cost (anchorFloor, destFloor)
import CE.FourClass.Wire (Block (..), Pair (..))
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import Data.Word (Word64)

-- | One wire entry: (1-based line, content hash, alnum width).
type Entry = (Int, Word64, Int)

hOf :: Entry -> Word64
hOf (_, h, _) = h

-- | Per-hash pairing budget: a hash whose removed x added occurrence
-- product exceeds bucketCap^2 degrades the request (all-or-nothing,
-- attack review F3) — the product IS the work the site enumeration
-- spends, so the bound is the same worst case as the old per-side
-- form while never firing on real shapes the per-side count
-- mislabeled: ripgrep b9de003f8 adds 66 identical #[inline] lines
-- with no removed partner (product 0), and 082245dad pairs 5 removed
-- #[derive(Debug)] with 118 added (product 590; such boilerplate is
-- floor-dead anyway). Measured self-slice headroom: largest bucket 9.
bucketCap :: Int
bucketCap = 64

-- | One side occurrence of a hash: (pair, run id, position in run).
data Occ = Occ {oPair :: !Int, oRun :: !Int, oPos :: !Int}

type RunMap = M.Map (Int, Int) [Entry]

buildRuns :: (Pair -> [[Entry]]) -> [Pair] -> RunMap
buildRuns side ps =
  M.fromList [((pIdx p, r), run) | p <- ps, (r, run) <- zip [0 ..] (side p)]

buildIx :: RunMap -> M.Map Word64 [Occ]
buildIx rm =
  M.fromListWith
    (flip (++))
    [(h, [Occ p r i]) | ((p, r), run) <- M.toList rm, (i, (_, h, _)) <- zip [0 ..] run]

-- | All accepted blocks plus whether the bucket cap tripped.
sites :: [Pair] -> ([Block], Bool)
sites ps = (blocks, capped)
 where
  remRuns = buildRuns pRem ps
  addRuns = buildRuns pAdd ps
  remIx = buildIx remRuns
  addIx = buildIx addRuns
  overWork remOccs addOccs = length remOccs * length addOccs > bucketCap * bucketCap
  -- One direction suffices: the product is symmetric and a hash
  -- absent from either index has product zero.
  capped =
    or
      [ overWork remOccs (M.findWithDefault [] h addIx)
      | (h, remOccs) <- M.toList remIx
      ]
  blocks =
    [ block
    | (h, remOccs) <- M.toList remIx
    , let addOccs = M.findWithDefault [] h addIx
    , not (overWork remOccs addOccs)
    , ro <- remOccs
    , ao <- addOccs
    , oPair ro /= oPair ao
    , Just block <- [tryBlock remRuns addRuns ro ao]
    ]

-- | Extend forward from a block START (each block is discovered
-- exactly once: interior positions have equal predecessors on both
-- sides and fail the start test). The floor counts DISTINCT content
-- values, not lines: one common line repeated twice is a single
-- piece of evidence and must not open a site (attack review F5 —
-- length alone let `[x,x]` matched against `[x,x]` clear the floor).
-- The evidence must also include one ANCHOR line (>= anchorFloor
-- alnum chars): two short distinct lines cleared the old floor on a
-- pure reformat commit (M5-1c-ii, requests 2a6f290b — `Timeout,` +
-- `TooManyRedirects,` invented a station).
tryBlock :: RunMap -> RunMap -> Occ -> Occ -> Maybe Block
tryBlock rm am ro ao
  | not isStart = Nothing
  | distinctEvidence >= destFloor && anchored =
      Just (Block (oPair ro) [l | (l, _, _) <- evidence] (oPair ao) (map fst3 (take n aTail)))
  | otherwise = Nothing
 where
  rRun = rm M.! (oPair ro, oRun ro)
  aRun = am M.! (oPair ao, oRun ao)
  rTail = drop (oPos ro) rRun
  aTail = drop (oPos ao) aRun
  isStart =
    oPos ro == 0
      || oPos ao == 0
      || hOf (rRun !! (oPos ro - 1)) /= hOf (aRun !! (oPos ao - 1))
  n = length (takeWhile (uncurry (==)) (zip (map hOf rTail) (map hOf aTail)))
  evidence = take n rTail
  distinctEvidence = S.size (S.fromList (map hOf evidence))
  anchored = any (\(_, _, w) -> w >= anchorFloor) evidence
  fst3 (l, _, _) = l
