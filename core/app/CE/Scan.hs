-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan.request handler (ADR-008 P3): decode measurement rows
-- [code, value] plus optional grade overrides [code, warn, fail]
-- and the optional naming-facts table (2.30.0, aligned to the
-- code-6 rows), enforce the row cap (over-cap = a complete degraded
-- reply that FAILS — the P1 posture), machine-check the boundary
-- contract in request order (CE.Scan.Contract, split out at the
-- 300-line wall) — then grade every row through the ONE graded
-- verdict table, deriving each code-6 value from its facts when
-- they ride. Levels return positionally (row i answers level
-- i) and the fail bit is the exit-code semantic: any level-2 row.
-- Measurement and report rendering stay in Rust; only codes, values
-- and name-shape facts cross the wire — subjects, names and paths
-- never do (§5.9.2 index privacy).
module CE.Scan (respond) where

import CE.Scan.Contract (violation)
import CE.Scan.Cost (conforms, gradeTable, gradeWith, scanRowCap)
import CE.Scan.Cycles (withCycles)
import CE.Scan.Fence (Fence (..), drifted, readFence)
import CE.Wire (RowsReq (..), Rulepack (..), rowsFamily)
import Data.Aeson
import qualified Data.Map.Strict as M
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Maybe (fromMaybe, isJust)


-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  rowsFamily "scan" overCap violation (\req -> reply proto req [] [] True) (judged proto)

-- | Every request dimension counts toward the cap (review C15: the
-- declared ceiling missed the second dimension; the third arrived
-- with 2.30.0, the fourth and fifth with the 3.2.0 rulepack tables).
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
    ]

-- | Every row through the ONE graded table — its class's line where
-- the class overrides that code (3.2.0), else the global effective
-- line — with the code-6 values derived from the facts when they
-- ride.
judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req (zipWith (grade eff over) classes rows) moved False
 where
  rp = rulepackOf req
  eff = effective (gradesOf req)
  over = M.fromList [((c, code), (w, f)) | [c, code, w, f] <- overridesOf rp]
  classes = fromMaybe (repeat 0) (rowClassesOf rp)
  -- the two derivations that produce EFFECTIVE values, in the order
  -- they compose: the naming facts settle each code-6 row, then the
  -- call cycles raise the cognitive rows they contain
  (rows, moved) = withCycles (callsOf req) (withFacts (namingOf req) (rowsOf req))

-- | The naming table as sent, [] when absent — the cap's view; road
-- selection stays on namingOf's Maybe.
namingRows :: RowsReq -> [[Integer]]
namingRows = fromMaybe [] . namingOf

-- | The facts road (2.30.0): each code-6 row's effective value is
-- the conforms verdict over its aligned facts row — derived HERE,
-- by the judgment's owner; the legacy road (no naming key) keeps
-- judging the 0/1 the client sent, byte-identically.
withFacts :: Maybe [[Integer]] -> [[Integer]] -> [[Integer]]
withFacts Nothing rows = rows
withFacts (Just naming) rows = go naming rows
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

-- | levels ride positionally; the effective grade table is echoed
-- whole so the Rust client asserts the round trip (the P4 knob-echo
-- pattern, table form). A degraded reply carries fail=true — a gate
-- that could not judge must never pass, said by the core. Since
-- 6.4.0 (O33) the fail bit is the disjunction of NAMED conditions
-- and the names ride as `failed` exactly when `knobsFence` rode —
-- `hard_line` (a row at the FAIL tier), `knobs_digest` (the fence
-- pair disagrees), `degraded` (which stands alone, the verdict
-- tooLarge posture: nothing else was judged). A legacy request
-- keeps its bytes: the same bit, no names.
reply :: String -> RowsReq -> [Integer] -> [[Integer]] -> Bool -> B8.ByteString
reply proto req levels moved degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("scan.result" :: String)
    , "id" .= rowsId req
    , "levels" .= levels
    , "counts"
        .= object
          [ "rows" .= length (rowsOf req)
          , "warns" .= count 1
          , "fails" .= count 2
          ]
    , "fail" .= any snd conds
    , -- a degraded reply echoes the DEFAULTS: its overrides were
      -- never validated, and an unvalidated table must not be
      -- presented as effective (review C14; the Verdict tooLarge
      -- posture)
      "grades" .= [[c, w, f] | (c, (w, f)) <- effective (if degraded then [] else gradesOf req)]
    , "degraded" .= degraded
    ]
      <> ["failed" .= [name | (name, True) <- conds] | isJust fence]
      <> ["reason" .= ("scan_too_large" :: String) | degraded]
      -- the override table echoes exactly when it rode and was judged
      -- with (3.2.0): the client asserts the round trip; a legacy or
      -- degraded reply keeps its byte shape
      <> ["gradeOverrides" .= overrides | not degraded && not (null overrides)]
      -- the recursion increment echoes what it raised, and only when
      -- the arcs rode (6.5.0): [rowIndex, effectiveValue], ascending
      -- — the measuring side renders the judged number without ever
      -- deriving the cycle, or the increment, for itself
      <> ["cocBumped" .= moved | not degraded && isJust (callsOf req)]
      -- the judged-language mask echoes exactly when it rode and was
      -- judged with (7.2.0): the client pins it like the grade table;
      -- a legacy or degraded reply keeps its byte shape
      <> ["judgedMask" .= m | not degraded, Just m <- [maskOf req]]
 where
  overrides = overridesOf (rulepackOf req)
  count l = length (filter (== l) levels)
  -- validated by fenceOffence before any reply is judged; the
  -- degraded reply reads it too, and a malformed pair on that road
  -- reads as unfenced — nothing was judged, `degraded` names why
  fence = either (const Unfenced) id . readFence <$> fenceOf req
  conds :: [(String, Bool)]
  conds
    | degraded = [("degraded", True)]
    | otherwise = [("hard_line", count 2 > 0), ("knobs_digest", maybe False drifted fence)]
