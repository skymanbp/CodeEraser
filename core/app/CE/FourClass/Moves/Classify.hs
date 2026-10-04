-- | The L1 judgment over one changed file pair (plan v2.33 W3), a
-- transliteration of cli/src/fourclass/model.rs `classify` /
-- `relocated`, decls.rs `tables`, stacking.rs `dup_spans` and batch.rs
-- `side_runs` at 093aede4, statement for statement. The measuring side's
-- line diff says which lines changed; a changed significant line whose
-- trimmed content also occurs among the other side's changed significant
-- lines is a move inside the pair. Contents cross as request-local codes
-- (equal code = equal trimmed text), so membership is exact.
module CE.FourClass.Moves.Classify (Classified (..), Unit (..), Line (..), classify, unitOf) where

import CE.FourClass.Moves.Cost (maxBridge)
import Data.Array (Array, listArray, (!))
import Data.List (sortBy)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import qualified Data.Set as S

-- | One line: content code (equal trimmed text), fnv1a64 of the trimmed
-- text, alphanumeric width (0 = not significant).
data Line = Line {lCode, lHash, lWidth :: Integer}

-- | One unit: key code, kind, 1-based inclusive span, stack bit.
data Unit = Unit {uKey :: Int, uKind :: Integer, uFrom, uTo :: Int, uStack :: Bool}

unitOf :: [Integer] -> Unit
unitOf row = case row of
  [k, kind, from, to, stack] -> Unit (fromInteger k) kind (fromInteger from) (fromInteger to) (stack == 1)
  _ -> Unit 0 0 1 1 False -- unreachable: shape validated upstream

