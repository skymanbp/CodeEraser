-- | The seven structure axes (design booklet §3): S0..S4 always
-- judged; staleness (5, S3c) and redundancy (6, S3b) judged when
-- their report rows ride the wire — both landed, both live here.
-- Each axis is ONE named predicate over the validated fact tables,
-- owning its knobs, so the perturbation battery has a lever per
-- row — the Score.hs mechanism at the tree scale. Everything here
-- is pure; the knobs travel as parameters bound at 'bound'.
module CE.Structure.Axes
  ( Facts (..)
  , Knobs (..)
  , bound
  , axes
  , findings
  , entropyRows
  ) where

import CE.Structure.Entropy (perMille, tsallis2Norm)
import qualified CE.Structure.Cost as Cost
import qualified Data.Map.Strict as M

-- | The validated fact tables (row shapes enforced by CE.Structure's
-- boundary contract before anything reaches here). The S2 mixing
-- axis judges on file TOUCH counts (fileRefs), one basis for both
-- sides of its comparison; the directed dir-edge table joins the
-- wire in S3+ with modularity proper — an unjudged table is dead
-- freight, not a reservation.
data Facts = Facts
  { fNodes :: [[Integer]]
  -- ^ [id, parent, depth, subdirs, files]
  , fPatterns :: [[Integer]]
  -- ^ [dirId, patternCode, count]
  , fConventions :: [[Integer]]
  -- ^ [dirId, bits]
  , fFileRefs :: [[Integer]]
  -- ^ [dirId, inside, outside, count]
  , fStaleDocs :: Maybe [[Integer]]
  -- ^ [dirId, stale, total] — same Maybe stance as fRedundancy:
  -- axis 5 judges exactly when the table rode the wire.
  , fRedundancy :: Maybe [[Integer]]
  -- ^ [dirId, dupBlocks, deadUnits] — Nothing = the table never
  -- rode the wire and axis 6 is NOT judged; Just [] = it rode
  -- empty and the axis judged clean (absence vs zero, spoken).
  }

data Knobs = Knobs
  { kDepthCeil :: Integer
  , kFanoutCeil :: Integer
  , kNamingMin :: Integer
  , kNamingCeil :: Integer
  , kMixRefFloor :: Integer
  , kMisplaceMin :: Integer
  , kBigDirFloor :: Integer
  , kViolCost :: Integer
  , kScale :: Integer
  , kDupMin :: Integer
  , kDeadMin :: Integer
  , kStaleMin :: Integer
  }

bound :: Knobs
bound =
  Knobs
    { kDepthCeil = Cost.depthCeil
    , kFanoutCeil = Cost.fanoutCeil
    , kNamingMin = Cost.namingMin
    , kNamingCeil = Cost.namingCeil
    , kMixRefFloor = Cost.mixRefFloor
    , kMisplaceMin = Cost.misplaceMin
    , kBigDirFloor = Cost.bigDirFloor
    , kViolCost = Cost.structViolCost
    , kScale = Cost.structScale
    , kDupMin = Cost.dupMin
    , kDeadMin = Cost.deadMin
    , kStaleMin = Cost.staleMin
    }

-- | Violation count per judged axis — 0 geometry / 1 naming /
-- 2 mixing / 3 misplacement / 4 documentation, plus 5 staleness and
-- 6 redundancy when their fact tables rode the wire (design §3: an
-- absent table is an unjudged axis, and the score divides by the
-- JUDGED count).
axes :: Knobs -> Facts -> [(Integer, Integer)]
axes k f =
  [ (0, count (geometry k f))
  , (1, count (naming k f))
  , (2, count (mixing k f))
  , (3, misplaced k f)
  , (4, count (docs k f))
  ]
    <> [(5, count (stale k rows)) | Just rows <- [fStaleDocs f]]
    <> [(6, count (redundant k rows)) | Just rows <- [fRedundancy f]]
 where
  count = toInteger . length

-- | The sparse per-directory drill-down rows [dirId, axis] the GUI
-- tree colours by (misplacement is per-file and reports its dir).
findings :: Knobs -> Facts -> [[Integer]]
findings k f =
  [ [d, axis]
  | (axis, ds) <-
      [(0, geometry k f), (1, naming k f), (2, mixing k f)]
        <> [(3, misplacedDirs k f), (4, docs k f)]
        <> [(5, stale k rows) | Just rows <- [fStaleDocs f]]
        <> [(6, redundant k rows) | Just rows <- [fRedundancy f]]
  , d <- ds
  ]

