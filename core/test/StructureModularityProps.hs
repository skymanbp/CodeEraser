-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | S7 modularity's battery (O54, wire 7.1.0) — split out of
-- StructureProps at the E01 300-line wall, the CE.Structure.Knobs
-- precedent applied to the test half. The axis reads TWO tables at
-- once and a boundary law holds them to one graph, so it gets its
-- own fixture rather than bending the family's: root (3 subdirs, no
-- files) over dir 1 and dir 2 (3 files each, an internal 3-cycle)
-- and dir 3 (3 files, NO internal reference, three edges out to
-- dir 1 and three in from dir 2 — the grab bag). Every digit by
-- hand, with m = 3 + 3 + 0 + 6 = 12 edges in the tree:
--
--   dir 1: e=3, o=3, i=6, mu=6  -> (3*12 − 3*6)/(6*6) = 18/36 = 500‰
--   dir 2: e=3, o=6, i=3, mu=6  -> 500‰
--   dir 3: e=0, o=3, i=3, mu=6  -> (0 − 9)/36 < 0            FLAGGED
--   dir 0: mu=0, under the mass floor, not judged at all
--
-- So axis 7 counts ONE directory at N=4 dirs: charge
-- floor(1000·1/5) = 200. S2 flags dir 3 as well (outs 6 > ins 0 at
-- traffic 6) and S4 flags the root (no config bit), so the fold is
-- 200 + 200 + 200 over SIX judged axes and the score is
-- 1000 − 6000 div 60 = 900, findings [[3,2],[0,4],[3,7]].
module StructureModularityProps (battery) where

import CE.Structure (respond)
import CE.Structure.Cost (structNodeCap)
import Data.Aeson
import WireHarness (degradedFace, field, refusedBy, replyObjWith, runChecks, setKey, tabledRequest)

-- | Every digit-level probe is a row of 'probes', named by its own
-- `why` — a ladder of near-identical assertion lines is this repo's
-- own T2 clone by measure (the EraseProps truthTable lesson), and
-- what deserves the reader's eye here is the arithmetic, not the
-- scaffolding around it.
battery :: IO Bool
battery =
  runChecks
    ( [(why, at req key == Just want) | (why, req, key, want) <- probes]
        <> [ ("modularity refusals name the offender", refusals)
           , ("dir-edge rows are priced against the structure node cap", edgesPriced)
           ]
    )

-- | One reply field through the REAL respond.
at :: Value -> String -> Maybe Value
at r key = replyObjWith respond r >>= \o -> field o key

-- | (why, request, reply key, the value that key must carry): the
-- hand-computed grab bag, then both knobs as levers moving in
-- OPPOSITE directions (F16 — raise the cohesion floor and the count
-- can only grow, raise the mass floor and it can only shrink), then
-- absence-vs-zero on the table itself.
probes :: [(String, Value, String, Value)]
probes =
  [ ("the grab bag is flagged on axis 7 and the two modules are not", modReq, "axes", charges)
  , ("the six-axis fold lands on the hand-computed score", modReq, "score", num 900)
  , ("the finding rows name the grab bag twice, S2 and S7", modReq, "findings", found)
  , ("knob 19: a floor past 500 permille takes the modules too", knobbed [[19, 600]], "score", num 862)
  , ("knob 20: a mass floor past the fixture's mu = 6 empties the axis", knobbed [[20, 7]], "score", num 934)
  , ("knob 20: the emptied axis is present and clean, not absent", knobbed [[20, 7]], "axes", emptied)
  , ("an empty dir-edge table is judged clean at charge zero", edgesOn bare (toJSON noRows), "axes", cleanly)
  , ("no dir-edge table at all leaves five axis rows", edgesOn bare Null, "axes", fiveRows)
  ]
 where
  charges = axes [[0, 0], [1, 0], [2, 200], [3, 0], [4, 200], [7, 200]]
  emptied = axes [[0, 0], [1, 0], [2, 200], [3, 0], [4, 200], [7, 0]]
  cleanly = axes [[0, 0], [1, 0], [2, 0], [3, 0], [4, 500], [7, 0]]
  fiveRows = axes [[0, 0], [1, 0], [2, 0], [3, 0], [4, 500]]
  found = axes [[3, 2], [0, 4], [3, 7]]

num :: Integer -> Value
num = toJSON

axes :: [[Integer]] -> Value
axes = toJSON

noRows :: [[Integer]]
noRows = []

-- | Root over three sibling directories of three files each.
grabNodes :: [[Integer]]
grabNodes = [[0, 0, 0, 3, 0], [1, 0, 1, 0, 3], [2, 0, 1, 0, 3], [3, 0, 1, 0, 3]]

-- | The intra mass and the crossing endpoints, per directory: dirs 1
-- and 2 hold `inside` 2 per file over 3 files = 6 = TWICE their three
-- internal edges (which is the whole reason the intra mass need not
-- ride), dir 3 holds none and owns 6 crossing endpoints.
grabRefs :: [[Integer]]
grabRefs = [[1, 2, 1, 3], [2, 2, 1, 3], [3, 0, 2, 3]]

-- | The crossing edges alone: three from dir 2 into dir 3, three out
-- of dir 3 into dir 1 — the boundary law ties this table to the one
-- above, six endpoints on dir 3 either way you count them.
grabEdges :: [[Integer]]
grabEdges = [[2, 3, 3], [3, 1, 3]]

modReq :: Value
modReq =
  tabledRequest
    "7.0.0"
    "structure.request"
    [("nodes", grabNodes), ("fileRefs", grabRefs), ("dirEdges", grabEdges)]

knobbed :: [[Integer]] -> Value
knobbed rows = setKey "knobs" (toJSON rows) modReq

edgesOn :: Value -> Value -> Value
edgesOn = flip (setKey "dirEdges")

-- | The same tree stripped of every reference — one root directory,
-- no files, no fileRefs — so the only thing left to vary is whether
-- the dir-edge table rode at all (aeson reads an explicit null the
-- same road an absent key takes).
bare :: Value
bare =
  setKey "fileRefs" (toJSON noRows) (setKey "nodes" (toJSON [[0, 0, 0, 0, 0 :: Integer]]) modReq)

-- | Every arm of the row spec, then the cross-table law — which is
-- the whole licence for deriving the intra mass instead of shipping
-- it: dir 1 owns three crossing endpoints in fileRefs and this table
-- accounts for two, so the pair is refused BY NAME rather than judged
-- on whichever half the core happens to trust.
refusals :: Bool
refusals = all named refusalRows
 where
  named (rows, want) = refusedBy respond (edgesOn modReq (toJSON rows)) want

refusalRows :: [([[Integer]], String)]
refusalRows =
  [ ([[1, 1, 1]], "self edge (from == to)")
  , ([[1, 9, 1]], "to dir out of range")
  , ([[1, 2, 0]], "count below 1")
  , ([[9, 1, 1]], "dir out of range")
  , ([[3, 1, 3], [2, 3, 3]], "not strictly ascending")
  , ([[2, 3, 3], [3, 1, 2]], "crossing endpoints in fileRefs but 2")
  ]

-- | C15: the dir-edge rows count against structNodeCap alongside the
-- node rows and the seam tables — a declared cap that misses a
-- request dimension walks that dimension uncapped.
edgesPriced :: Bool
edgesPriced = degradedFace respond overCapEdges "findings" "structure_too_large"
 where
  overCapEdges = edgesOn bare (toJSON [[0, 1, 1 :: Integer] | _ <- [0 .. structNodeCap]])
