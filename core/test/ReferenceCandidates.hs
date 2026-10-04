-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates family's second implementation (plan v2.33 W3):
-- design vol.2 §4.2's four sources, §4.3's two bounds and the
-- exhaustive source S5, spelled as set comprehensions over Data.Map —
-- a line anchor's owner is the minimum of the containing units by
-- (width, id), MinHash hashes each element ‖ salt byte by byte with no
-- shortcut, a band key hashes its rows' bytes, hash groups and band
-- buckets are Map keys, the union is a Data.Map from pair to OR-ed
-- source bits, histograms are Data.Map multisets intersected by
-- Map.intersectionWith, and the S5 window is every same-language pair
-- whose smaller node count passes the size bound against the larger.
-- It reads only the 85/100 ratio, the LSH shape, the hot cap and the
-- ceilings, and answers the reply fields `candidates.result` shows.
module ReferenceCandidates (Req (..), bandKeysOf, expected, hot, pairsIn, requests) where

import CE.Candidates.Cost (candidateNearCap, candidatePrintCap, candidateSigCap, candidateUnitCap, lshShape)
import CE.Clone.Cost (tsedDen, tsedNum)
import CE.Dedup.Cost (hotCap)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Bits (shiftR, xor, (.&.), (.|.))
import Data.List (minimumBy, sortOn, tails)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import Data.Word (Word64, Word8)
import ReferenceContract (firstOffender, labelled)
import ReferenceFlowGen (G, S (..), rand, runG)

-- | One request: exhaustive, units, sigs, prints, near runs.
data Req = Req {rEx :: Bool, rUnits, rSigs, rPrints, rNear :: [[Integer]]}

-- | A unit as the reference reads it.
data U = U {uLang, uFile, uKey, uNodes, uFrom, uTo :: Integer, uHist :: M.Map Integer Integer}

readUnit :: [Integer] -> U
readUnit row = case row of
  (l : f : k : n : a : b : hist) -> U l f k n a b (M.fromList (twos hist))
  _ -> U 0 0 0 0 0 0 M.empty
 where
  twos (x : y : more) = (x, y) : twos more
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

-- | fnv1a64 over a byte list.
fnv :: [Word8] -> Word64
fnv = foldl (\h b -> (h `xor` fromIntegral b) * 1099511628211) 14695981039346656037

leBytes :: Int -> Integer -> [Word8]
leBytes width x = [fromIntegral ((x `shiftR` (8 * k)) .&. 255) | k <- [0 .. width - 1]]

