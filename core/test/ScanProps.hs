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
import Data.List (isInfixOf)
import qualified Data.ByteString.Lazy as BL
import WireHarness (field, replyObjWith, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("grade boundaries: both lines, both directions", boundaries)
    , ("levels ride positionally through the real respond", positional)
    , ("a grade override flips the named row's level and echoes", overrideLever)
    , ("scan refusals name the offender with code and message", refusals)
    , ("an over-cap scan request degrades to a reply that FAILS", degradedFails)
    ]

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
wireReq rows =
  object
    [ "proto" .= ("2.7.0" :: String)
    , "type" .= ("scan.request" :: String)
    , "id" .= (1 :: Int)
    , "rows" .= rows
    ]

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
  refused r want = case respond "0.0.1" (BL.toStrict (encode r)) of
    Left (_, code, msg) -> code == "contract" && want `isInfixOf` msg
    Right _ -> False

-- | P1 posture on the new family: one row past the cap degrades to
-- a complete reply whose fail bit is TRUE.
degradedFails :: Bool
degradedFails = case replyObj (wireReq [[0, 0] | _ <- [0 .. scanRowCap]]) of
  Nothing -> False
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
