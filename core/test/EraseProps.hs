-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The erase family's battery (M9 batch 3): the safety predicate
-- against an enumerated truth table (exact tuples — a wrong REASON
-- is as red as a wrong verdict), the mixed request through the REAL
-- respond with count conservation, refusal-by-name for every
-- malformed fact, the knobless stance, the degraded-fails posture
-- with an EMPTY verdict table (a refused plan licenses nothing), and
-- since 7.2.0 (plan v2.30 step 7b) the target closure — the rule
-- that says which judged row stands for a target — against its own
-- hand-written table, through the real respond, with its refusals;
-- and since 9.1.0 (lane v235_licence) the rule that no whole-file row
-- is ever eraseable, over every legal fact combination.
-- Scaffolding lives in WireHarness.
module EraseProps (battery) where

import CE.Erase (respond)
import CE.Erase.Cost (eraseRowCap, judgeLinked, judgeRow, keptRows, reasonNames)
import Data.Aeson
import WireHarness (degradedFace, field, refusedBy, replyObjWith, rowsRequest, runLegs, setKey)

-- | The linked-section leg (release 1.10.0) rides beside the older legs.
battery :: IO Bool
battery = runLegs (names <> [linkedName]) (probes <> [linkedFace])
 where
  linkedName = "a linked section refuses as linked_section after the byte checks, and linked refusals name the offender"
  names =
    [ "the predicate matches the enumerated truth table, reasons included"
    , "a mixed request judges every row in order and conserves counts"
    , "erase refusals name the offender"
    , "the family is knobless: any knob row refuses by name"
    , "an over-cap erase request degrades to an EMPTY table that FAILS"
    , "the target closure keeps an eraseable row, else the advisory about the file"
    , "kept rides on every judged reply, a request without targets is refused by name, and target refusals name the offender"
    , "no t1_twin or dead_file row is eraseable under any legal fact combination"
    ]
  probes = [truthTable, mixed, refusals, knobless, degradedFails, closure, targetsFace, noFileLicence]

-- | Exact expected (eraseable, reason) per fact combination — the
-- independent second derivation is this literal table, written from
-- the contract (erase.md), not from the code. Facts and expectations
-- ride as two PARALLEL data lists: a per-case assertion line was a
-- T2 clone chain by this repo's own measure (the length conjunct
-- guards zipWith's silent truncation).
truthTable :: Bool
truthTable =
  length facts == length wants
    && and (zipWith (\r v -> judgeRow r == v) facts wants)
 where
  facts =
    -- class 0 is absent by construction (4.0.0): the row shape
    -- refuses it upstream, so no judgeRow case can be written for
    -- it -- `refusals` below is where its retirement is asserted.
    [ [1, 60, 60, 60, 1], [1, 59, 60, 60, 1]
    , [1, 60, 60, 61, 1], [1, 60, 60, 60, 0]
    , [2, 1, 1, 1, 0], [2, 0, 1, 1, 0], [2, 1, 0, 1, 0]
    , [2, 1, 1, 0, 0], [2, 1, 1, 1, 3]
    , -- class 2 across the dead domain (7.0.0): the public codes 2
      -- and 4 are refused as public_surface BEFORE the trust fact is
      -- weighed (the last row names the order), and the 6.x boolean 1
      -- is code 1; a private copy whose every fact holds is refused
      -- as reachable_by_name (9.1.0), never licensed
      [2, 1, 1, 3, 0], [2, 1, 1, 2, 0], [2, 1, 1, 4, 0], [2, 1, 1, 2, 3]
    , -- class 3 across the WHOLE dead domain x the trust fact
      -- (6.1.0). The public codes 2 and 4 are refused whatever their
      -- confidence, because RG10 is a bar and not a strength -- the
      -- last row names the order: unvouched AND public reports
      -- public; a private code answers language_unresolved at
      -- confidence 0 and reachable_by_name past it, the trust fact
      -- named first because it says the graph could not even show
      -- the file unreferenced (9.1.0)
      [3, 1, 0, 0, 0], [3, 1, 2, 0, 0], [3, 3, 2, 0, 0]
    , [3, 2, 2, 0, 0], [3, 4, 2, 0, 0], [3, 2, 0, 0, 0], [3, 3, 1, 0, 0]
    ]
  wants =
    [ (True, 0), (False, 2), (False, 2), (False, 3)
    , (False, 7), (False, 5), (False, 3), (False, 4), (False, 1)
    , (False, 7), (False, 6), (False, 6), (False, 6)
    , (False, 1), (False, 7), (False, 7)
    , (False, 6), (False, 6), (False, 6), (False, 7)
    ]

-- | An erase request over `rows`, its target table (required at 9.0.0)
-- one path per row in key order: a one-line span for a verbatim_doc
-- row, the whole file otherwise.
req :: [[Integer]] -> Value
req rows = setKey "targets" (toJSON (zipWith target [0 :: Integer ..] rows)) (bare rows)
 where
  target i r = if take 1 r == [1] then [i, 1, 1] else [i, 0, 0]