-- | A group past the hot cap (CE.Dedup.Cost's).
hot :: [a] -> Bool
hot xs = toInteger (length xs) > hotCap

-- | A group's pairs: each element with every later one, or — a hot
-- group — each with its successor alone.
pairsIn :: [a] -> [(a, a)]
pairsIn xs
  | hot xs = zip xs (drop 1 xs)
  | otherwise = concat (zipWith (\a rest -> map ((,) a) rest) xs (drop 1 (tails xs)))

-- | The band keys of a set: MinHash rows, then each band's row bytes.
bandKeysOf :: [Integer] -> [(Integer, Word64)]
bandKeysOf set = [(toInteger b, fnv (concatMap (leBytes 8 . toInteger) (take rows (drop (b * rows) sig)))) | b <- [0 .. bands - 1]]
 where
  (perms, bands, rows) = let (p, b, r) = lshShape in (fromInteger p, fromInteger b, fromInteger r) :: (Int, Int, Int)
  sig = [minimum [fnv (leBytes 8 x <> leBytes 4 (toInteger i)) | x <- set] | i <- [0 .. perms - 1]]

offence :: Req -> Maybe String
offence r =
  firstOffender
    ( labelled "unit" unitWhy (rUnits r)
        <> ["sigs: not one set per unit" | length (rSigs r) /= length (rUnits r)]
        <> labelled "sig" sigWhy (rSigs r)
        <> labelled "print" (width4 "malformed print" printWhy) (rPrints r)
        <> labelled "near" (width4 "malformed near run" nearWhy) (rNear r)
    )
 where
  word x = x >= 0 && x < 2 ^ (64 :: Int)
  unitWhy row = case row of
    (l : f : k : nodes : a : b : hist)
      | l < 0 -> Just "negative language"
      | f < 0 -> Just "negative file"
      | k < 0 -> Just "negative key"
      | nodes < 1 -> Just "non-positive nodes"
      | a < 1 || b < a -> Just "span not 1-based and ordered"
      | odd (length hist) -> Just "histogram not [kind,count] pairs"
      | any (< 0) (evens hist) -> Just "negative kind"
      | any (< 1) (evens (drop 1 hist)) -> Just "non-positive count"
      | evens hist /= M.keys (M.fromList (zip (evens hist) (repeat ()))) -> Just "kinds not strictly ascending"
      | sum (evens (drop 1 hist)) /= nodes -> Just "histogram does not sum to nodes"
      | otherwise -> Nothing
    _ -> Just "malformed unit"
  sigWhy set
    | null set = Just "empty set"
    | not (all word set) = Just "element outside u64"
    | otherwise = Nothing
  printWhy row = if not (word (head' row)) then Just "hash outside u64" else if any (< 0) (drop 1 row) then Just "negative file, line or tok" else Nothing
  nearWhy row = if any (< 0) row then Just "negative file or line" else Nothing
  width4 msg check row = if length row /= 4 then Just msg else check row
  head' = foldr const 0
  evens xs = [x | (i, x) <- zip [0 :: Int ..] xs, even i]

-- | The reply fields (pairs, raw, bandGroups, counts, degraded,
-- reason), or Left the refusal's message stem.
expected :: Req -> Either String [Maybe Value]
expected r
  | overCap = Right (answerOf r [] [] [] (replicate 13 0) True)
  | Just why <- offence r = Left why
  | otherwise = Right (answerOf r (M.toList finalBits) rawRows (M.toList bucketSizes) tallies False)
 where
  overCap =
    toInteger (length (rUnits r)) > candidateUnitCap
      || toInteger (sum (map length (rSigs r))) > candidateSigCap
      || toInteger (length (rPrints r)) > candidatePrintCap
      || toInteger (length (rNear r)) > candidateNearCap
  us = M.fromList (zip [0 ..] (map readUnit (rUnits r)))
  unit i = us M.! i
  owner f l = case [(i, u) | (i, u) <- M.toList us, uFile u == f, uFrom u <= l, l <= uTo u] of
    [] -> Nothing
    cs -> Just (fst (minimumBy (comparing (\(i, u) -> (uTo u - uFrom u, i))) cs))
  anchor (f1, l1) (f2, l2) = (,) <$> owner f1 l1 <*> owner f2 l2
  printAt = M.fromList (zip [0 :: Int ..] (rPrints r))
  hashGroups = filter ((> 1) . length) (M.elems (M.fromListWith (flip (<>)) [(h, [i]) | (i, h : _) <- M.toList printAt]))
  ordered xs = if hot xs then sortOn (\i -> let row = printAt M.! i in (row !! 1, row !! 3)) xs else xs
  lineOf i = let row = printAt M.! i in (row !! 1, row !! 2)
  nearEvents = [anchor (fa, la) (fb, lb) | [fa, la, fb, lb] <- rNear r]
  printEvents = [anchor (lineOf a) (lineOf b) | g <- hashGroups, (a, b) <- pairsIn (ordered g)]
  buckets = filter ((> 1) . length) (M.elems (M.fromListWith (flip (<>)) [(k, [i]) | (i, s) <- zip [0 ..] (rSigs r), k <- bandKeysOf s]))
  bucketSizes = M.fromListWith (+) [(toInteger (length b), 1 :: Integer) | b <- buckets]
  bandEvents = [Just ab | b <- buckets, ab <- pairsIn b]
  source evs = (pairsBy, byLangOf pairsBy, tally isCross, tally (== Nothing), tally isSelf)
   where
    good = [(min a b, max a b) | Just (a, b) <- evs, a /= b, uLang (unit a) == uLang (unit b)]
    pairsBy = M.fromList [(p, ()) | p <- good]
    isCross e = case e of Just (a, b) -> a /= b && uLang (unit a) /= uLang (unit b); _ -> False
    isSelf e = case e of Just (a, b) -> a == b; _ -> False
    tally p = toInteger (length (filter p evs))
  byLangOf ps = M.fromListWith (+) [(uLang (unit a), 1 :: Integer) | (a, _) <- M.keys ps]
  srcs = [(0, 1, source nearEvents), (2, 4, source printEvents), (3, 8, source bandEvents)]
  sent = M.unionsWith (.|.) [M.map (const bit) ps | (_, bit, (ps, _, _, _, _)) <- srcs]
  sameKey = [(i, j) | (i, x) <- M.toList us, (j, y) <- M.toList us, i < j, uKey x == uKey y, uFile x /= uFile y]
  (sameLang, crossed) = (filter sameL sameKey, filter (not . sameL) sameKey)
  sameL (i, j) = uLang (unit i) == uLang (unit j)
  unionBits = M.unionWith (.|.) sent (M.fromList [(p, 2) | p <- sameLang])
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
  finalBits = if rEx r then M.union keptBits (M.fromList [(p, 16) | p <- newS5]) else keptBits
  s2Lang = M.fromListWith (+) [(uLang (unit i), 1 :: Integer) | (i, _) <- sameLang]
  rawRows = [[s, l, k] | (s, perLang) <- sortOn fst ((1, s2Lang) : [(s, bl) | (s, _, (_, bl, _, _, _)) <- srcs]), (l, k) <- M.toList perLang]
  total f = sum [f t | (_, _, t) <- srcs]
  count v = toInteger (M.size (M.filter (== v) judged))
  s5 xs = if rEx r then toInteger (length xs) else 0
  tallies =
    [ total (\(_, _, _, o, _) -> o)
    , total (\(_, _, _, _, s) -> s)
    , toInteger (length (filter hot hashGroups))
    , toInteger (length (filter hot buckets))
    , toInteger (M.size unionBits)
    , toInteger (length crossed) + total (\(_, _, c, _, _) -> c)
    , count 1
    , count 2
    , count 0
    , s5 window
    , s5 cutS5
    , s5 already
    , s5 newS5
    ]

-- | The six reply fields from the kept rows, the raw per-source rows,
-- the band-group sizes, the thirteen tallies past the three input
-- counts, and the degraded flag.
answerOf :: Req -> [((Integer, Integer), Integer)] -> [[Integer]] -> [(Integer, Integer)] -> [Integer] -> Bool -> [Maybe Value]
answerOf r rows raws sizes ns deg =
  [ Just (toJSON [[a, b, s] | ((a, b), s) <- rows])
  , Just (toJSON raws)
  , Just (toJSON [[z, k] | (z, k) <- sizes])
  , Just (object (zipWith (.=) names (toInteger (length (rUnits r)) : toInteger (length (rPrints r)) : toInteger (length (rNear r)) : ns)))
  , Just (Bool deg)
  , if deg then Just (String "candidates_too_large") else Nothing
  ]
 where
  names =
    [ "units", "prints", "near", "unowned", "selfPairs", "printHot", "bandHot", "union", "crossLanguage"
    , "prunedSize", "prunedLabel", "survivors", "s5Windowed", "s5PrunedLabel", "s5Already", "s5New"
    ]

-- | Two hundred seeded requests: two to nine units in one or two
-- languages, three files and three keys (so S2 finds, crosses
-- languages and meets the other sources), spans inside lines 1..12
-- (so anchors nest, miss and tie), node counts 20..67 (so the 85/100
-- window both admits and cuts), histograms over five kinds, sets drawn
-- from six small values (so bands collide), fingerprint instances over
-- three hashes (now and then all on one hash, a group past the hot
-- cap), and a few
-- near runs.
requests :: [Req]
requests = [runG request (S (n * 5003 + 41) 0 0 0) | n <- [1 .. 200 :: Int]]

request :: G Req
request = do
  ex <- rand 3
  size <- (+ 2) <$> rand 8
  units <- mapM (const unitG) [1 .. size]
  sigs <- mapM (const setG) [1 .. size]
  hotRun <- rand 4
  nPrints <- (if hotRun == 0 then (+ 66) else id) <$> rand 9
  prints <- mapM (const (printG (if hotRun == 0 then 1 else 3))) [1 .. nPrints]
  nNear <- rand 4
  near <- mapM (const nearG) [1 .. nNear]
  pure (Req (ex /= 0) units sigs prints near)

unitG :: G [Integer]
unitG = do
  l <- toInteger <$> rand 2
  f <- toInteger <$> rand 3
  k <- toInteger <$> rand 3
  a <- toInteger . (+ 1) <$> rand 8
  w <- toInteger <$> rand 5
  counts <- mapM (const (toInteger <$> rand 12)) [1 .. 4 :: Int]
  pad <- toInteger <$> rand 4
  let hist = [(kind, c) | (kind, c) <- zip [0 ..] counts, c > 0] <> [(9, pad + 20)]
  pure ([l, f, k, sum (map snd hist), a, a + w] <> concat [[kind, c] | (kind, c) <- hist])

setG :: G [Integer]
setG = do
  picks <- mapM (const (rand 6)) [1 .. 3 :: Int]
  pure (M.keys (M.fromList [(toInteger p * 1000003 + 17, ()) | p <- picks]))

printG :: Int -> G [Integer]
printG hashes = do
  h <- rand hashes
  f <- rand 3
  l <- rand 12
  t <- rand 40
  pure [toInteger h * 7919 + 2 ^ (63 :: Int), toInteger f, toInteger l + 1, toInteger t]

nearG :: G [Integer]
nearG = do
  cells' <- mapM (const (rand 12)) [1 .. 4 :: Int]
  pure (zipWith (\k c -> if even (k :: Int) then toInteger (c `mod` 3) else toInteger c + 1) [0 ..] cells')
