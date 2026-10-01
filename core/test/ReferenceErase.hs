-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The erase family's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 12's safety predicate as per-class
-- LISTS of (refusal reason, holds) whose first holding entry names the
-- refusal, and the target closure as a SET-FORM definition checked by
-- pairwise comparison — a row stands for its target when no eraseable
-- whole-file row owns its path from above and no other open row of
-- the same target outranks it (eraseable first, then the class's
-- licence or, unlicensed, its advisory precedence, then the earlier
-- row). The shipped closure groups by Map and folds; this one compares
-- every pair. Only the family's frozen constants are read.
module ReferenceErase (expected, exhaustive) where

import CE.Erase.Cost (eraseRowCap, publicDeadVerdicts)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.List (sortOn)
import ReferenceContract (breaks, firstOffender, labelled)

-- | One row's (eraseable, reason): the class's refusal list in its
-- booklet order, the first entry that holds names the reason.
verdict :: [Integer] -> (Bool, Integer)
verdict row = case [why | (why, True) <- refusals row] of
  (why : _) -> (False, why)
  [] -> (True, 0)

refusals :: [Integer] -> [(Integer, Bool)]
refusals row = case row of
  [1, verbatim, a, b, same] -> [(2, verbatim < max a b), (3, same /= 1)]
  [2, covered, same, copy, unresolved] ->
    [(5, covered /= 1), (3, same /= 1), (4, copy == 0), (6, copy `elem` publicDeadVerdicts), (1, unresolved /= 0)]
  [3, dead, conf, _, _] -> [(6, dead `elem` publicDeadVerdicts), (1, conf == 0)]
  _ -> [(4, True)]

-- | (licence when eraseable, advisory precedence when not) per class:
-- the twin names the live unit it duplicates, the dead file only its
-- death; unlicensed, the dead file's own refusal is the one shown.
precedence :: [(Integer, (Integer, Integer))]
precedence = [(1, (0, 0)), (2, (2, 1)), (3, (1, 2))]

standing :: [Integer] -> Bool -> (Integer, Integer)
standing row ok = case lookup (classOf row) precedence of
  Just (lic, adv) -> if ok then (1, lic) else (0, adv)
  Nothing -> (if ok then 1 else 0, 0)
 where
  classOf r = case r of
    (c : _) -> c
    [] -> 0

-- | One bit per row: does it stand for its target?
kept :: [[Integer]] -> [[Integer]] -> [Bool]
kept targets rows = [isOpen i && all (not . outranks i) others | (i, _) <- indexed]
 where
  oks = map (fst . verdict) rows
  indexed = zip [0 :: Int ..] (zip targets rows)
  others = [j | (j, _) <- indexed]
  at i = snd (indexed !! i)
  key i = take 3 (fst (at i))
  owners = [p | (j, ([p, 0, 0], _)) <- indexed, oks !! j]
  isOpen i = case key i of
    [p, s, _] -> not (s > 0 && p `elem` owners)
    _ -> True
  rank i = standing (snd (at i)) (oks !! i)
  outranks i j = j /= i && isOpen j && key j == key i && (rank j > rank i || (rank j == rank i && j < i))

-- | The contract, in the cascade's order: each row, then any knob,
-- then the target table (alignment, each target, key order).
offence :: [[Integer]] -> [[Integer]] -> Maybe [[Integer]] -> Maybe String
offence rows knobs targets =
  firstOffender (labelled "row" rowWhy rows <> labelled "knob" (const (Just "erase/1 declares no knob codes")) knobs <> targetWhys)
 where
  targetWhys = case targets of
    Nothing -> []
    Just ts
      | length ts /= length rows -> ["targets: " <> show (length ts) <> " rows for " <> show (length rows) <> " fact rows"]
      | otherwise ->
          labelled "target" (uncurry targetWhy) (zip ts rows)
            <> breaks "target" "out of key order" (\a b -> take 3 a > take 3 b) ts

-- | One fact row's contract as (offence, holds) in the cascade's
-- order; the first that holds names the refusal.
rowWhy :: [Integer] -> Maybe String
rowWhy r = case r of
  [c, w, x, y, z] ->
    firstOffender
      [ why
      | (why, True) <-
          [ ("retired class 0", c == 0)
          , ("unknown class", c `notElem` [1, 2, 3])
          , ("negative fact", minimum [w, x, y, z] < 0)
          , ("dead verdict outside 1..4", c == 3 && w `notElem` [1 .. 4])
          , ("bytesEqual not a boolean", c == 1 && z > 1)
          , ("coverage/equality not booleans", c == 2 && max w x > 1)
          , ("dead verdict outside 0..4", c == 2 && y > 4)
          , ("confidence outside 0..2", c == 3 && x > 2)
          ]
      ]
  _ -> Just "malformed row"

targetWhy :: [Integer] -> [Integer] -> Maybe String
targetWhy t r = case (t, take 1 r) of
  ([p, s, e], [c])
    | minimum [p, s, e] < 0 -> Just "negative field"
    | (s == 0) /= (e == 0) -> Just "half-open span"
    | s > e -> Just "span end before start"
    | c == 1 && s == 0 -> Just "verbatim_doc row names a whole file"
    | c /= 1 && s > 0 -> Just "whole-file class names a span"
    | otherwise -> Nothing
  _ -> Just "malformed row"

-- | The reply fields (rows, counts, fail, degraded, reason, kept), or
-- Left the refusal's message stem. The cap counts fact rows alone.
expected :: [[Integer]] -> [[Integer]] -> Maybe [[Integer]] -> Either String [Maybe Value]
expected rows knobs targets
  | toInteger (length rows) > eraseRowCap = Right (answer ([] :: [(Bool, Integer)]) True)
  | Just why <- offence rows knobs targets = Left why
  | otherwise = Right (answer (map verdict rows) False)
 where
  answer vs deg =
    [ Just (toJSON [[if e then 1 else 0, why] | (e, why) <- vs])
    , Just (object ["rows" .= length rows, "eraseable" .= length (filter fst vs), "advisory" .= length (filter (not . fst) vs)])
    , Just (Bool deg)
    , Just (Bool deg)
    , if deg then Just (String "erase_too_large") else Nothing
    , if deg then Nothing else fmap (\ts -> toJSON [if k then 1 else 0 :: Integer | k <- kept ts rows]) targets
    ]

-- | The exhaustive closure family: every SEQUENCE of up to three rows
-- (so rows sharing a target arrive in every order, and the earlier-row
-- tie is exercised both ways) and every multiset of four rows
-- over three paths, a whole-file twin and a whole-file dead row each
-- eraseable or not, and a verbatim segment on one of two spans
-- eraseable or not — each sent in key order, stably.
exhaustive :: [([[Integer]], [[Integer]])]
exhaustive = [unzip (sortOn fst picks) | picks <- concatMap (\n -> sequence (replicate n alphabet)) [0 .. 3] <> multisets (4 :: Int) alphabet]
 where
  alphabet =
    [ ([p, s, e], row)
    | p <- [0 .. 2]
    , (s, e, row) <-
        [(0, 0, [2, 1, 1, 1, 0]), (0, 0, [2, 1, 1, 4, 0]), (0, 0, [3, 1, 2, 0, 0]), (0, 0, [3, 2, 2, 0, 0])]
          <> [(s, e, [1, v, 9, 9, 1]) | (s, e) <- [(1, 5), (3, 9)], v <- [9, 4]]
    ]
  multisets 0 _ = [[]]
  multisets _ [] = []
  multisets k xs@(x : rest) = map (x :) (multisets (k - 1) xs) <> multisets k rest
