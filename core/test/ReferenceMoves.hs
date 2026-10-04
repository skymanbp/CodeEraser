-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The L1 judgment's second implementation (plan v2.33 W3, moves/1):
-- every rule read straight off the measuring side's old prose, spelled
-- over plain lists — membership by `elem`, the owner by a strict fold,
-- tallies by `filter`, the runs by explicit recursion — where
-- CE.FourClass.Moves.Classify uses sets, maps and arrays. It answers
-- the `pairs` table a moves.result carries.
module ReferenceMoves (Pair, movesExpected, movesRequest, movesRequests) where

import CE.FourClass.Moves.Cost (maxBridge)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (nub, sortBy)
import Data.Ord (comparing)
import DocumentGen (seededBy)
import ReferenceFlowGen (G, rand)

-- | One side: line tokens (0 = an insignificant line), changed 0-based
-- indices, units [key, kind, start, end, stack].
type SideR = ([Integer], [Integer], [[Integer]])

type Pair = (SideR, SideR)

hashOf :: Integer -> Integer
hashOf t = t * 2654435761 + 7

widthOf :: Integer -> Integer
widthOf t = if t == 0 then 0 else 1 + t `mod` 3

keyHash :: Integer -> Integer
keyHash k = k * 40503 + 977

-- | The request over four key codes.
movesRequest :: [Pair] -> Value
movesRequest ps =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("moves.request" :: String)
    , "id" .= (1 :: Int)
    , "keys" .= map keyHash [0 .. 3]
    , "pairs" .= [object ["before" .= side b, "after" .= side a] | (b, a) <- ps]
    ]
 where
  side (toks, changed, units) =
    object ["lines" .= [[t, hashOf t, widthOf t] | t <- toks], "changed" .= changed, "units" .= units]

movesExpected :: [Pair] -> Value
movesExpected = toJSON . map pairExpected

pairExpected :: Pair -> Value
pairExpected (b@(bToks, rem', bUnits), a@(aToks, add, aUnits)) =
  object
    [ "counts" .= [len add - len movedAdd, len movedAdd, len rem' - len movedRem, len movedRem]
    , "moved" .= movedRows
    , "relocated" .= relocated b a movedRows
    , "declRem" .= oneSided bUnits aUnits
    , "declAdd" .= oneSided aUnits bUnits
    , "dupSpans" .= stacked bUnits aUnits
    , "runsRem" .= runs bToks rem' movedRem
    , "runsAdd" .= runs aToks add movedAdd
    ]
 where
  len = toInteger . length
  at toks i = toks !! fromInteger i
  movedIn toks is otherToks others =
    [i | i <- is, at toks i /= 0, at toks i `elem` [at otherToks j | j <- others, at otherToks j /= 0]]
  movedRem = movedIn bToks rem' aToks add
  movedAdd = movedIn aToks add bToks rem'
  movedRows = [[i + 1, 1, owner bUnits (i + 1)] | i <- movedRem] <> [[j + 1, 0, owner aUnits (j + 1)] | j <- movedAdd]

-- | The narrowest unit holding the line, the earliest on a tie; -1.
owner :: [[Integer]] -> Integer -> Integer
owner units line = snd (foldl pick (Nothing, -1) (zip [0 ..] units))
 where
  pick (best, _) (i, [_, _, s, e, _])
    | s <= line && line <= e && maybe True (e - s <) best = (Just (e - s), i)
  pick acc _ = acc

relocated :: SideR -> SideR -> [[Integer]] -> [Integer]
relocated (_, rem', bUnits) (_, add, aUnits) movedRows =
  [ k
  | [k, _, bs, be, _] <- bUnits
  , [_, _, as, ae, _] <- take 1 (filter ((== [k]) . take 1) aUnits)
  , let rm = count (\i -> bs <= i + 1 && i < be) rem'
  , let ad = count (\i -> as <= i + 1 && i < ae) add
  , rm + ad > 0
  , ownedBy 1 bUnits k == rm
  , ownedBy 0 aUnits k == ad
  ]
 where
  count p = length . filter p
  ownedBy r units k = length [() | [_, r', o] <- movedRows, r' == r, o >= 0, take 1 (units !! fromInteger o) == [k]]

oneSided :: [[Integer]] -> [[Integer]] -> [Integer]
oneSided side other =
  map snd (sortBy (comparing fst) [((k, kind), i) | (i, [k, kind, _, _, _]) <- zip [0 ..] side, times side k kind == 1, times other k kind == 0])
 where
  times us k kind = length (filter (\u -> take 2 u == [k, kind]) us)

stacked :: [[Integer]] -> [[Integer]] -> [[Integer]]
stacked bUnits aUnits =
  sortBy compare
    [ [keyHash k, s, e]
    | k <- nub [k' | (k' : _) <- top aUnits]
    , let keyed us = filter ((== [k]) . take 1) (top us)
    , let mine = keyed aUnits
    , length mine >= 2
    , length mine > length (keyed bUnits)
    , [_, _, s, e, _] <- mine
    ]
 where
  top us = [u | u@[_, _, s, e, 1] <- us, null [() | [_, _, s', e', _] <- us, (s' < s && e <= e') || (s' <= s && e < e')]]

-- | The leftover runs, by explicit recursion over the changed lines.
runs :: [Integer] -> [Integer] -> [Integer] -> [[[Integer]]]
runs toks changed moved = go (map (+ 1) changed) 0 0 False []
 where
  go [] _ _ _ acc = reverse acc
  go (l : ls) prev kept open acc
    | t == 0 = go ls l kept open' acc
    | (l - 1) `elem` moved = go ls l kept False acc
    | open' && l - kept - 1 <= toInteger maxBridge = go ls l l True (grow acc)
    | otherwise = go ls l l True ([entry] : acc)
   where
    t = toks !! fromInteger (l - 1)
    open' = open && l == prev + 1
    entry = [l, hashOf t, widthOf t]
    grow (r : rs) = (r <> [entry]) : rs
    grow [] = [[entry]]

-- | Two hundred seeded changesets: one to three pairs, each side zero
-- to twelve lines over five tokens (0 insignificant) so moves, blanks
-- and bridges all occur, half the lines changed, up to four units over
-- four keys and three kinds.
movesRequests :: [[Pair]]
movesRequests = seededBy 4099 31 changeset

changeset :: G [Pair]
changeset = do
  n <- (+ 1) <$> rand 3
  mapM (const ((,) <$> sideG <*> sideG)) [1 .. n]

sideG :: G SideR
sideG = do
  n <- rand 13
  toks <- mapM (const (toInteger <$> rand 5)) [1 .. n]
  flips <- mapM (const (rand 2)) [1 .. n]
  units <- if n == 0 then pure [] else rand 5 >>= \m -> mapM (const (unitG n)) [1 .. m]
  pure (toks, [toInteger i | (i, f) <- zip [0 :: Int ..] flips, f == 0], units)
 where
  unitG n = do
    k <- rand 4
    kind <- rand 3
    from <- (+ 1) <$> rand n
    w <- rand n
    st <- rand 2
    pure (map toInteger [k, kind, from, min n (from + w), st])