-- | One pair's answer: the four counts [addedNovel, addedMoved,
-- removedDeleted, removedMoved]; the moved lines [line, removed, unit]
-- (unit = the side's unit index, -1 = top level), before's first; the
-- relocated before units' key codes; the one-sided declarations as unit
-- indices; the duplicated spans [keyHash, start, end]; the leftover runs
-- of each side, each entry [line, hash, width].
data Classified = Classified
  { counts :: [Int]
  , moved :: [[Int]]
  , relocated :: [Int]
  , declRem, declAdd :: [Int]
  , dupSpans :: [[Integer]]
  , runsRem, runsAdd :: [[[Integer]]]
  }

-- | A side: lines by 0-based index, changed 0-based indices ascending,
-- units in segmentation order.
type SideIn = (Array Int Line, [Int], [Unit])

significant :: Line -> Bool
significant l = lWidth l > 0

-- | model.rs `classify`: removed lines first (each moved or deleted),
-- then added lines (each moved or novel); then the relocation summary,
-- the declaration tables, the stacking spans and the leftover runs.
classify :: Array Int Integer -> SideIn -> SideIn -> Classified
classify keyHash before@(a, removed, bUnits) after@(b, added, aUnits) =
  Classified
    { counts = [length added - length movedAdd, length movedAdd, length removed - length movedRem, length movedRem]
    , moved = movedRows
    , relocated = relocatedUnits movedRows before after
    , declRem = oneSided bUnits aUnits
    , declAdd = oneSided aUnits bUnits
    , dupSpans = dupSpansOf keyHash bUnits aUnits
    , runsRem = sideRuns a removed [l | [l, 1, _] <- movedRows]
    , runsAdd = sideRuns b added [l | [l, 0, _] <- movedRows]
    }
 where
  sigContents ls changed = S.fromList [lCode (ls ! i) | i <- changed, significant (ls ! i)]
  removedSig = sigContents a removed
  addedSig = sigContents b added
  movedRem = [i | i <- removed, significant (a ! i), S.member (lCode (a ! i)) addedSig]
  movedAdd = [j | j <- added, significant (b ! j), S.member (lCode (b ! j)) removedSig]
  movedRows = [movedLine i 1 bUnits | i <- movedRem] <> [movedLine j 0 aUnits | j <- movedAdd]

-- | model.rs `moved_line`: 1-based line, side flag, owning unit.
movedLine :: Int -> Int -> [Unit] -> [Int]
movedLine idx removed units = [idx + 1, removed, owner units (idx + 1)]

-- | units.rs `owner`: among units holding the line, the one with the
-- least `end - start`; Iterator::min_by_key keeps the FIRST minimum.
-- -1 = the file's top level.
owner :: [Unit] -> Int -> Int
owner units line = maybe (-1) snd (foldl' keep Nothing holding)
 where
  holding = [(uTo u - uFrom u, i) | (i, u) <- zip [0 ..] units, uFrom u <= line, line <= uTo u]
  keep Nothing x = Just x
  keep (Just best) x = if fst x < fst best then Just x else Just best

-- | model.rs `relocated`: for each before unit in order, the FIRST after
-- unit of its key; relocated when a changed line sits in either span
-- and every changed line in each span is a move owned by a unit of
-- that key on its side.
relocatedUnits :: [[Int]] -> SideIn -> SideIn -> [Int]
relocatedUnits movedRows (_, removed, bUnits) (_, added, aUnits) =
  [ uKey bu
  | bu <- bUnits
  , au <- take 1 [u | u <- aUnits, uKey u == uKey bu]
  , let rm = changedIn bu removed
  , let ad = changedIn au added
  , rm + ad > 0
  , movedOf 1 bUnits (uKey bu) == rm
  , movedOf 0 aUnits (uKey au) == ad
  ]
 where
  movedOf removedFlag units key =
    let keys = listArray (0, length units - 1) (map uKey units) :: Array Int Int
     in length [() | [_, r, o] <- movedRows, r == removedFlag, o >= 0, keys ! o == key]
  changedIn u changed = length [() | i <- changed, uFrom u <= i + 1, i < uTo u]

-- | decls.rs `one_sided`: the (key, kind) tally of each side, the units
-- of this side whose identity occurs exactly once here and never there,
-- sorted on (key, kind) — key codes rank the key texts, so code order is
-- Rust's String order.
oneSided :: [Unit] -> [Unit] -> [Int]
oneSided side other =
  map snd (sortBy (comparing fst) [((uKey u, uKind u), i) | (i, u) <- zip [0 ..] side, mine M.! ident u == 1, M.notMember (ident u) theirs])
 where
  ident u = (uKey u, uKind u)
  tally us = M.fromListWith (+) [(ident u, 1 :: Int) | u <- us]
  (mine, theirs) = (tally side, tally other)

-- | stacking.rs `dup_spans`: the top-level stackable spans of each side
-- grouped by key (source order kept), every after-side group of two or
-- more that outnumbers its before-side group, flattened to
-- [fnv1a(key), start, end] and sorted.
dupSpansOf :: Array Int Integer -> [Unit] -> [Unit] -> [[Integer]]
dupSpansOf keyHash bUnits aUnits =
  sortBy
    compare
    [ [keyHash ! k, toInteger s, toInteger e]
    | (k, spans) <- M.toList afterSpans
    , length spans >= 2
    , length spans > maybe 0 length (M.lookup k beforeSpans)
    , (s, e) <- spans
    ]
 where
  (beforeSpans, afterSpans) = (topLevelSpans bUnits, topLevelSpans aUnits)

-- | stacking.rs `top_level_spans`: units no unit of the side encloses
-- (strictly on one end), stackable keys only, grouped by key.
topLevelSpans :: [Unit] -> M.Map Int [(Int, Int)]
topLevelSpans units = foldl' add M.empty [u | u <- units, topLevel u, uStack u]
 where
  topLevel u = not (any (\v -> (uFrom v < uFrom u && uTo u <= uTo v) || (uFrom v <= uFrom u && uTo u < uTo v)) units)
  add m u = M.insertWith (flip (<>)) (uKey u) [(uFrom u, uTo u)] m

-- | batch.rs `side_runs`: walk the changed lines (1-based, ascending);
-- an unchanged gap closes the open run, an insignificant line bridges,
-- a moved line closes the run, an over-long bridge closes it, any other
-- line extends the open run or starts a new one.
sideRuns :: Array Int Line -> [Int] -> [Int] -> [[[Integer]]]
sideRuns ls changed movedLines = reverse (map reverse runs)
 where
  (_, _, _, runs) = foldl' step (False, 0, 0, []) (map (+ 1) changed)
  step (open0, prev, lastKept, acc) l
    | not (significant t) = (open1, l, lastKept, acc)
    | l `elem` movedLines = (False, l, lastKept, acc)
    | open2 = (True, l, l, extend acc)
    | otherwise = (True, l, l, [entry] : acc)
   where
    open1 = open0 && l == prev + 1
    open2 = open1 && not (l - lastKept - 1 > maxBridge)
    t = ls ! (l - 1)
    entry = [toInteger l, lHash t, lWidth t]
    extend (r : rs) = (entry : r) : rs
    extend [] = [[entry]]
