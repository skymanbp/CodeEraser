-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The flow family's battery (plan v2.31 step 3): the hand-written
-- cases of FlowCases (each finding kind, try / catch / finally,
-- loops and switches, goto and label, the exemptions, the dynamic
-- skip; every contract refusal by name), then two hundred generated
-- requests against the trace reference — every finding row with its
-- judged bit, and the dynamic units counted — and the shapes those
-- requests must reach, the cap, the counts of a skipped unit and the
-- empty request. The judged bit is held to ReferenceFlow's own copy of
-- §5's two tables everywhere, the hand cases too. Scaffolding lives in
-- WireHarness, FlowCases and ReferenceFlow; the probes read their
-- reply fields through `fieldsOf` in one projection each.
module FlowProps (battery) where

import CE.Flow (respond)
import CE.Flow.Contract (FlowReq (..), overCap)
import CE.Flow.Cost
import Data.Aeson
import Data.Bits (testBit)
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import FlowCases (Case (..), judgments)
import FlowRefusals (refusals)
import ReferenceFlow (dynamicUnits, judgedBit, referenceFindings)
import ReferenceFlowGen (Node (..), Prog (..), batches, langCodes, request)
import WireHarness (field, fieldsOf, generated, refusedBy, replyObjWith, runChecks, runLegs, setKey, tabledRequest)

battery :: IO Bool
battery = (&&) <$> runLegs (map caseName table <> names) (map holds table <> probes) <*> runChecks legs
 where
  table = judgments <> refusals
  legs = generated ("two hundred generated requests agree with the trace reference finding for finding, judged bits and dynamic units", "the generated requests reach every rule shape §5 states") agrees reached shapes batches

names :: [String]
names =
  [ "an over-cap request degrades with no findings and its counts; the cap counts the four tables together"
  , "a skipped unit is counted, and the counts name the four tables"
  , "an empty request answers no findings"
  ]

probes :: [Bool]
probes = [capped, dynamicCounted, emptyRequest]

-- | A case holds when the core answers its findings, or refuses by
-- its message.
holds :: Case -> Bool
holds c = case caseRefusal c of
  Just message -> refusedBy respond (caseRequest c) message
  Nothing -> findingsOf (caseRequest c) == Just [rows (caseExpect c)]

-- | The core's finding rows with the judged bit (9.0.0, the sixth
-- column) taken off once it is held to ReferenceFlow's table over the
-- unit's language in the request's units table: the hand cases state
-- the five trace columns. A bit off the table reads as no answer.
findingsOf :: Value -> Maybe [Maybe Value]
findingsOf r = map (>>= traced) <$> fieldsOf respond r ["findings"]
 where
  traced v = case fromJSON v of
    Success fs | all bitHeld fs -> Just (toJSON (map (take 5) fs))
    _ -> Nothing
  bitHeld f = case f of
    [u, kind, _, _, _, bit] -> bit == judgedBit (M.findWithDefault (-1) u langOf) kind
    _ -> False
  langOf = M.fromList [(u, l) | Object o <- [r], Just us <- [field o "units"], Success rows' <- [fromJSON us], u : l : _ <- rows' :: [[Integer]]]

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

blank :: Value
blank = tabledRequest "7.0.0" "flow.request" [("units", []), ("stmts", []), ("vars", []), ("uses", [])]

-- | A generated request: its findings, six columns each, and its
-- dynamic unit count against the reference's.
agrees :: [Prog] -> Maybe String
agrees ps
  | got == want = Nothing
  | otherwise = Just (take 300 (show got <> " /= " <> show want))
 where
  got = case fieldsOf respond (request ps) ["findings", "counts"] of
    Just [Just fs, Just (Object counts)] -> Just (fs, field counts "dynamicUnits")
    _ -> Nothing
  want = Just (toJSON (referenceFindings ps), Just (toJSON (dynamicUnits ps)))

