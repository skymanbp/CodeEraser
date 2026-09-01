-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The scan family's battery (ADR-008 P3): the graded verdict
-- table's boundaries in both directions at both lines, the override
-- lever through the REAL respond (a knob value must flip a named
-- level — the charter's counterfactual), refusal-by-name, and the
-- degraded-fails posture inherited from P1. Scaffolding lives in
-- WireHarness — this module keeps only its probes.
module ScanProps (battery) where

import CE.Scan (respond)
import CE.Scan.Cost (gradeWith, scanRowCap)
import Data.Aeson
import WireHarness (field, refusedBy, replyObjWith, rowsRequest, runChecks, setKey)

battery :: IO Bool
battery = do
  base <-
    runChecks
      [ ("grade boundaries: both lines, both directions", boundaries)
      , ("levels ride positionally through the real respond", positional)
      , ("a grade override flips the named row's level and echoes", overrideLever)
      , ("scan refusals name the offender with code and message", refusals)
      , ("the facts road derives fn-naming, exemption gated on Go", factsRoad)
      , ("naming refusals: alignment, shape, the pre-judged value", namingRefusals)
      , ("an over-cap scan request degrades to a reply that FAILS", degradedFails)
      ]
  -- the rulepack channel (3.2.0), its own table
  rulepack <-
    runChecks
      [ ("a class line moves ITS rows' level, the global row's not, and echoes", classLever)
      , ("class refusals: alignment, the fence, class 0, the ladder, the order", classRefusals)
      ]
  -- the fence channel (6.4.0), its own table
  fence <-
    runChecks
      [ ("K51: knobsFence answers failed in canonical order, and its absence is a byte", fenceRoad)
      ]
  pure (base && rulepack && fence)

-- | K51 (6.4.0, O33): the fence value rides as sent — null (no
-- committed baseline) and a matching pair name nothing; a drifted
-- pair (a null half included) names `knobs_digest`; a hard-line row
-- beside a drift names both in the core's order; a request without
-- the key answers no `failed` at all (the legacy bytes); a malformed
-- value refuses by name; and the over-cap face names `degraded`
-- alone. The fail bit is the disjunction of the names every time.
fenceRoad :: Bool
fenceRoad =
  failedOf (fenced Null [[1, 50]]) == Just []
    && failedOf (fenced (pair (Just 5) (Just 5)) [[1, 50]]) == Just []
    && failedOf (fenced (pair Nothing Nothing) [[1, 50]]) == Just []
    && failedOf (fenced (pair (Just 5) (Just 6)) [[1, 50]]) == Just ["knobs_digest"]
    && failedOf (fenced (pair Nothing (Just 6)) [[1, 50]]) == Just ["knobs_digest"]
    && failedOf (fenced (pair (Just 5) (Just 6)) [[0, 800]]) == Just ["hard_line", "knobs_digest"]
    && failedOf (fenced Null [[0, 800]]) == Just ["hard_line"]
    && failOf (fenced (pair (Just 5) (Just 6)) [[1, 50]]) == Just True
    && failOf (fenced Null [[1, 50]]) == Just False
    && fmap (field' "failed") (replyObj (wireReq [[1, 50]])) == Just Nothing
    && refusedBy respond (fenced (toJSON [5 :: Integer]) [[1, 50]]) "knobsFence: malformed"
    && failedOf (fenced Null [[0, 0] | _ <- [0 .. scanRowCap]]) == Just ["degraded"]
 where
  fenced v rows = setKey "knobsFence" v (wireReq rows)
  pair a b = toJSON [a, b :: Maybe Integer]
  field' k o = field o k
  failedOf req = case replyObj req of
    Just o -> case field o "failed" of
      Just v -> case fromJSON v :: Result [String] of
        Success names -> Just names
        _ -> Nothing
      Nothing -> Nothing
    Nothing -> Nothing
  failOf req = case replyObj req of
    Just o -> case field o "fail" of
      Just (Bool b) -> Just b
      _ -> Nothing
    Nothing -> Nothing

-- | The SHIPPED comparison at the file-lines row (300/750): clean at
-- the line, warn one past it, warn AT the hard line, fail one past
-- it — and the no-hard-line rows (fail 0) never grade 2, including
-- the boolean naming row (warn 0).
boundaries :: Bool
boundaries =
  map (gradeWith (300, 750)) [300, 301, 750, 751] == [0, 1, 1, 2]
    && map (gradeWith (5, 0)) [5, 6, 100000] == [0, 1, 1]
    && map (gradeWith (0, 0)) [0, 1] == [0, 1]

wireReq :: [[Integer]] -> Value
wireReq = rowsRequest "6.5.0" "scan.request"

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

positional :: Bool
positional = case replyObj (wireReq [[0, 301], [1, 50], [6, 1], [0, 800]]) of
  Nothing -> False
  Just o ->
    field o "levels" == Just (toJSON [1, 0, 1, 2 :: Integer])
      && field o "fail" == Just (Bool True)

-- | The charter's counterfactual: the SAME 310-line row is a warn
-- under the defaults and clean under a requested [0,400,750] — and
-- the echoed grade table carries the override, so the Rust mirror
-- pin has a surface to hold.
overrideLever :: Bool
overrideLever = case (run Nothing, run (Just [[0, 400, 750]])) of
  (Just (l0, g0), Just (l1, g1)) ->
    l0 == toJSON [1 :: Integer]
      && l1 == toJSON [0 :: Integer]
      && headGrade g0 == Just (toJSON [0, 300, 750 :: Integer])
      && headGrade g1 == Just (toJSON [0, 400, 750 :: Integer])
  _ -> False
 where
  -- explicit signature at the definition (the GHC-39999 where-block
  -- polymorphism lesson, third occurrence): Nothing leaves the
  -- override element type ambiguous otherwise
  run :: Maybe [[Integer]] -> Maybe (Value, Value)
  run over = do
    o <- replyObj (maybe id (setKey "grades" . toJSON) over (wireReq [[0, 310]]))
    (,) <$> field o "levels" <*> field o "grades"
  headGrade v = case v of
    Array rows -> foldr (\x _ -> Just x) Nothing rows
    _ -> Nothing

refusals :: Bool
refusals =
  and
    [ refused (wireReq [[7, 1]]) "unknown metric code"
    , refused (wireReq [[0, -1]]) "negative value"
    , refused (wireReq [[0]]) "malformed row"
    , refused (gradeReq [[0, 400]]) "malformed row (need [code,warn,fail])"
    , refused (gradeReq [[0, 400, 300]]) "fail line below warn"
    , refused (gradeReq [[1, 50, 75], [0, 300, 750]]) "not strictly ascending"
    ]
 where
  gradeReq gs = setKey "grades" (toJSON (gs :: [[Integer]])) (wireReq [])
  refused = refusedBy respond

-- | The 2.30.0 counterfactual ON THE WIRE: identical facts
-- (mixedCaps, underscore, go-vet test shape) are exempt under Go's
-- lang code and a warn under TypeScript's — the leak the Rust-side
-- predicate carried; snake facts judge on the uppercase bit; an
-- empty facts table beside function-free rows is legal; and the
-- legacy road (no naming key) still judges the sent 0/1.
factsRoad :: Bool
factsRoad =
  and
    [ run [[4, 2, 0, 1, 1]] [[6, 0]] == Just (toJSON [0 :: Integer])
    , run [[1, 2, 0, 1, 1]] [[6, 0]] == Just (toJSON [1 :: Integer])
    , run [[0, 1, 1, 0, 0]] [[6, 0]] == Just (toJSON [1 :: Integer])
    , run [] [[0, 301]] == Just (toJSON [1 :: Integer])
    , (replyObj (wireReq [[6, 1]]) >>= \o -> field o "levels") == Just (toJSON [1 :: Integer])
    ]
 where
  run naming rows =
    replyObj (setKey "naming" (toJSON (naming :: [[Integer]])) (wireReq rows))
      >>= \o -> field o "levels"

-- | Every refusal the naming table can earn, named: a facts row
-- with no code-6 row to bind to, the judged-set and style and
-- boolean fences, the malformed shape, and a code-6 row that tried
-- to carry the verdict past the facts.
namingRefusals :: Bool
namingRefusals =
  and
    [ ref [[4, 2, 0, 1, 1]] [] "naming: 1 rows for 0 fn-naming rows"
    , ref [[7, 2, 0, 1, 1]] [[6, 0]] "naming 0: lang outside the judged set"
    , ref [[4, 3, 0, 1, 1]] [[6, 0]] "naming 0: unknown style"
    , ref [[4, 2, 2, 1, 1]] [[6, 0]] "naming 0: non-boolean fact"
    , ref [[4, 2, 0, 1]] [[6, 0]] "malformed row (need [lang,style,upper,under,test])"
    , ref [[4, 2, 0, 1, 1]] [[6, 1]] "row 0: pre-judged fn-naming value"
    ]
 where
  ref naming rows = refusedBy respond (setKey "naming" (toJSON (naming :: [[Integer]])) (wireReq rows))

-- | The P3 counterfactual (plan v2.13 ①): two identical 60-line
-- fn rows, one in class 1 whose fn line sits at 80, one on the
-- global table — the classed row grades clean, the global one warns;
-- the override table echoes when it rode and is absent when it did
-- not; and a class column alone (no override) judges like the
-- global table.
classLever :: Bool
classLever =
  levels classed == Just (toJSON [0, 1 :: Integer])
    && echo classed == Just (toJSON over)
    && levels unclassed == Just (toJSON [1, 1 :: Integer])
    && echo unclassed == Nothing
 where
  over = [[1, 1, 80, 90 :: Integer]]
  unclassed = setKey "rowClasses" (toJSON [1, 0 :: Integer]) classRows
  classed = setKey "gradeOverrides" (toJSON over) unclassed
  levels r = replyObj r >>= \o -> field o "levels"
  echo r = replyObj r >>= \o -> field o "gradeOverrides"

classRows :: Value
classRows = wireReq [[1, 60], [1, 60]]

-- | Every refusal the rulepack channel can earn, by name: a class
-- column that does not align, a class at the fence (both tables),
-- class 0 in the override table, an incoherent override ladder, a
-- malformed override, and a disordered table.
classRefusals :: Bool
classRefusals = all (uncurry (refusedBy respond)) probes
 where
  probes =
    [ (classes [1], "rowClasses: 1 classes for 2 rows")
    , (classes [65, 0], "rowClasses 0: class beyond the fence")
    , (over [[0, 1, 80, 90]], "class 0 has no override channel")
    , (over [[65, 1, 80, 90]], "class beyond the fence")
    , (over [[1, 1, 80, 70]], "gradeOverride 0: fail line below warn")
    , (over [[1, 1, 80]], "malformed row (need [class,code,warn,fail])")
    , (over [[2, 0, 400, 750], [1, 0, 400, 750]], "gradeOverride 1: not strictly ascending")
    ]
  classes cs = setKey "rowClasses" (toJSON (cs :: [Integer])) classRows
  over os = setKey "gradeOverrides" (toJSON (os :: [[Integer]])) classRows

-- | P1 posture on the new family: one row past the cap degrades to
-- a complete reply whose fail bit is TRUE.
degradedFails :: Bool
degradedFails = case replyObj (wireReq [[0, 0] | _ <- [0 .. scanRowCap]]) of
  Nothing -> False
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
