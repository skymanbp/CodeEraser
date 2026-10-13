-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan/1's recursion increment (6.5.0, plan v2.23 step 4): the
-- cycle membership CE.Scan.Cycles judges, checked against an
-- independent oracle over EVERY four-vertex graph, plus the call
-- table's refusals and the key's presence rule. Its own module
-- beside ScanProps for the reason AdvisoryProps sits beside
-- GraphWireProps: the increment is a class of its own, and both
-- ScanProps and Spec.hs stand at their size line.
--
-- The oracle is ReferenceGraph's naive reachability fixpoint, which
-- knows nothing of Data.Graph — so the exhaustive leg compares two
-- implementations sharing no code, not one implementation with
-- itself.
module ScanCyclesProps (battery) where

import CE.Scan (respond)
import CE.Scan.Cycles (withCycles)
import Data.Aeson (Value, toJSON)
import Data.Bits (setBit)
import qualified Data.IntSet as IS
import qualified Data.Set as S
import ReferenceGraph (arcsOf, reachB)
import WireHarness (field, refusedBy, replyObjWith, runChecks, scanRequest, setKey)

battery :: IO Bool
battery = runChecks [("v2.23: " <> name, ok) | (name, ok) <- legs]

legs :: [(String, Bool)]
legs =
  [ ("every four-vertex graph agrees with an independent oracle", exhaustive)
  , ("a one-way call is not a cycle", oneWay)
  , ("only cognitive rows move, and only by one", onlyCognitive)
  , ("an arc added never un-charges a unit", monotone)
  , ("no table, no change and no key", legacy)
  , ("the key rides whenever the arcs did, an empty answer included", keyRides)
  , ("the call table's four refusals name the offender", refusals)
  ]

-- | v sits on a cycle exactly when it is reachable from itself in
-- one or more steps — the whole rule, spelled without Data.Graph.
loopedB :: [(Int, Int)] -> Int -> Bool
loopedB arcs v =
  IS.member v (IS.fromList [d | (a, d) <- arcs, IS.member a (reachB arcs [v])])

edges :: [(Int, Int)] -> [[Integer]]
edges arcs = [[toInteger a, toInteger b] | (a, b) <- arcs]

-- | Four rows, all cognitive, all measured at 7.
base :: [[Integer]]
base = replicate 4 [4, 7]

raised :: [[Integer]] -> S.Set Integer
raised arcs = S.fromList [i | [i, _] <- snd (withCycles (Just arcs) base)]

exhaustive :: Bool
exhaustive = null (take 1 [code | code <- [0 .. 65535 :: Int], not (agrees code)])
 where
  agrees code =
    let arcs = arcsOf 4 code
        (rows, moved) = withCycles (Just (edges arcs)) base
        want = [i | i <- [0 .. 3], loopedB arcs i]
     in rows == [[4, if i `elem` want then 8 else 7] | i <- [0 .. 3]]
          && moved == [[toInteger i, 8] | i <- want]

oneWay :: Bool
oneWay = snd (withCycles (Just [[0, 1]]) base) == []

-- | The increment reaches the cognitive rows and nothing else: one
-- row per code, one self-arc on code 4's row, and only that row
-- moves — by exactly one.
onlyCognitive :: Bool
onlyCognitive = rows == want && moved == [[4, 10]]
 where
  mixed = [[c, 9] | c <- [0 .. 6]]
  (rows, moved) = withCycles (Just [[4, 4]]) mixed
  want = [[c, if c == 4 then 10 else 9] | c <- [0 .. 6]]

monotone :: Bool
monotone =
  and
    [ raised (edges (arcsOf 4 code)) `S.isSubsetOf` raised (edges (arcsOf 4 (setBit code bit)))
    | code <- [0, 7, 33, 291, 4096, 32768]
    , bit <- [0 .. 15]
    ]

legacy :: Bool
legacy =
  withCycles Nothing base == (base, [])
    && replyObjWith respond (ask (map last base) Nothing) /= Nothing
    && bumpedOf Nothing == Nothing

-- | The reply's key rides exactly when the arcs did — an empty
-- raised table still answers, because "no cycles here" and "never
-- asked" must not read alike.
keyRides :: Bool
keyRides =
  bumpedOf (Just (edges [(0, 1)])) == Just (toJSON ([] :: [[Integer]]))
    && bumpedOf (Just (edges [(0, 0)])) == Just (toJSON [[1 :: Integer, 8]])

-- | Each case brings the rows its offence needs: the shape battery
-- runs before the order one, so a table whose ENDPOINTS are already
-- wrong can never demonstrate the ordering refusal.
refusals :: Bool
refusals =
  and
    [ refusedBy respond (setKey "callEdges" (toJSON (arcs :: [[Integer]])) (scanRequest rows)) why
    | (rows, arcs, why) <-
        [ (mixedRows, [[1, 9]], "endpoint outside the rows")
        , (mixedRows, [[1, 3]], "endpoint is not a cognitive row")
        , (concat (replicate 2 triple), [[4, 4], [1, 1]], "not strictly ascending")
        , (mixedRows, [[1]], "malformed row (need [from,to])")
        ]
    ]

-- | Row 1 cognitive (in its unit's triple), row 3 a file row: an
-- endpoint has both a seat that exists and a seat that is the wrong
-- metric to land on.
mixedRows :: [[Integer]]
mixedRows = triple <> [[0, 3]]

triple :: [[Integer]]
triple = [[3, 0], [4, 0], [5, 0]]

-- | Units on the wire (the events are required since 9.0.0): each
-- unit's [3,4,5] triple, its cognitive value as that many labelled
-- jumps (a flat +1 that moves neither cyclomatic nor nesting,
-- CE.Scan.Complexity bit 3), the arcs re-pointed at the cognitive
-- rows.
ask :: [Integer] -> Maybe [[Integer]] -> Value
ask cocs arcs = maybe id (setKey "callEdges" . toJSON . map (map seat)) arcs (setKey "events" (toJSON evs) (scanRequest rows))
 where
  rows = concat (replicate (length cocs) triple)
  seat i = 3 * i + 1
  evs = [[seat u, s, -1, 0, 8, 0] | (u, v) <- zip [0 ..] cocs, s <- [0 .. v - 1]]

bumpedOf :: Maybe [[Integer]] -> Maybe Value
bumpedOf arcs = replyObjWith respond (ask (map last base) arcs) >>= \o -> field o "cocBumped"
