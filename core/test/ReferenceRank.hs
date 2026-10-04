-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The advisor ranking's second implementation (plan v2.33 W3), spelled
-- the way the measuring side spelled it before the move: the log2 by
-- `until` and a fold over the fraction bits, the contribution as the
-- expanded integer fraction 22·tf·avg / (10·tf·avg + 3·avg + 9·len)
-- (k1 = 6/5 and b = 3/4 cleared by hand, not the core's rationals), the
-- expansion as a fold carrying the added list, the ranking as one sortOn
-- over a Data.Map of per-seat sums. It answers the reply fields
-- `rank.result` shows, and draws two hundred seeded small corpora.
module ReferenceRank (RReq (..), encodeReq, expected, requests) where

import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (sortOn)
import qualified Data.Map.Strict as M
import Data.Ord (Down (..))
import ReferenceFlowGen (G, S (..), rand, runG)

-- | One request as the battery builds it.
data RReq = RReq
  { rN, rAvg, rK :: Integer
  , rExclude :: Maybe Integer
  , rSeen :: [Integer]
  , rQuery :: [[Integer]]
  , rWiden :: Bool
  , rWords, rPairs, rTerms, rPostings, rLens :: [[Integer]]
  }

encodeReq :: RReq -> Value
encodeReq r =
  object $
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("rank.request" :: String)
    , "id" .= (1 :: Int)
    , "n" .= rN r
    , "avg" .= rAvg r
    , "k" .= rK r
    , "seen" .= rSeen r
    , "query" .= rQuery r
    , "widen" .= rWiden r
    , "words" .= rWords r
    , "pairs" .= rPairs r
    , "terms" .= rTerms r
    , "postings" .= rPostings r
    , "lens" .= rLens r
    ]
      <> ["exclude" .= e | Just e <- [rExclude r]]

