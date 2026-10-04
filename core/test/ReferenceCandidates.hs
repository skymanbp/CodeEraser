-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates family's second implementation (plan v2.33 W3):
-- design vol.2 §4.2's same-key source and §4.3's two bounds, and the
-- exhaustive source S5, spelled as set comprehensions — histograms are
-- Data.Map multisets intersected by Map.intersectionWith, the union is
-- a Data.Map from pair to OR-ed source bits, the S5 window is every
-- same-language pair whose smaller node count passes the size bound
-- against the larger (no sorting, no walk, no early exit). It reads only
-- the 85/100 ratio and the caps and answers the reply fields
-- `candidates.result` shows.
module ReferenceCandidates (expected, requests) where

import CE.Candidates.Cost (candidatePairCap, candidateUnitCap)
import CE.Clone.Cost (tsedDen, tsedNum)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Bits ((.|.))
import qualified Data.Map.Strict as M
import ReferenceContract (breaks, firstOffender, labelled)
import ReferenceFlowGen (G, S (..), rand, runG)

-- | A unit as the reference reads it: language, file, key, nodes, kind
-- multiset.
data U = U {uLang, uFile, uKey, uNodes :: Integer, uHist :: M.Map Integer Integer}

readUnit :: [Integer] -> U
readUnit row = case row of
  (l : f : k : n : hist) -> U l f k n (M.fromList (twos hist))
  _ -> U 0 0 0 0 M.empty
 where
  twos (a : b : more) = (a, b) : twos more
  twos _ = []

-- | "q cannot reach 85/100 of mx".
short :: Integer -> Integer -> Bool
short q mx = not (q * tsedDen >= tsedNum * mx)

-- | 0 = kept, 1 = the size bound, 2 = the label bound.
verdictOf :: U -> U -> Int
verdictOf x y
  | short (min (uNodes x) (uNodes y)) top = 1
  | short (M.foldr (+) 0 (M.intersectionWith min (uHist x) (uHist y))) top = 2
  | otherwise = 0
 where
  top = max (uNodes x) (uNodes y)

offence :: [[Integer]] -> [[Integer]] -> Maybe String
offence units pairs =
  firstOffender
    ( labelled "unit" unitWhy units
        <> labelled "pair" pairWhy pairs
        <> breaks "pair" "not strictly ascending" (\a b -> take 2 a >= take 2 b) pairs
    )
 where
  n = toInteger (length units)
  langOf i = take 1 (concat (take 1 (drop (fromInteger i) units)))
  unitWhy row = case row of
    (l : f : k : nodes : hist)
      | l < 0 -> Just "negative language"
      | f < 0 -> Just "negative file"
      | k < 0 -> Just "negative key"
      | nodes < 1 -> Just "non-positive nodes"
      | odd (length hist) -> Just "histogram not [kind,count] pairs"
      | any (< 0) (evens hist) -> Just "negative kind"
      | any (< 1) (evens (drop 1 hist)) -> Just "non-positive count"
      | evens hist /= M.keys (M.fromList (zip (evens hist) (repeat ()))) -> Just "kinds not strictly ascending"
      | sum (evens (drop 1 hist)) /= nodes -> Just "histogram does not sum to nodes"
      | otherwise -> Nothing
    _ -> Just "malformed unit"
  pairWhy row = case row of
    [a, b, s]
      | a < 0 || b >= n -> Just "endpoint out of range"
      | a >= b -> Just "endpoints not ordered"
      | langOf a /= langOf b -> Just "cross-language pair"
      | s `notElem` [1, 4, 5, 8, 9, 12, 13] -> Just "sources not a subset of the sent generators"
      | otherwise -> Nothing
    _ -> Just "malformed pair"
  evens xs = [x | (i, x) <- zip [0 :: Int ..] xs, even i]