-- | The same request without its target table.
bare :: [[Integer]] -> Value
bare = rowsRequest "7.0.0" "erase.request"

mixed :: Bool
mixed = case replyObjWith respond (req rows) of
  Just o ->
    field o "rows" == Just (toJSON [[0, 7 :: Integer], [0, 1], [1, 0], [0, 3]])
      && field o "degraded" == Just (Bool False)
      && field o "fail" == Just (Bool False)
      && (counts o "eraseable" == Just 1)
      && (counts o "advisory" == Just 3)
      && (counts o "rows" == Just 4)
  Nothing -> False
 where
  -- the two dead_file rows ride class 3 since 4.0.0 retired class
  -- 0; confidence 2 (vouched) refuses as reachable_by_name since
  -- 9.1.0, confidence 0 (unvouched) as language_unresolved
  rows =
    [ [3, 3, 2, 0, 0]
    , [3, 1, 0, 0, 0]
    , [1, 80, 80, 80, 1]
    , [2, 1, 0, 1, 0]
    ]
  counts o k = do
    Object c <- field o "counts"
    n <- field c k
    case n of Number v -> Just v; _ -> Nothing

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[4, 0, 0, 0, 0]]) "row 0: unknown class"
    , -- K2: the retired class is refused BY NAME, not folded into
      -- "unknown class" -- a client still sending it learns which
      -- road replaced it (2.32.0's class 3), and no class-0 row can
      -- ever be judged again
      refusedBy respond (req [[0, 1, 0, 0, 0]]) "row 0: retired class 0"
    , refusedBy respond (req [[1, 60, 60, 60, 2]]) "row 0: bytesEqual not a boolean"
    , refusedBy respond (req [[3, 1, 0, 0]]) "row 0: malformed row"
    , -- the class-3 (verdict, confidence) and class-2 (covered, equal,
      -- dead code) shape probes, one row table per class
      and [refusedBy respond (req [[3, v, c, 0, 0]]) ("row 0: " <> why) | (v, c, why) <- class3]
    , and [refusedBy respond (req [[2, w, x, y, 0]]) ("row 0: " <> why) | (w, x, y, why) <- class2]
    ]
  where
    class3 =
      [ (0, 0, "dead verdict outside 1..4")
      , (5, 0, "dead verdict outside 1..4")
      , (1, 3, "confidence outside 0..2")
      , (1, -1, "negative fact")
      ]
    class2 =
      [ (2, 1, 1, "coverage/equality not booleans")
      , (1, 1, 5, "dead verdict outside 0..4")
      ]

knobless :: Bool
knobless =
  refusedBy
    respond
    (setKey "knobs" (toJSON [[0 :: Integer, 1]]) (req [[3, 1, 0, 0, 0]]))
    "knob 0: erase/1 declares no knob codes"

degradedFails :: Bool
degradedFails = degradedFace respond (req big) "rows" "erase_too_large"
 where
  big = replicate (fromInteger eraseRowCap + 1) [3, 1, 0, 0, 0]

-- | The closure's own truth table (erase.md §closure), written from
-- the contract: (fact rows, targets, the kept bits) — a private dead
-- file and its private twin on one whole-file target, both advisory
-- since 9.1.0, the dead file's row standing and the span row on the
-- same path staying (no whole-file row owns a path any more); the
-- dead file's row over a twin whose bytes differ, over a twin whose
-- file is unvouched, over a public twin; a span on a path whose
-- whole-file row is advisory stays; two rows of one class on one
-- target break to the earliest; an eraseable span over an advisory
-- span on one target; two paths close independently. Fact rows, targets
-- and kept bits ride as three PARALLEL lists (the truthTable
-- discipline: a tuple row per case was a T2 clone chain by this
-- repo's own measure; the length conjuncts guard zipWith3's silent
-- truncation).
closure :: Bool
closure =
  length facts == length targets
    && length targets == length wants
    && and (zipWith3 (\r t w -> keptRows t r (map judgeRow r) == w) facts targets wants)
 where
  facts =
    [ [[3, 1, 2, 0, 0], [2, 1, 1, 1, 0], [1, 80, 80, 80, 1]]
    , [[3, 1, 2, 0, 0], [2, 1, 0, 1, 0]]
    , [[3, 1, 0, 0, 0], [2, 1, 1, 1, 0]]
    , [[3, 2, 2, 0, 0], [2, 1, 1, 2, 0]]
    , [[3, 2, 2, 0, 0], [1, 80, 80, 80, 1]]
    , [[1, 80, 80, 80, 1], [1, 90, 90, 90, 1]]
    , [[3, 1, 2, 0, 0], [1, 80, 80, 80, 1]]
    , [[1, 59, 60, 60, 1], [1, 80, 80, 80, 1]]
    ]
  targets =
    [ [[0, 0, 0], [0, 0, 0], [0, 4, 9]]
    , [[0, 0, 0], [0, 0, 0]]
    , [[0, 0, 0], [0, 0, 0]]
    , [[0, 0, 0], [0, 0, 0]]
    , [[0, 0, 0], [0, 3, 5]]
    , [[2, 3, 5], [2, 3, 5]]
    , [[0, 0, 0], [1, 3, 5]]
    , [[1, 3, 5], [1, 3, 5]]
    ]
  wants =
    [ [True, False, True]
    , [True, False]
    , [True, False]
    , [True, False]
    , [True, True]
    , [True, False]
    , [True, True]
    , [False, True]
    ]

