-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan.request handler (ADR-008 P3): decode measurement rows
-- [code, value] plus optional grade overrides [code, warn, fail]
-- and the optional naming-facts table (2.30.0, aligned to the
-- code-6 rows), enforce the row cap (over-cap = a complete degraded
-- reply that FAILS — the P1 posture), machine-check the boundary
-- contract in request order — then grade every row through the ONE
-- graded verdict table, deriving each code-6 value from its facts
-- when they ride. Levels return positionally (row i answers level
-- i) and the fail bit is the exit-code semantic: any level-2 row.
-- Measurement and report rendering stay in Rust; only codes, values
-- and name-shape facts cross the wire — subjects, names and paths
-- never do (§5.9.2 index privacy).
module CE.Scan (respond) where

import CE.Scan.Cost (conforms, gradeTable, gradeWith, scanRowCap)
import CE.Scan.Fence (Fence (..), drifted, fenceOffence, readFence)
import CE.Wire (RowsReq (..), Rulepack (..), rowsFamily, tableOffence)
import Data.Aeson
import qualified Data.Map.Strict as M
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Control.Applicative ((<|>))
import Data.Foldable (asum)
import Data.List (find)
import Data.Maybe (fromMaybe, isJust)
-- the rulepack fence is ONE predicate for every family that carries
-- a class id (the verdict family minted it at 3.1.0; 6.4.0 spelled
-- the inclusive bound once, O38)
import CE.Verdict.Cost (classIdPastFence)


-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  rowsFamily "scan" overCap violation (\req -> reply proto req [] True) (judged proto)

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
    ]

-- | Every row through the ONE graded table — its class's line where
-- the class overrides that code (3.2.0), else the global effective
-- line — with the code-6 values derived from the facts when they
-- ride.
judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req (zipWith (grade eff over) classes rows) False
 where
  rp = rulepackOf req
  eff = effective (gradesOf req)
  over = M.fromList [((c, code), (w, f)) | [c, code, w, f] <- overridesOf rp]
  classes = fromMaybe (repeat 0) (rowClassesOf rp)
  rows = withFacts (namingOf req) (rowsOf req)

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

-- | First boundary-contract offender in request order (Clone.hs
-- posture: the message names the violator deterministically); the
-- ascending pass compares CODES alone (warn values legitimately
-- vary), through CE.Wire's shared checker.
violation :: RowsReq -> Maybe String
violation req =
  asum
    [ asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
    , tableOffence "grade" (take 1) gradeShape (gradesOf req)
    , namingBattery req
    , classBattery req
    , fenceOffence (fenceOf req)
    ]

-- | The naming-facts table's own contract (2.30.0): aligned 1:1
-- with the code-6 rows in request order, each row the five shape
-- facts — and the verdict provably absent from the wire: a code-6
-- row must carry value 0 when facts ride (the staleDocs lesson,
-- inverted before shipping this time: one judgment, one road).
namingBattery :: RowsReq -> Maybe String
namingBattery req = case namingOf req of
  Nothing -> Nothing
  Just naming ->
    asum
      [ counts naming
      , asum (zipWith namingShape [0 :: Int ..] naming)
      , asum (zipWith preJudged [0 :: Int ..] (rowsOf req))
      ]
 where
  counts naming
    | length naming /= fnRows =
        Just ("naming: " <> show (length naming) <> " rows for " <> show fnRows <> " fn-naming rows")
    | otherwise = Nothing
  fnRows = length [() | (6 : _) <- rowsOf req]
  preJudged i row = case row of
    [6, v] | v /= 0 -> Just ("row " <> show i <> ": pre-judged fn-naming value (naming facts ride)")
    _ -> Nothing

-- | The rulepack channel's own contract (3.2.0): a class column
-- aligned 1:1 with the rows, every class below the fence; override
-- rows [classId, code, warn, fail] from class 1 below the fence
-- (class 0 IS the global table, which `grades` already overrides),
-- a known code, a coherent ladder — the gradeShape reading, one
-- class dimension wider — and (classId, code) strictly ascending.
classBattery :: RowsReq -> Maybe String
classBattery req =
  (rowClassesOf rp >>= aligned)
    <|> tableOffence "gradeOverride" (take 2) overrideShape (overridesOf rp)
 where
  rp = rulepackOf req
  n = length (rowsOf req)
  aligned cs
    | length cs /= n = Just ("rowClasses: " <> show (length cs) <> " classes for " <> show n <> " rows")
    | otherwise = past <$> find (\(_, c) -> c < 0 || classIdPastFence c) (zip [0 :: Int ..] cs)
  past (i, _) = "rowClasses " <> show (i :: Int) <> ": class beyond the fence"

overrideShape :: Int -> [Integer] -> Maybe String
overrideShape i row = case row of
  (c : rest@[_, _, _])
    | c < 1 -> Just (label <> "class 0 has no override channel")
    | classIdPastFence c -> Just (label <> "class beyond the fence")
    | otherwise -> ladderShape label rest
  _ -> Just (label <> "malformed row (need [class,code,warn,fail])")
 where
  label = "gradeOverride " <> show i <> ": "

namingShape :: Int -> [Integer] -> Maybe String
namingShape i row = case row of
  [lang, style, upper, under, test]
    | lang < 0 || lang > 6 -> Just (label <> "lang outside the judged set")
    | style < 0 || style > 2 -> Just (label <> "unknown style")
    | any (`notElem` [0, 1]) [upper, under, test] -> Just (label <> "non-boolean fact")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [lang,style,upper,under,test])")
 where
  label = "naming " <> show i <> ": "

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [code, v]
    | code < 0 || code > 6 -> Just (label <> "unknown metric code")
    | v < 0 -> Just (label <> "negative value")
    | v >= 18446744073709551616 -> Just (label <> "value outside u64")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [code,value])")
 where
  label = "row " <> show i <> ": "

-- | A grade override must stay a coherent ladder: a hard line BELOW
-- the warn line is refused, never silently reordered. fail == warn
-- is deliberately legal (review C19 ruling): it is the single-line
-- config — every breach grades 2 and the warn band is empty by the
-- user's own choice, which gradeWith honors on both sides of the
-- mirror.
gradeShape :: Int -> [Integer] -> Maybe String
gradeShape i = ladderShape ("grade " <> show i <> ": ")

-- | One [code, warn, fail] ladder under a caller's label — the grade
-- table's rows and (3.2.0) the tail of every override row read
-- through the same predicate: the first held fault, by name.
ladderShape :: String -> [Integer] -> Maybe String
ladderShape label row = case row of
  [code, warn, failLine] ->
    (label <>) . snd
      <$> find
        fst
        [ (code < 0 || code > 6, "unknown metric code")
        , (warn < 0 || failLine < 0, "negative field")
        , (failLine /= 0 && failLine < warn, "fail line below warn")
        ]
  _ -> Just (label <> "malformed row (need [code,warn,fail])")

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
reply :: String -> RowsReq -> [Integer] -> Bool -> B8.ByteString
reply proto req levels degraded =
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
