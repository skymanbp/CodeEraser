-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The tombstone family's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 14's conjunction written as a TRUTH
-- TABLE over the row's three facts, the budget as a comprehension,
-- the contract as a list of named predicates. It reads only the
-- family's constants (CE.Tombstone.Cost) — never isSite, overBudget
-- or the CE.Wire cascade — and answers the reply fields a reader of
-- `tombstone.result` sees, so TombstoneEquivProps can lay the two
-- side by side over an exhaustive small domain.
module ReferenceTombstone (expected, rowAlphabet, smallAlphabet, budgets) where

import CE.Tombstone.Cost (budgetCode, kindProse, minMarks, minName, tombstoneRowCap)
import Data.Aeson (Value (..), object, toJSON, (.=))
import ReferenceContract (firstOffender, knobbedContract)

-- | Every row the exhaustive domain draws from: kinds 0..2 (bracketed
-- label, bare label, prose) by marks 0..3 by erased names 0..3.
rowAlphabet :: [[Integer]]
rowAlphabet = [[k, m, n] | k <- [0 .. 2], m <- [0 .. 3], n <- [0 .. 3]]

-- | One representative per (kind, verdict) class — the alphabet's
-- first row of each — for the longer sequences.
smallAlphabet :: [[Integer]]
smallAlphabet = concat [take 1 [r | r <- rowAlphabet, take 1 r == [k], site r == v] | k <- [0 .. 2], v <- [False, True]]

-- | No budget, then every budget 0..5.
budgets :: [Maybe Integer]
budgets = Nothing : map Just [0 .. 5]

-- | Does a row of this kind need a retrospective mark in its sentence?
-- The table form of booklet 14: only the prose kind reads the marks
-- column; both label kinds stand on the name alone.
needsMark :: [(Integer, Bool)]
needsMark = [(k, k == kindProse) | k <- [0 .. kindProse]]

-- | One row's verdict, read off the table.
site :: [Integer] -> Bool
site row = case row of
  [k, m, n] -> case lookup k needsMark of
    Just True -> m >= minMarks && n >= minName
    Just False -> n >= minName
    Nothing -> False
  _ -> False

-- | The contract as named predicates over the whole request; the
-- first that names an offender wins, rows before knobs.
offence :: [[Integer]] -> [[Integer]] -> Maybe String
offence = knobbedContract (named rowNames 3 "malformed row") (named knobNames 2 "malformed knob")
 where
  rowNames r = [("kind outside 0..2", take 1 r `notElem` map pure [0 .. kindProse]), ("negative count", any (< 0) (drop 1 r))]
  knobNames r = [("unknown knob code", take 1 r /= [budgetCode]), ("negative knob value", any (< 0) (drop 1 r))]

-- | A width check, then the first named condition that holds.
named :: ([Integer] -> [(String, Bool)]) -> Int -> String -> [Integer] -> Maybe String
named conditions width malformed r
  | length r /= width = Just malformed
  | otherwise = firstOffender [why | (why, True) <- conditions r]

-- | What the reply says, field by field (sites, counts, over, knobs,
-- degraded, reason) — or Left the refusal message's stem for a request
-- the contract refuses. The cap is judged before the contract, rows
-- and knobs counted together.
expected :: [[Integer]] -> [[Integer]] -> Either String [Maybe Value]
expected rows knobs
  | toInteger (length rows + length knobs) > tombstoneRowCap = Right (answer ([] :: [Integer]) False True)
  | Just why <- offence rows knobs = Left why
  | otherwise = Right (answer sites over False)
 where
  sites = [i | (i, r) <- zip [0 :: Integer ..] rows, site r]
  declared = [v | [c, v] <- knobs, c == budgetCode]
  over = or [toInteger (length sites) > b | b <- take 1 (reverse declared)]
  prose = length [() | r <- rows, site r, take 1 r == [kindProse]]
  answer ss o deg =
    [ Just (toJSON ss)
    , Just
        ( object
            [ "rows" .= length rows
            , "label" .= (if deg then 0 else length ss - prose)
            , "prose" .= (if deg then 0 else prose)
            ]
        )
    , Just (Bool o)
    , Just (toJSON [[budgetCode, b] | b <- take 1 (reverse declared)])
    , Just (Bool deg)
    , if deg then Just (String "tombstone_too_large") else Nothing
    ]
