-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The tombstone family against its second implementation (plan
-- v2.32 step 7): ReferenceTombstone's truth table and the shipped
-- respond must answer the same sites, counts, condition, knob echo
-- and degradation — over EVERY request of up to two rows drawn from
-- the 48-row alphabet under every budget, every three- and four-row
-- request over the six class representatives, the contract's
-- refusals, and both sides of the row cap.
module TombstoneEquivProps (battery) where

import CE.Tombstone (respond)
import CE.Tombstone.Cost (tombstoneRowCap)
import ReferenceContract (answers, knobbedRequest, pairTable)
import ReferenceTombstone (budgets, expected, rowAlphabet, smallAlphabet)
import WireHarness (runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "every request of up to two alphabet rows, under every budget, agrees with the truth table"
    , "every three- and four-row request over the class representatives agrees"
    , "the contract's refusals agree with the reference's named predicates"
    , "both sides of the row cap agree, rows and knobs counted together"
    ]
    [ all agrees [(rs, ks) | rs <- upTo 2 rowAlphabet, ks <- knobRows]
    , all agrees [(rs, ks) | n <- [3, 4], rs <- sequenceOf n smallAlphabet, ks <- knobRows]
    , all agrees refusals
    , all agrees capped
    ]

-- | The knob table each budget spells: none, or one row of code 0.
knobRows :: [[[Integer]]]
knobRows = [[[0, v] | Just v <- [b]] | b <- budgets]

upTo :: Int -> [a] -> [[a]]
upTo n xs = concat [sequenceOf k xs | k <- [0 .. n]]

sequenceOf :: Int -> [a] -> [[a]]
sequenceOf k xs = sequence (replicate k xs)

-- | The two sides agree: the same fields, or the same refusal.
agrees :: ([[Integer]], [[Integer]]) -> Bool
agrees (rs, ks) =
  answers respond ["sites", "counts", "over", "knobs", "degraded", "reason"] (knobbedRequest "tombstone.request" rs ks) (expected rs ks)

-- | A foreign kind, a negative count, a short row, a foreign knob
-- code, a negative budget, a duplicated knob, and an offending row
-- standing before an offending knob.
refusals :: [([[Integer]], [[Integer]])]
refusals =
  pairTable
    [ "[[3,0,1]] ; []"
    , "[[0,0,1],[2,-1,1]] ; []"
    , "[[0,1]] ; [[0,2]]"
    , "[[2,1,1]] ; [[1,5]]"
    , "[[2,1,1]] ; [[0,-1]]"
    , "[[2,1,1]] ; [[0,1],[0,2]]"
    , "[[1,0,-4]] ; [[7,7]]"
    ]

-- | The cap on both sides: exactly at the cap (judged) and one past
-- it through the knob row (degraded, condition unevaluated).
capped :: [([[Integer]], [[Integer]])]
capped =
  [ (replicate (fromInteger tombstoneRowCap) [2, 1, 1], [])
  , (replicate (fromInteger tombstoneRowCap) [2, 1, 1], [[0, 3]])
  ]