shapes :: [String]
shapes =
  [ "a forward goto", "a backward goto", "a goto out of a try through its finally"
  , "a break out of a switch to a loop", "a break to a switch a loop encloses", "a break to an outer loop", "a continue to an outer loop"
  , "a try inside a try", "a return through two finallys", "a catchless try whose finally reads a body store", "a dynamic unit beside a judged one"
  , "an unreachable run", "a dead store", "an unused local", "an unused parameter", "a judged finding", "an unjudged finding"
  ]
    <> ["language " <> show c | c <- langCodes]

-- | The shapes a generated request reaches: read off its trees (where
-- each jump lands, what encloses it) and off the reference's rows.
reached :: [Prog] -> S.Set String
reached ps =
  S.fromList $
    concatMap unitShapes ps
      <> ["a dynamic unit beside a judged one" | dynamicUnits ps > 0, dynamicUnits ps < length ps]
      <> ["language " <> show (pLang p) | p <- ps]
      <> concat [[kindName k, if bit == 1 then "a judged finding" else "an unjudged finding"] | [_, k, _, _, _, bit] <- referenceFindings ps]
 where
  kindName k = ["an unreachable run", "a dead store", "an unused local", "an unused parameter"] !! fromInteger k

-- | One unit's jump and nesting shapes, each statement read with its
-- ancestors innermost first.
unitShapes :: Prog -> [String]
unitShapes p = concatMap one (ancestry [] (pRoots p))
 where
  one (n, up) = case nKind n of
    k | k == kindGoto -> [if nAux n > nSeq n then "a forward goto" else "a backward goto"] <> ["a goto out of a try through its finally" | any (leaves n) (fenced n up)]
    k | k == kindBreak -> jumped n up "a break out of a switch to a loop" "a break to an outer loop" <> ["a break to a switch a loop encloses" | any ((== kindLoop) . nKind) (above (nAux n) up)]
    k | k == kindContinue -> jumped n up "" "a continue to an outer loop"
    k | k == kindTry -> ["a try inside a try" | any ((== kindTry) . nKind) up] <> ["a catchless try whose finally reads a body store" | finallyReads n]
    k | k == kindReturn -> ["a return through two finallys" | length (fenced n up) >= 2]
    _ -> []
  -- no catch, and a finally reading a variable the body writes
  finallyReads t =
    let (fins, others) = span' t
        accs ns m = [v | x <- concatMap subtree ns, (v, mode) <- nAcc x, m mode]
     in not (any ((== kindCatch) . nKind) (nKids t)) && any (`elem` accs others (/= modeRead)) (accs fins (/= modeWrite))
  span' t = (filter ((== kindFinally) . nKind) (nKids t), filter ((/= kindFinally) . nKind) (nKids t))
  -- the trys with a finally whose body or catch encloses n
  fenced n up = [t | (inner, t) <- zip (n : up) up, nKind t == kindTry, nKind inner /= kindFinally, any ((== kindFinally) . nKind) (nKids t)]
  leaves n t = nAux n `notElem` map nSeq (subtree t)
  -- what lies between a jump and its target, and above the target
  between n up = takeWhile ((/= nAux n) . nSeq) up
  above s up = drop 1 (dropWhile ((/= s) . nSeq) up)
  jumped n up overSwitch overLoop =
    let target = take 1 (dropWhile ((/= nAux n) . nSeq) up)
        loopTarget = any ((== kindLoop) . nKind) target
     in [overSwitch | loopTarget, overSwitch /= "", any ((== kindSwitch) . nKind) (between n up)]
          <> [overLoop | loopTarget, any ((== kindLoop) . nKind) (between n up)]

ancestry :: [Node] -> [Node] -> [(Node, [Node])]
ancestry up = concatMap (\n -> (n, up) : ancestry (n : up) (nKids n))

subtree :: Node -> [Node]
subtree n = n : concatMap subtree (nKids n)

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
    && testBit (2 ^ flagDynamic :: Int) flagDynamic

emptyRequest :: Bool
emptyRequest =
  fieldsOf respond blank ["findings", "degraded", "reason"] == Just [rows [], Just (Bool False), Nothing]
    && maybe False (const True) (replyObjWith respond blank)