-- | The wire face of the closure: `kept` rides one bit per row on every
-- judged reply (the counts stay the whole table's), never on a degraded
-- one; a request without the target table is refused by name (required
-- at 9.0.0); and every target refusal names its row and reason.
targetsFace :: Bool
targetsFace =
  and
    [ fmap (`field` "kept") (replyObjWith respond withTargets) == Just (Just (toJSON [1, 0, 1 :: Integer]))
    , refusedBy respond (bare rows) "targets: required at 9.0.0"
    , fmap (`field` "kept") (replyObjWith respond (targeted big (replicate (length big) [0, 0, 0]))) == Just Nothing
    , length refusedRows == length refusedTargets && length refusedTargets == length whys
    , and (zipWith3 (\r t why -> refusedBy respond (targeted r t) why) refusedRows refusedTargets whys)
    ]
 where
  rows = [[3, 1, 2, 0, 0], [2, 1, 1, 1, 0], [1, 80, 80, 80, 1]]
  withTargets = targeted rows [[0, 0, 0], [0, 0, 0], [0, 4, 9]]
  targeted r t = setKey "targets" (toJSON (t :: [[Integer]])) (req r)
  big = replicate (fromInteger eraseRowCap + 1) [3, 1, 0, 0, 0]
  -- the refusal probes as three PARALLEL lists — fact rows, targets,
  -- the named reason — read in step (the truthTable discipline)
  refusedRows =
    [ [[3, 1, 2, 0, 0]], [[3, 1, 2, 0, 0]], [[3, 1, 2, 0, 0]], [[3, 1, 2, 0, 0]]
    , [[1, 80, 80, 80, 1]], [[1, 80, 80, 80, 1]], [[2, 1, 1, 1, 0]], [[3, 1, 2, 0, 0], [3, 1, 2, 0, 0]]
    ]
  refusedTargets =
    [ [], [[0, 0]], [[-1, 0, 0]], [[0, 0, 4]]
    , [[0, 9, 4]], [[0, 0, 0]], [[0, 4, 9]], [[1, 0, 0], [0, 0, 0]]
    ]
  whys =
    [ "targets: 0 rows for 1 fact rows", "target 0: malformed row", "target 0: negative field", "target 0: half-open span"
    , "target 0: span end before start", "target 0: verbatim_doc row names a whole file", "target 0: whole-file class names a span", "target 1: out of key order"
    ]

-- | The 9.1.0 rule as a sweep, not a sample: every legal fact row of
-- the two whole-file classes (each fact over its whole contract range)
-- judges advisory with a reason inside the frozen table, and the rows
-- that pass every other bar name reachable_by_name, so the refusal is
-- the rule and not an accident of some earlier bar.
noFileLicence :: Bool
noFileLicence =
  not (any (fst . judgeRow) (twins <> deads))
    && all ((< toInteger (length reasonNames)) . snd . judgeRow) (twins <> deads)
    && map judgeRow [[2, 1, 1, 1, 0], [2, 1, 1, 3, 0], [3, 1, 1, 0, 0], [3, 3, 2, 0, 0]] == replicate 4 (False, 7)
 where
  twins = [[2, w, x, y, z] | w <- [0, 1], x <- [0, 1], y <- [0 .. 4], z <- [0 .. 3]]
  deads = [[3, w, x, y, z] | w <- [1 .. 4], x <- [0 .. 2], y <- [0 .. 2], z <- [0 .. 2]]

-- | The linked table (10.0.0, release 1.10.0): a doc row that would
-- erase is refused as linked_section (8) when its section receives a
-- link; a row the byte checks refuse keeps its reason; through the
-- real respond the table moves exactly its row; and each refusal names
-- the row (out of range, not a documentation row, links below 1, rows
-- not ascending, malformed).
linkedFace :: Bool
linkedFace =
  map (uncurry judgeLinked) [(1, ok), (0, ok), (2, [1, 59, 60, 60, 1]), (1, [1, 60, 60, 60, 0])]
    == [(False, 8), (True, 0), (False, 2), (False, 3)]
    && fmap (`field` "rows") (replyObjWith respond (linked [[1, 1]])) == Just (Just (toJSON [[1, 0], [0, 8 :: Integer], [0, 7]]))
    && and (zipWith (\l why -> refusedBy respond (linked l) why) bad whys)
 where
  ok = [1, 60, 60, 60, 1]
  linked l = setKey "linked" (toJSON (l :: [[Integer]])) (req [ok, ok, [3, 1, 2, 0, 0]])
  bad = [[[3, 1]], [[2, 1]], [[1, 0]], [[1, 1], [0, 1]], [[1]]]
  whys = ["linked 0: row out of range", "linked 0: not a documentation row", "linked 0: links below 1", "linked 1: rows not ascending", "linked 0: malformed row"]
