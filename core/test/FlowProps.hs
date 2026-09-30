-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The flow family's battery (plan v2.31 step 3): the hand-written
-- cases of FlowCases (each finding kind, try / catch / finally,
-- loops and switches, goto and label, the exemptions, the dynamic
-- skip; every contract refusal by name), then two hundred generated
-- programs against the trace reference, the cap, the counts of a
-- skipped unit and the empty request. Scaffolding lives in
-- WireHarness, FlowCases and ReferenceFlow; the probes read their
-- reply fields through `fieldsOf` in one projection each.
module FlowProps (battery) where

import CE.Flow (respond)
import CE.Flow.Contract (FlowReq (..), overCap)
import CE.Flow.Cost (rowCap)
import Data.Aeson
import FlowCases (Case (..), judgments)
import FlowRefusals (refusals)
import ReferenceFlow (referenceFindings)
import ReferenceFlowGen (programs, request)
import WireHarness (fieldsOf, refusedBy, replyObjWith, runLegs, setKey, tabledRequest)

battery :: IO Bool
battery = runLegs (map caseName table <> names) (map holds table <> probes)
 where
  table = judgments <> refusals

names :: [String]
names =
  [ "two hundred generated programs agree with the trace reference finding for finding"
  , "an over-cap request degrades with no findings and its counts; the cap counts the four tables together"
  , "a skipped unit is counted, and the counts name the four tables"
  , "an empty request answers no findings"
  ]

probes :: [Bool]
probes = [generated, capped, dynamicCounted, emptyRequest]

-- | A case holds when the core answers its findings, or refuses by
-- its message.
holds :: Case -> Bool
holds c = case caseRefusal c of
  Just message -> refusedBy respond (caseRequest c) message
  Nothing -> findingsOf (caseRequest c) == Just [rows (caseExpect c)]

findingsOf :: Value -> Maybe [Maybe Value]
findingsOf r = fieldsOf respond r ["findings"]

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

blank :: Value
blank = tabledRequest "7.0.0" "flow.request" [("units", []), ("stmts", []), ("vars", []), ("uses", [])]

generated :: Bool
generated = and [findingsOf (request p) == Just [rows (referenceFindings p)] | p <- programs]

-- | 524,289 statement rows through the real respond; the cap
-- predicate at the boundary across the four tables.
capped :: Bool
capped =
  fieldsOf respond big ["degraded", "reason", "findings"] == Just [Just (Bool True), Just "flow_too_large", rows []]
    && overCap (FlowReq Null [] (replicate half [0]) [] (replicate (half + 1) [0]) (Left ""))
    && not (overCap (FlowReq Null [] (replicate half [0]) [] (replicate half [0]) (Left "")))
 where
  half = fromInteger rowCap `div` 2
  big = setKey "stmts" (toJSON (replicate (fromInteger rowCap + 1) [0, 0, -1, 1, 0, 0 :: Integer])) blank

-- | The dynamic case is the judgments table's last: one unit, two
-- statements, one variable, two accesses, no finding, one skipped.
dynamicCounted :: Bool
dynamicCounted =
  fieldsOf respond (caseRequest (last judgments)) ["counts"]
    == Just [Just (object (zipWith (.=) ["units", "stmts", "vars", "uses", "findings", "dynamicUnits"] [1, 2, 1, 2, 0, 1 :: Int]))]

emptyRequest :: Bool
emptyRequest =
  fieldsOf respond blank ["findings", "degraded", "reason"] == Just [rows [], Just (Bool False), Nothing]
    && maybe False (const True) (replyObjWith respond blank)
