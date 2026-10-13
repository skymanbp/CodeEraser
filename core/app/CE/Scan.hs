-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan.request handler (ADR-008 P3): decode measurement rows
-- [code, value] plus optional grade overrides [code, warn, fail]
-- and the naming-facts table (2.30.0, aligned to the code-6 rows),
-- the events table (7.2.0, each unit's structure, from which its three
-- complexity rows are derived) and the fence (6.4.0) — the three
-- required since 9.0.0 — enforce the row cap (over-cap = a complete degraded
-- reply that FAILS — the P1 posture), machine-check the boundary
-- contract in request order (CE.Scan.Contract, split out at the
-- 300-line wall) — then grade every row through the ONE graded
-- verdict table, deriving each code-6 value from its facts when
-- they ride. Levels return positionally (row i answers level
-- i) and the fail bit is the exit-code semantic: any level-2 row.
-- Measurement and report rendering stay in Rust; only codes, values
-- and name-shape facts cross the wire — subjects, names and paths
-- never do (§5.9.2 index privacy).
module CE.Scan (conditionNames, respond) where

import CE.Scan.Contract (violation)
import CE.Scan.Cost (conforms, gradeTable, gradeWith, scanRowCap)
import CE.Scan.Cycles (withCycles)
import CE.Scan.Events (derivedRows, withEvents)
import CE.Scan.Fence (Fence (..), drifted, readFence)
import CE.Wire (Respond, RowsReq (..), Rulepack (..), resultLine, rowsFamily, tooLarge)
import qualified CE.Wire.Families as F
import Data.Aeson
import qualified Data.Map.Strict as M
import qualified Data.ByteString.Char8 as B8
import Data.Maybe (fromMaybe, isJust)


-- | The shared cascade with this family's bindings (CE.Wire).
respond :: Respond
respond proto =
  rowsFamily F.scan overCap violation (\req -> reply proto req (Echo [] [] []) True) (judged proto)

-- | Every request dimension counts toward the cap (review C15: the
-- declared ceiling missed the second dimension; the third arrived
-- with 2.30.0, the fourth and fifth with the 3.2.0 rulepack tables,
-- the seventh with the 7.2.0 events table).
overCap :: RowsReq -> Bool
overCap req = toInteger (sum dims) > scanRowCap
 where
  rp = rulepackOf req
  dims =
    [ length (rowsOf req)
    , length (gradesOf req)
    , length (namingRows req)
    , length (overridesOf rp)
    , maybe 0 length (rowClassesOf rp)
    , maybe 0 length (callsOf req)
    , length (eventRows req)
    ]

-- | Every row through the ONE graded table — its class's line where
-- the class overrides that code (3.2.0), else the global effective
-- line — with the code-6 values derived from the facts when they
-- ride.
judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req (Echo (zipWith (grade eff over) classes rows) moved (derivedRows rows)) False
 where
  rp = rulepackOf req
  eff = effective (gradesOf req)
  over = M.fromList [((c, code), (w, f)) | [c, code, w, f] <- overridesOf rp]
  classes = fromMaybe (repeat 0) (rowClassesOf rp)
  -- the three derivations that produce EFFECTIVE values, in the order
  -- they compose: the events fold each unit's three complexity rows,
  -- the naming facts settle each code-6 row, then the call cycles
  -- raise the cognitive rows they contain
  (rows, moved) = withCycles (callsOf req) (withFacts (namingRows req) (withEvents (eventRows req) (rowsOf req)))

-- | The naming and events tables as sent — the cap's view and, past the
-- contract (both required since 9.0.0), the judgment's; [] only on a
-- request the cap degrades before the contract reads it.
namingRows, eventRows :: RowsReq -> [[Integer]]
namingRows = fromMaybe [] . namingOf
eventRows = fromMaybe [] . eventsOf

