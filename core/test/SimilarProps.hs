-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The similar family's battery (plan v2.29 step 5): the same-role
-- conjunction against its truth table, the order as exact rationals
-- (a denominator per row, ties by request index, a difference no
-- double could see), the roles echoed in request order, an empty
-- request, refusal by name for every contract clause of both tables,
-- and the degraded posture (empty order and roles). Scaffolding lives
-- in WireHarness; the probes read their reply fields through
-- `fieldsOf` in one projection each.
module SimilarProps (battery) where

import CE.Similar (respond)
import CE.Similar.Cost (isRole, roleMinCallee, roleMinName, roleMinNameShape, ratio, similarCap)
import Data.Aeson
import WireHarness (fieldsOf, refusedBy, rowsRequest, runChecks, setKey)

-- | The request every probe edits: the query bag and the candidate
-- rows under the anchor proto; `req` is the bagless form.
request :: [[Integer]] -> [[Integer]] -> Value
request q rows = setKey "query" (toJSON q) (rowsRequest "6.7.0" "similar.request" rows)

req :: [[Integer]] -> Value
req = request []

-- | A row from its two evidence hits, shape bit and score fraction.
row :: Integer -> Integer -> Integer -> Integer -> Integer -> [Integer]
row n c shape num den = [n, 0, c, 0, 0, 0, shape, num, den]

battery :: IO Bool
battery =
  runChecks
    [ ("the conjunction matches its truth table", truthTable)
    , ("candidates stand in exact rational order, ties by request index, roles in request order", ordering)
    , ("a rational difference below double precision still orders", exactness)
    , ("an empty request answers empty tables", emptyReq)
    , ("similar refusals name the offender", refusals)
    , ("an over-cap request degrades to empty tables", degradedFace)
    ]

-- | A shared name and callee convicts; two names with the shape equal
-- convict; one name alone, a shape alone, a callee alone do not.
truthTable :: Bool
truthTable =
  (roleMinName, roleMinCallee, roleMinNameShape) == (1, 1, 2)
    && map isRole [row 1 1 0 0 1, row 2 0 1 0 1, row 1 0 1 0 1, row 2 0 0 0 1, row 0 3 1 0 1, row 0 0 0 0 1]
      == [True, True, False, False, False, False]
    && ratio (row 0 0 0 3 2) == 3 / 2

ordering :: Bool
ordering =
  fieldsOf respond (request [[7, 768], [9, 256]] rows) ["order", "roles", "counts", "degraded"]
    == Just
      [ Just (toJSON [0, 2, 1, 3 :: Integer])
      , Just (toJSON [True, False, True, False])
      , Just (object ["rows" .= (4 :: Int), "queryTerms" .= (2 :: Int), "role" .= (2 :: Int)])
      , Just (Bool False)
      ]
 where
  -- 3/2 (role), 7/5, 3/2 again (role, tie → after row 0), 0/1
  rows = [row 1 1 0 3 2, row 1 0 0 7 5, row 2 0 1 3 2, row 0 0 0 0 1]

-- | 10^20+1 / 10^20 against 1/1: a double reads both as 1.0.
exactness :: Bool
exactness =
  fieldsOf respond (req [row 0 0 0 1 1, row 0 0 0 (10 ^ (20 :: Int) + 1) (10 ^ (20 :: Int))]) ["order"]
    == Just [Just (toJSON [1, 0 :: Integer])]

emptyReq :: Bool
emptyReq =
  fieldsOf respond (req []) ["order", "roles", "degraded"]
    == Just [Just (toJSON ([] :: [Integer])), Just (toJSON ([] :: [Bool])), Just (Bool False)]

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[1, 0, 1]]) "row 0: malformed row"
    , refusedBy respond (req [row (-1) 1 0 1 1]) "row 0: negative hit"
    , refusedBy respond (req [row 1 1 2 1 1]) "row 0: shapeEqual not a boolean"
    , refusedBy respond (req [row 1 1 0 (-1) 1]) "row 0: negative score"
    , refusedBy respond (req [row 1 1 0 1 0]) "row 0: non-positive denominator"
    , refusedBy respond (request [[7]] []) "query 0: malformed query term"
    , refusedBy respond (request [[-1, 256]] []) "query 0: negative term hash"
    , refusedBy respond (request [[7, 0]] []) "query 0: non-positive weight"
    , refusedBy respond (request [[9, 256], [7, 256]] []) "query 1: not strictly ascending"
    , refusedBy respond (request [[7, 256], [7, 256]] []) "query 1: not strictly ascending"
    ]

degradedFace :: Bool
degradedFace =
  fieldsOf respond (req big) ["degraded", "order", "roles", "reason"]
    == Just [Just (Bool True), Just (toJSON ([] :: [Integer])), Just (toJSON ([] :: [Bool])), Just "similar_too_large"]
 where
  big = replicate (fromInteger similarCap + 1) (row 1 1 0 1 1)
