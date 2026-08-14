-- | The ADR-006 ratchet, pure: continuous ceilings with the
-- max(+2%, +10) single-edit tolerance, and the discrete violation
-- SET (new member = fail, removed member auto-shrinks). No baseline
-- means ESTABLISH: nothing fails, the current facts become the new
-- baseline (the betterer opening move). Knobs travel as parameters;
-- ratchetBound binds the Cost constants (the 2g mechanism).
module CE.Verdict.Ratchet
  ( Baseline (..)
  , RatchetKnobs (..)
  , Ratcheted (..)
  , ratchetBound
  , tolerated
  , ratchet
  ) where

import qualified CE.Verdict.Cost as Cost
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | A parsed ce-baseline.json: per-entity continuous ceilings
-- [u,metricCode,value] and the discrete violation-fingerprint set.
data Baseline = Baseline
  { bCont :: [[Integer]]
  , bDisc :: [Integer]
  }

data RatchetKnobs = RatchetKnobs
  { rTolNum :: Integer
  , rTolDen :: Integer
  , rTolAbs :: Integer
  }

ratchetBound :: RatchetKnobs
ratchetBound = RatchetKnobs Cost.tolNum Cost.tolDen Cost.tolAbs

data Ratcheted = Ratcheted
  { rOver :: [[Integer]]
  -- ^ [u,metricCode,value,allowed]: ceiling busted past tolerance.
  , rDrawn :: [[Integer]]
  -- ^ [u,metricCode,drawn]: over the ceiling but inside tolerance —
  -- consumption is REPORTED (plan: it lands in the Stop summary).
  , rAdded :: [Integer]
  , rRemoved :: [Integer]
  , rNewCont :: [[Integer]]
  , rNewDisc :: [Integer]
  }

-- | The single-edit ceiling allowance: max of the +2% leg (integer
-- div truncates DOWN — the conservative side, the "ties don't open"
-- stance) and the +10 leg. The legs cross at ceiling 500;
-- Spec.costModel pins one assertion on each side.
tolerated :: RatchetKnobs -> Integer -> Integer
tolerated k c = max (c * rTolNum k `div` rTolDen k) (c + rTolAbs k)

-- | Judge current facts against the baseline. Continuous: per
-- (entity, metric) — value above tolerated(ceiling) is OVER; above
-- the ceiling but tolerated is DRAWN tolerance; the new ceiling is
-- min(value, old) (auto-tighten), and entities the baseline never
-- saw adopt their value (bootstrap, not a violation). Discrete:
-- plain set difference both ways; the new set IS the current set —
-- the only-shrink stance (new ⊆ old) is the CALLER's acceptance
-- gate, not a fact this function may fake.
ratchet :: RatchetKnobs -> Maybe Baseline -> [[Integer]] -> [Integer] -> Ratcheted
ratchet _ Nothing cont disc = Ratcheted [] [] [] [] cont disc
ratchet k (Just b) cont disc =
  Ratcheted
    { rOver =
        [ [u, c, v, allowed]
        | ((u, c), (v, Just bv)) <- rows
        , let allowed = tolerated k bv
        , v > allowed
        ]
    , rDrawn =
        [ [u, c, v - bv]
        | ((u, c), (v, Just bv)) <- rows
        , bv < v
        , v <= tolerated k bv
        ]
    , rAdded = S.toAscList (curSet `S.difference` baseSet)
    , rRemoved = S.toAscList (baseSet `S.difference` curSet)
    , rNewCont = [[u, c, maybe v (min v) mbv] | ((u, c), (v, mbv)) <- rows]
    , rNewDisc = disc
    }
 where
  baseMap = M.fromList [((u, c), v) | [u, c, v] <- bCont b]
  rows =
    [ ((u, c), (v, M.lookup (u, c) baseMap))
    | [u, c, v] <- cont
    ]
  curSet = S.fromList disc
  baseSet = S.fromList (bDisc b)
