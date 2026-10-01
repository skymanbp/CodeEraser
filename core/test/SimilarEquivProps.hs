-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The similar family against its second implementation (plan v2.32
-- step 7): two hundred seeded requests through the shipped respond and
-- through ReferenceSimilar's rank-by-comparison must agree on the
-- order, the role bits, the counts and the degradation; the contract's
-- refusals and both sides of the cap (query terms and rows priced
-- together) agree too.
module SimilarEquivProps (battery) where

import CE.Similar (respond)
import CE.Similar.Cost (similarCap)
import Data.Aeson (Value, object, (.=))
import ReferenceContract (answers, pairTable)
import ReferenceSimilar (expected, requests)
import WireHarness (runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "two hundred seeded requests agree on order, roles, counts and degradation"
    , "the seeded requests are not vacuous: ties, zero scores and both role arms occur"
    , "the contract's refusals agree with the reference's predicates"
    , "both sides of the cap agree, query terms and rows counted together"
    ]
    [ all agrees requests
    , nonVacuous
    , all agrees refusals
    , all agrees capped
    ]

request :: [[Integer]] -> [[Integer]] -> Value
request query rows =
  object ["proto" .= ("7.0.0" :: String), "type" .= ("similar.request" :: String), "id" .= (1 :: Int), "query" .= query, "rows" .= rows]

agrees :: ([[Integer]], [[Integer]]) -> Bool
agrees (query, rows) =
  answers respond ["order", "roles", "counts", "degraded", "reason"] (request query rows) (expected query rows)

-- | The generator reaches what the order and the role must decide.
nonVacuous :: Bool
nonVacuous =
  any (tie . snd) requests
    && any (any ((== [0]) . take 1 . drop 7) . snd) requests
    && any (any (\r -> take 3 r == [1, 0, 1] && take 1 (drop 6 r) == [0]) . snd) requests
    && any (any (\r -> take 1 r == [2] && take 1 (drop 6 r) == [1] && take 1 (drop 2 r) == [0]) . snd) requests
 where
  tie rs = or [a * d == c * b && a /= c | (a : b : _, c : d : _) <- pairs rs]
  pairs rs = [(drop 7 x, drop 7 y) | (i, x) <- zip [0 :: Int ..] rs, (j, y) <- zip [0 :: Int ..] rs, i < j]

refusals :: [([[Integer]], [[Integer]])]
refusals =
  pairTable
    [ "[[-1,1]] ; []"
    , "[[3,0]] ; []"
    , "[[4,1],[4,2]] ; []"
    , "[[1]] ; []"
    , "[] ; [[0,0,0,0,0,0,0,1]]"
    , "[] ; [[1,0,1,0,0,0,0,1,1],[0,-1,0,0,0,0,0,1,1]]"
    , "[] ; [[1,0,1,0,0,0,2,1,1]]"
    , "[] ; [[1,0,1,0,0,0,1,-1,1]]"
    , "[] ; [[1,0,1,0,0,0,1,1,0]]"
    , "[[2,1],[1,1]] ; [[1,0,1,0,0,0,1,1,0]]"
    ]

-- | At the cap exactly (judged) and one past it (degraded): the query
-- bag carries the mass so the judged side ranks a single candidate.
capped :: [([[Integer]], [[Integer]])]
capped =
  [ ([[h, 1] | h <- [0 .. similarCap - 2]], [[2, 0, 0, 0, 0, 0, 1, 3, 4]])
  , ([[h, 1] | h <- [0 .. similarCap - 1]], [[2, 0, 0, 0, 0, 0, 1, 3, 4]])
  ]
