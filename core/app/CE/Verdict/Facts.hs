-- | The score's input tables: the types live here, CE.Verdict.Score
-- re-exports them, every import site reads them through it.
module CE.Verdict.Facts
  ( Facts (..)
  , ClassKnobs
  , classKnobsOf
  , classOf
  ) where

import qualified Data.Map.Strict as M

-- | The validated fact tables (row shapes enforced by CE.Verdict's
-- boundary contract before anything reaches here). Continuous rows
-- are read by PREFIX throughout (3.1.0): a legacy three-column row
-- and a classed four-column row take the same patterns.
data Facts = Facts
  { fPos :: [[Integer]]
  -- ^ [u,indeg,sccId,sccSize,reachIn] (10.0.0: graph/1's outdegree is not sent)
  , fChurn :: [[Integer]]
  -- ^ [u,rewrite,append] (3.0.0: the constant added / unmeasured survived left)
  , fCont :: [[Integer]]
  -- ^ [u,metricCode,value] or, classed (3.1.0), [u,metricCode,value,classId]
  , fDocFiles :: [Integer]
  -- ^ file-universe indices whose language is documentation
  , fClassKnobs :: ClassKnobs
  -- ^ (classId, code) -> value: the ceilings codes 0/1/2 under a class (3.1.0)
  , fSelfLoops :: [Integer]
  -- ^ file-universe indices carrying a self-arc (6.4.0): a singleton SCC is a cycle only through one
  }

-- | The class overrides as ONE Map, built once per judgment from the
-- wire rows (plan v2.13 ①): the wire's row order is a validation
-- fact and never a judgment fact (ClassProps pins the permutation).
-- Lookup falls back to the global reading, so an unclassed row — or
-- a class with no row for that code — judges exactly as before.
type ClassKnobs = M.Map (Integer, Integer) Integer

classKnobsOf :: [[Integer]] -> ClassKnobs
classKnobsOf rows = M.fromList [((c, code), v) | [c, code, v] <- rows]

-- | A continuous row's class: the 4th column when it rides, 0 (the
-- global table) on a legacy three-column row.
classOf :: [Integer] -> Integer
classOf row = case drop 3 row of
  (c : _) -> c
  [] -> 0