-- | The facts road (2.30.0): each code-6 row's effective value is
-- the conforms verdict over its aligned facts row — derived HERE,
-- by the judgment's owner.
withFacts :: [[Integer]] -> [[Integer]] -> [[Integer]]
withFacts naming rows = go naming rows
 where
  go _ [] = []
  go ns (row : rest) = case (row, ns) of
    ([6, _], n : ns') -> [6, if conforms n then 0 else 1] : go ns' rest
    _ -> row : go ns rest

-- | The effective grade table: every default row, overridden per
-- code by the request (the effectiveKnobs pattern — absent rows
-- keep the Cost.hs DEFAULTS; ce.toml is the source on the Rust
-- side, this wire is the road).
effective :: [[Integer]] -> [(Integer, (Integer, Integer))]
effective overrides =
  [(c, pick c (w, f)) | (c, w, f) <- gradeTable]
 where
  pick c dflt = last (dflt : [(w, f) | [c', w, f] <- overrides, c' == c])

-- | One row against its class's line where the class overrides that
-- code (3.2.0), else the global effective line — a Map lookup with
-- the global pair as the default, so class 0 and an unoverridden
-- code both judge exactly as before.
grade ::
  [(Integer, (Integer, Integer))] ->
  M.Map (Integer, Integer) (Integer, Integer) ->
  Integer ->
  [Integer] ->
  Integer
grade table over cls row = case row of
  [code, v] | Just wf <- lookup code table -> gradeWith (M.findWithDefault wf (cls, code) over) v
  _ -> error "row shape enforced by violation"

-- | What a judged reply echoes beside its grades: the levels, the
-- cognitive rows the recursion increment raised (6.5.0) and the
-- derived complexity rows (7.2.0) — all empty on the degraded face.
data Echo = Echo [Integer] [[Integer]] [[Integer]]

-- | levels ride positionally; the effective grade table is echoed
-- whole so the Rust client asserts the round trip (the P4 knob-echo
-- pattern, table form). A degraded reply carries fail=true — a gate
-- that could not judge must never pass, said by the core. Since
-- 6.4.0 (O33) the fail bit is the disjunction of the NAMED
-- conditions (`conditions`), which ride as `failed` on every reply
-- (the fence is required since 9.0.0).
reply :: String -> RowsReq -> Echo -> Bool -> B8.ByteString
reply proto req (Echo levels moved derived) degraded =
  resultLine
    proto
    F.scan
    (rowsId req)
    ( [ "levels" .= levels
      , "fail" .= any snd conds
      , -- a degraded reply echoes the DEFAULTS: its overrides were
        -- never validated, and an unvalidated table must not be
        -- presented as effective (review C14; the Verdict tooLarge
        -- posture)
        "grades" .= [[c, w, f] | (c, (w, f)) <- effective (if degraded then [] else gradesOf req)]
      ]
        <> ["failed" .= [name | (name, True) <- conds]]
        -- the override table echoes exactly when it rode and was judged
        -- with (3.2.0): the client asserts the round trip; a legacy or
        -- degraded reply keeps its byte shape
        <> ["gradeOverrides" .= overrides | not degraded && not (null overrides)]
        -- the recursion increment echoes what it raised, and only when
        -- the arcs rode (6.5.0): [rowIndex, effectiveValue], ascending
        <> ["cocBumped" .= moved | not degraded && isJust (callsOf req)]
        -- the derived complexity rows echo on every judged reply (7.2.0;
        -- the events required since 9.0.0): [rowIndex, value] for every
        -- cyclomatic, cognitive and nesting row, post recursion — the
        -- measuring side renders the numbers without deriving one
        <> ["derived" .= derived | not degraded]
    )
    ["rows" .= length (rowsOf req), "warns" .= count 1, "fails" .= count 2]
    (tooLarge F.scan degraded)
 where
  overrides = overridesOf (rulepackOf req)
  count l = length (filter (== l) levels)
  fence = fenced req
  conds = conditions fence levels degraded

-- | The fence as sent (6.4.0): validated by fenceOffence before any
-- reply is judged; the degraded reply reads it too, and a malformed
-- pair or an absent key on that road reads as unfenced — nothing was
-- judged, `degraded` names why.
fenced :: RowsReq -> Fence
fenced req = maybe Unfenced (either (const Unfenced) id . readFence) (fenceOf req)

-- | The named conditions the fail bit is the disjunction of (6.4.0,
-- O33): `hard_line` (a row at the FAIL tier), `knobs_digest` (the
-- fence pair disagrees), `degraded` (which stands alone, the verdict
-- tooLarge posture: nothing else was judged).
conditions :: Fence -> [Integer] -> Bool -> [(String, Bool)]
conditions _ _ True = [("degraded", True)]
conditions fence levels False =
  [("hard_line", any (== 2) levels), ("knobs_digest", drifted fence)]

-- | The conditions by name in their canonical order — the scan
-- report's `failed` (plan v2.32 step 5) — read off `conditions`, the
-- one spelling.
conditionNames :: [String]
conditionNames = map fst (conditions Unfenced [] False <> conditions Unfenced [] True)
