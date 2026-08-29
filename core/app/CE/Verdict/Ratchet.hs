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
-- [u,metricCode,value], the discrete violation-fingerprint set,
-- (v0.6, plan v2.6 §B) the frozen soft line — Nothing on a
-- pre-v0.6 file, re-derived only at establish — and (5.1.0, plan
-- v2.14 ②) the rulepack FINGERPRINT the ceilings were established
-- under. The digest is a scalar, not a rule: what it fences is the
-- silent kind of loosening, where a glob edit moves every line and
-- the baseline still looks agreed-to.
data Baseline = Baseline
  { bCont :: [[Integer]]
  , bDisc :: [Integer]
  , bSoft :: Maybe Integer
  , bKnobsDigest :: Maybe Integer
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
  , rDropped :: [[Integer]]
  -- ^ [u,metricCode,value] (6.4.0, O40): a baseline row whose entity
  -- the client lists as PRESENT (on disk, under the scope) yet sent
  -- no current row for — scope loss, named, never a silent removal.
  , rNewCont :: [[Integer]]
  , rNewDisc :: [Integer]
  }

-- | The single-edit ceiling allowance. Globally: max of the +2% leg
-- (integer div truncates DOWN — the conservative side, the "ties
-- don't open" stance) and the +10 leg; the legs cross at ceiling 500
-- and Spec.costModel pins one assertion on each side.
--
-- A class that declares its own tolerance REPLACES both legs with an
-- absolute allowance (5.1.0). Absolute, not proportional, on purpose:
-- the classes that want this knob are vendored trees and fixtures,
-- which want either zero slack or a fixed number of lines, and a
-- percentage of a large file is exactly the unearned growth the plan
-- objected to. t = 0 therefore means ANY growth is over — the global
-- legs cannot rescue it, because they are not consulted.
tolerated :: RatchetKnobs -> Maybe Integer -> Integer -> Integer
tolerated _ (Just t) c = c + t
tolerated k Nothing c = max (c * rTolNum k `div` rTolDen k) (c + rTolAbs k)

-- | Judge current facts against the baseline. Continuous: per
-- (entity, metric) — value above tolerated(ceiling) is OVER; above
-- the ceiling but tolerated is DRAWN tolerance; the new ceiling is
-- min(value, old) (auto-tighten), and entities the baseline never
-- saw adopt their value (bootstrap, not a violation). A baseline
-- row with no current row is a removal — unless its entity is in
-- the client's PRESENT set (6.4.0, O40), in which case the file is
-- on disk and simply stopped being measured (an exclude, a
-- .ceignore line, a scope), and the row is DROPPED: named, failing,
-- and gone from the new baseline only through a named act. Absent
-- set = the legacy road. Discrete: plain set difference both ways;
-- the new set IS the current set — the only-shrink stance (new ⊆
-- old) is the CALLER's acceptance gate, not a fact this function
-- may fake. The class allowance is asked per (class, metric):
-- code 4 answers metric 1 where declared, code 3 otherwise (O37).
ratchet ::
  RatchetKnobs ->
  (Integer -> Integer -> Maybe Integer) ->
  Maybe [Integer] ->
  Maybe Baseline ->
  [[Integer]] ->
  [Integer] ->
  Ratcheted
ratchet _ _ _ Nothing cont disc = Ratcheted [] [] [] [] [] (map identity cont) disc
ratchet k classTol present (Just b) cont disc =
  Ratcheted
    { rOver =
        [ [u, c, v, allowed]
        | ((u, c), (v, Just _, allowed)) <- rows
        , v > allowed
        ]
    , rDrawn =
        [ [u, c, v - bv]
        | ((u, c), (v, Just bv, allowed)) <- rows
        , bv < v
        , v <= allowed
        ]
    , rAdded = S.toAscList (curSet `S.difference` baseSet)
    , rRemoved = S.toAscList (baseSet `S.difference` curSet)
    , rDropped =
        [ [u, c, v]
        | Just here <- [S.fromList <$> present]
        , [u, c, v] <- bCont b
        , S.member u here
        , S.notMember (u, c) curKeys
        ]
    , rNewCont = [[u, c, maybe v (min v) mbv] | ((u, c), (v, mbv, _)) <- rows]
    , rNewDisc = disc
    }
 where
  baseMap = M.fromList [((u, c), v) | [u, c, v] <- bCont b]
  -- the class is read from the CURRENT row and spends only on the
  -- allowance; the baseline stays three columns, so a class is a
  -- charging parameter and never a ratchet fact (plan v2.13 ①)
  rows =
    [ ((u, c), (v, mbv, maybe 0 (tolerated k (classTol cls c)) mbv))
    | (u : c : v : rest) <- cont
    , let cls = classOf rest
    , let mbv = M.lookup (u, c) baseMap
    ]
  -- the 4th column when it rides, class 0 (the global table) when it
  -- does not: an unclassed row keeps the global legs exactly
  classOf (x : _) = x
  classOf [] = 0
  curSet = S.fromList disc
  baseSet = S.fromList (bDisc b)
  curKeys = S.fromList [(u, c) | (u : c : _) <- cont]

-- | A row stripped to its ratchet identity and value: the class
-- column, when it rides, is not part of what a baseline records.
identity :: [Integer] -> [Integer]
identity = take 3