-- | The reply fields (pairs, keyPairs, counts, degraded, reason), or
-- Left the refusal's message stem.
expected :: Bool -> [[Integer]] -> [[Integer]] -> Either String [Maybe Value]
expected ex units pairs
  | toInteger (length units) > candidateUnitCap || toInteger (length pairs) > candidatePairCap =
      Right (answer [] [] (replicate 9 0) True)
  | Just why <- offence units pairs = Left why
  | otherwise = Right (answer (M.toList finalBits) (M.toList byLang) tallies False)
 where
  us = M.fromList (zip [0 ..] (map readUnit units))
  unit i = us M.! i
  sameKey =
    [ (i, j)
    | (i, x) <- M.toList us
    , (j, y) <- M.toList us
    , i < j
    , uKey x == uKey y
    , uFile x /= uFile y
    ]
  (sameLang, crossed) = (filter sameL sameKey, filter (not . sameL) sameKey)
  sameL (i, j) = uLang (unit i) == uLang (unit j)
  unionBits = M.unionWith (.|.) (M.fromList [((a, b), s) | [a, b, s] <- pairs]) (M.fromList [(p, 2) | p <- sameLang])
  judged = M.mapWithKey (\(a, b) _ -> verdictOf (unit a) (unit b)) unionBits
  kept = M.filter (== 0) judged
  keptBits = M.intersection unionBits kept
  window =
    [ (i, j)
    | (i, x) <- M.toList us
    , (j, y) <- M.toList us
    , i < j
    , uLang x == uLang y
    , not (short (min (uNodes x) (uNodes y)) (max (uNodes x) (uNodes y)))
    ]
  already = [p | p <- window, M.member p kept]
  cutS5 = [p | p@(i, j) <- window, not (M.member p kept), verdictOf (unit i) (unit j) == 2]
  newS5 = [p | p@(i, j) <- window, not (M.member p kept), verdictOf (unit i) (unit j) == 0]
  finalBits = if ex then M.union keptBits (M.fromList [(p, 16) | p <- newS5]) else keptBits
  byLang = M.fromListWith (+) [(uLang (unit i), 1 :: Integer) | (i, _) <- sameLang]
  count v = toInteger (M.size (M.filter (== v) judged))
  s5 xs = if ex then toInteger (length xs) else 0
  tallies =
    [ toInteger (M.size unionBits)
    , toInteger (length crossed)
    , count 1
    , count 2
    , count 0
    , s5 window
    , s5 cutS5
    , s5 already
    , s5 newS5
    ]
  answer :: [((Integer, Integer), Integer)] -> [(Integer, Integer)] -> [Integer] -> Bool -> [Maybe Value]
  answer rows langs ns deg =
    [ Just (toJSON [[a, b, s] | ((a, b), s) <- rows])
    , Just (toJSON [[l, c] | (l, c) <- langs])
    , Just (object (zipWith (.=) names (toInteger (length units) : toInteger (length pairs) : ns)))
    , Just (Bool deg)
    , if deg then Just (String "candidates_too_large") else Nothing
    ]
  names = ["units", "pairs", "union", "crossLanguage", "prunedSize", "prunedLabel", "survivors", "s5Windowed", "s5PrunedLabel", "s5Already", "s5New"]

-- | Two hundred seeded requests: two to nine units in one or two
-- languages, three files and three keys (so S2 finds, crosses
-- languages and meets the sent sources), node counts 20..67 (so the
-- 85/100 window both admits and cuts), histograms over five kinds, and
-- a random ascending subset of the same-language pairs as the sent
-- union, each with a non-empty subset of the S1/S3/S4 bits.
requests :: [(Bool, [[Integer]], [[Integer]])]
requests = [runG request (S (n * 5003 + 41) 0 0 0) | n <- [1 .. 200 :: Int]]

request :: G (Bool, [[Integer]], [[Integer]])
request = do
  ex <- rand 3
  size <- (+ 2) <$> rand 8
  units <- mapM (const unitG) [1 .. size]
  picks <- mapM (const (rand 3)) [1 .. size * size]
  srcs <- mapM (const (rand 7)) [1 .. size * size]
  let langOf i = take 1 (units !! i)
      same = [(i, j) | i <- [0 .. size - 1], j <- [i + 1 .. size - 1], langOf i == langOf j]
      bits = [1, 4, 5, 8, 9, 12, 13]
      chosen = [[toInteger i, toInteger j, bits !! s] | ((i, j), p, s) <- zip3 same picks srcs, p /= 0]
  pure (ex /= 0, units, chosen)

unitG :: G [Integer]
unitG = do
  l <- toInteger <$> rand 2
  f <- toInteger <$> rand 3
  k <- toInteger <$> rand 3
  counts <- mapM (const (toInteger <$> rand 12)) [1 .. 4 :: Int]
  pad <- toInteger <$> rand 4
  let hist = [(kind, c) | (kind, c) <- zip [0 ..] counts, c > 0] <> [(9, pad + 20)]
  pure (l : f : k : sum (map snd hist) : concat [[kind, c] | (kind, c) <- hist])