-- | S5: directories whose stale-document count reaches its floor —
-- the docs whose referenced code moved on after their last edit
-- (the measurement side joins the md ladder with the churn window;
-- here only the counts are judged).
stale :: Knobs -> [[Integer]] -> [Integer]
stale k rows = [d | [d, s, _total] <- rows, s >= kStaleMin k]

-- | S6: directories whose clone-block or dead-unit rollup reaches
-- its floor — duplication and orphans are the per-file families'
-- verdicts convolved to the tree scale, never re-derived here.
redundant :: Knobs -> [[Integer]] -> [Integer]
redundant k rows =
  [ d
  | [d, dupBlocks, deadUnits] <- rows
  , dupBlocks >= kDupMin k || deadUnits >= kDeadMin k
  ]

geometry :: Knobs -> Facts -> [Integer]
geometry k f =
  [ i
  | [i, _, depth, subdirs, files] <- fNodes f
  , depth > kDepthCeil k || subdirs + files > kFanoutCeil k
  ]

-- | Sibling sets big enough to judge whose normalized naming
-- diversity exceeds the ceiling — a style mix, not a convention.
naming :: Knobs -> Facts -> [Integer]
naming k f =
  [ d
  | (d, cs) <- M.toList (patternsByDir f)
  , sum cs >= kNamingMin k
  , perMille (tsallis2Norm cs) > kNamingCeil k
  ]

patternsByDir :: Facts -> M.Map Integer [Integer]
patternsByDir f = M.fromListWith (++) [(d, [c]) | [d, _, c] <- fPatterns f]

-- | Directories whose cross-directory reference touches outweigh
-- their internal cohesion, once they carry enough traffic to judge
-- — ONE basis (per-file touch counts) on both sides of the
-- comparison, never edges-vs-touches.
mixing :: Knobs -> Facts -> [Integer]
mixing k f =
  [ d
  | (d, (ins, outs)) <- M.toList (touchesByDir f)
  , ins + outs >= kMixRefFloor k
  , outs > ins
  ]

touchesByDir :: Facts -> M.Map Integer (Integer, Integer)
touchesByDir f =
  M.fromListWith add [(d, (i * n, o * n)) | [d, i, o, n] <- fFileRefs f]
 where
  add (a, b) (c, d) = (a + c, b + d)

-- | Files whose reference neighbourhood lives elsewhere: outside
-- past the floor AND more than twice inside (the ratio is the
-- predicate's definition in v1). Counts ride the aggregated rows.
misplaced :: Knobs -> Facts -> Integer
misplaced k f = sum [n | (_, n) <- misplacedRows k f]

-- | The dirs those files report — the axis-3 findings leg the doc
-- promised but dropped (deduped: several fFileRefs rows per dir).
misplacedDirs :: Knobs -> Facts -> [Integer]
misplacedDirs k f = M.keys (M.fromList [(d, ()) | (d, _) <- misplacedRows k f])

-- | ONE predicate for both faces of axis 3 (count and location).
misplacedRows :: Knobs -> Facts -> [(Integer, Integer)]
misplacedRows k f =
  [ (d, n)
  | [d, inside, outside, n] <- fFileRefs f
  , outside >= kMisplaceMin k
  , outside > 2 * inside
  ]

-- | Big directories missing their README, plus the root missing a
-- recognized config (bits: 1 = README, 2 = config).
docs :: Knobs -> Facts -> [Integer]
docs k f =
  [ i
  | [i, _, _, _, files] <- fNodes f
  , let bits = M.findWithDefault 0 i convMap
  , (files >= kBigDirFloor k && even bits) || (i == 0 && bits < 2)
  ]
 where
  convMap = M.fromList [(d, b) | [d, b] <- fConventions f]

-- | The headline entropy rows: 0 = the GLOBAL naming distribution
-- (pattern counts summed over every directory), 1 = the file-count
-- distribution across directories (how evenly the tree spreads),
-- both normalized Tsallis-2 in per-mille.
entropyRows :: Facts -> [[Integer]]
entropyRows f =
  [ [0, perMille (tsallis2Norm globalPatterns)]
  , [1, perMille (tsallis2Norm dirFiles)]
  ]
 where
  globalPatterns =
    M.elems (M.fromListWith (+) [(code, c) | [_, code, c] <- fPatterns f])
  dirFiles = [files | [_, _, _, _, files] <- fNodes f, files > 0]
