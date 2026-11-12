{-# LANGUAGE OverloadedStrings #-}

-- | The extent law's request tables refused by name (CE.Verdict.ExtentRows),
-- and the battery's non-vacuity: every axis carries extent on some
-- tree of the families, and some score lands strictly inside the scale.
module VerdictExtentRefusals (refusalLegs) where

import CE.Verdict (respond)
import Data.Aeson (Value, toJSON)
import ReferenceExtent
import VerdictExtentGen
import WireHarness (dropKey, refusedBy, setKey)

refusalLegs :: [(String, Bool)]
refusalLegs =
  [ ("extent refusal: a request without one of the four tables", refusedBy respond (dropKey "proseLines" base) "extent: contSpans, dupSpans, refLines and proseLines ride on every request")
  , ("extent refusal: a contSpans table one row short", refusedBy respond (with "contSpans" [[0, 0, 0]]) "contSpans: need one row per continuous row")
  , ("extent refusal: a size row carrying a span", refusedBy respond (with "contSpans" [[0, 1, 2], [0, 1, 5]]) "contSpans 0: only a function row carries a span")
  , ("extent refusal: a function span with hi < lo", refusedBy respond (with "contSpans" [[0, 0, 0], [0, 4, 2]]) "contSpans 1: a function span needs 1 <= lo <= hi")
  , ("extent refusal: an unknown file index", refusedBy respond (with "dupSpans" [[7, 1, 1, 0]]) "dupSpans 0: unknown file index")
  , ("extent refusal: a duplicate span outside its file", refusedBy respond (with "dupSpans" [[0, 4, 9, 0]]) "dupSpans 0: span outside its file")
  , ("extent refusal: a duplicate span with hi < lo", refusedBy respond (with "dupSpans" [[0, 3, 2, 0]]) "dupSpans 0: span hi < lo")
  , ("extent refusal: a reference line outside its file", refusedBy respond (with "refLines" [[0, 1, 6]]) "refLines 0: line outside its file")
  , ("extent refusal: a universe file with no line count", refusedBy respond (with "proseLines" []) "extent: file 1 has no line count")
  , ("extent refusal: a prose line count for a file with a size row", refusedBy respond (with "proseLines" [[0, 5]]) "proseLines 0: the file owns a size row")
  , ("extent refusal: a second size row for one file", refusedBy respond (setKey "contSpans" (toJSON twinSpans) twinBase) "contSpans 1: a second size row for one file")
  , ("extent refusal: a descending dupSpans table", refusedBy respond (with "dupSpans" [[0, 2, 2, 0], [0, 1, 1, 0]]) "dupSpans 1: not strictly ascending")
  , ("extent non-vacuity: every axis carries extent somewhere, and some score is strictly inside the scale", nonVacuous)
  ]
 where
  -- a five-line code file with one function, beside a three-line prose file
  tree = MTree [(blank 5) {mFuncs = [(1, 3, 20)]}, (blank 3) {mProse = True}] [] 3 15 2
  base = requestOf (replicate 7 1) 10 tree
  with :: String -> [[Integer]] -> Value
  with k rows = setKey k (toJSON rows) base
  -- two code files: point the second one's size row at the first file
  twin = MTree [blank 4, blank 6] [] 3 15 2
  twinBase = requestOf (replicate 7 1) 10 twin
  twinSpans = [[0, 0, 0], [0, 0, 0 :: Integer]]

nonVacuous :: Bool
nonVacuous = and [any (\t -> refExtent (== a) t > 0) trees | a <- [0 .. 6]] && any inside trees
 where
  trees = exhaustive <> randomFamily
  inside t = let (s, _, _) = refScore (replicate 7 1) 1 t in s > 0 && s < 1000