-- | floor (256 · log2 (num / den)), the operands halved together below
-- 2^62 before each squaring.
log2Ref :: Integer -> Integer -> Integer
log2Ref num den = whole * 256 + fst (foldl' step (0, (num, top)) [1 .. 8 :: Int])
 where
  whole = toInteger (length (takeWhile (\i -> num >= den * 2 ^ (i + 1)) [0 :: Integer ..]))
  top = den * 2 ^ whole
  step (acc, (n, d)) _ =
    let (n', d') = until (\(x, y) -> x < 2 ^ (62 :: Int) && y < 2 ^ (62 :: Int)) (\(x, y) -> (x `quot` 2, y `quot` 2)) (n, d)
        (sn, sd) = (n' * n', d' * d')
     in if sn >= 2 * sd then (2 * acc + 1, (sn, 2 * sd)) else (2 * acc, (sn, sd))

idfRef :: Integer -> Integer -> Integer
idfRef n df = if n <= 2 * df then 0 else log2Ref (2 * n + 1 - 2 * df) (2 * df + 1)

ppmiRef :: Integer -> Integer -> Integer -> Integer -> Integer
ppmiRef n nab na nb = if nab >= 2 && nab * n > na * nb then log2Ref (nab * n) (na * nb) else 0

contribRef :: Integer -> Integer -> Integer -> Integer -> Integer -> Integer
contribRef w idf tf len avg = (w * idf * 22 * tf * avg * 65536) `quot` (10 * tf * avg + 3 * avg + 9 * len)

weightOf :: Integer -> Integer
weightOf ch = 256 * maybe 1 id (lookup ch [(0, 3), (2, 2)])

-- | (term, channel, weight, spelled), the bare query then the expansion.
ranked :: RReq -> ([(Integer, Integer, Integer, Bool)], [(Integer, Integer, Integer, Bool)])
ranked r = (bareQ, widened)
 where
  bareQ = [(t, ch, tf * weightOf ch, True) | [t, ch, tf] <- rQuery r]
  marg = M.fromList [(t, m) | [t, m] <- rWords r]
  spelled = map (\(t, _, _, _) -> t) bareQ
  widened = if null (rWords r) then [] else reverse (foldl' widenOne [] bareQ)
  widenOne acc (a, ch, w, _)
    | ch `notElem` [0, 2, 3] = acc
    | otherwise = foldl' (\xs (b, p) -> if b `elem` spelled || any (\(x, _, _, _) -> x == b) xs then xs else (b, ch, w * min p 1024 `quot` 2048, False) : xs) acc (near a)
  near a =
    let na = M.findWithDefault 0 a marg
        cands = [(b, p) | [a', b, nab, nb] <- rPairs r, a' == a, let p = ppmiRef (rN r) nab na nb, p >= 512]
     in if na == 0 || 4 * na > rN r then [] else take 3 (sortOn (\(b, p) -> (Down p, b)) cands)

-- | The reply fields [hits, added, query, counts, degraded].
expected :: RReq -> [Maybe Value]
expected r =
  [ Just (toJSON [s : sc `quot` 65536 : sc : h | (s, sc, h) <- kept])
  , Just (toJSON [[t, ch] | (t, ch, _, _) <- extra])
  , Just (toJSON (M.toList (M.fromListWith (+) [(t, w) | (t, _, w, _) <- q])))
  , Just (object ["rows" .= rows, "queryTerms" .= length q, "hits" .= length kept])
  , Just (toJSON False)
  ]
 where
  (bareQ, extra) = ranked r
  q = if rWiden r then bareQ <> extra else bareQ
  rows = length (rSeen r) + sum (map length [rQuery r, rWords r, rPairs r, rTerms r, rPostings r, rLens r])
  df = M.fromList [(t, d) | [t, d] <- rTerms r]
  len = M.fromList [(s, l) | [s, l] <- rLens r]
  per =
    M.fromListWith
      (\(a, x) (b, y) -> (a + b, zipWith (+) x y))
      [ (s, (contribRef w idf tf (len M.! s) (rAvg r), [if sp && c == ch then 1 else 0 | c <- [0 .. 5]]))
      | (t, ch, w, sp) <- q
      , let idf = idfRef (rN r) (M.findWithDefault 0 t df)
      , idf > 0
      , [t', s, tf] <- rPostings r
      , t' == t
      ]
  cut = take (fromInteger (rK r)) (sortOn (\(s, sc, _) -> (Down sc, s)) [(s, sc, h) | (s, (sc, h)) <- M.toList per, Just s /= rExclude r])
  kept = [x | x@(s, _, _) <- cut, s `notElem` rSeen r]

-- | Two hundred seeded corpora: two to twelve seats over eight terms of
-- random channels (hashes above 2^63, as fnv1a64 spells them), each
-- seat carrying a term with chance 1/4 at tf 1–3; every term's df and
-- postings, every seat's length; a random query, k, exclusion and seen
-- set; the words' counts and every co-occurrence when the request
-- carries them (always when widened).
requests :: [RReq]
requests = [runG request (S (n * 7919 + 13) 0 0 0) | n <- [1 .. 200 :: Int]]

request :: G RReq
request = do
  n <- (+ 2) <$> rand 11
  chans <- mapM (const (toInteger <$> rand 6)) vocab
  cells <- mapM (\_ -> mapM (\_ -> rand 4) vocab) [1 .. n]
  tfs <- mapM (\_ -> mapM (\_ -> (+ 1) <$> rand 3) vocab) [1 .. n]
  pads <- mapM (const (toInteger <$> rand 3)) [1 .. n]
  picks <- mapM (const ((,) <$> rand 2 <*> rand 2)) vocab
  k <- toInteger <$> rand 5
  ex <- rand 3
  exSeat <- toInteger <$> rand n
  seenBits <- mapM (const (rand 4)) [1 .. n]
  w <- rand 2
  c <- rand 2
  let seat = [0 .. toInteger n - 1]
      tc = zip vocab chans
      has s t = [tf | (s', row, tfRow) <- zip3 seat cells tfs, s' == s, (t', cell, tf) <- zip3 vocab row tfRow, t' == t, cell == 0]
      postings = [[t, s, toInteger tf] | t <- vocab, s <- seat, tf <- has s t]
      df t = toInteger (length [() | (t' : _) <- postings, t' == t])
      lens = [[s, sum [x | [_, s', x] <- postings, s' == s] + pad] | (s, pad) <- zip seat pads, any (\p -> p !! 1 == s) postings]
      query = [[t, ch, toInteger (a + 1)] | ((t, ch), (q, a)) <- zip tc picks, q == 0]
      wordTerms = [t | (t, ch) <- tc, ch `elem` [0, 2, 3]]
      both a b = toInteger (length [s | s <- seat, not (null (has s a)), not (null (has s b))])
      cooc = w == 1 || c == 1
      wordsRows = [[t, df t] | [t, ch, _] <- query, ch `elem` [0, 2, 3]]
      pairsRows = [[a, b, both a b, df b] | [a, _] <- wordsRows, b <- wordTerms, b /= a, both a b > 0]
      sumAvg = max 1 (sum [l | [_, l] <- lens] `quot` toInteger n)
  pure
    RReq
      { rN = toInteger n
      , rAvg = sumAvg
      , rK = k
      , rExclude = if ex == 0 then Just exSeat else Nothing
      , rSeen = [s | (s, bit) <- zip seat seenBits, bit == 0]
      , rQuery = query
      , rWiden = w == 1
      , rWords = if cooc then wordsRows else []
      , rPairs = if cooc then pairsRows else []
      , rTerms = [[t, df t] | t <- vocab]
      , rPostings = postings
      , rLens = lens
      }
 where
  vocab = [2 ^ (63 :: Int) + 7 * i | i <- [0 .. 7]]
