-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The advisor's ranking (plan v2.33 W3): the integer log2, idf and
-- contribution on the hand-computed values the measuring side's own
-- battery held before the move, the two fetch ratios pinned against the
-- formulas they shortcut, a ranking and a widening worked by hand, two
-- hundred seeded corpora agreeing with ReferenceRank on every reply
-- field, the contract's refusals by name, and the cap.
module RankProps (battery) where

import CE.Similar.Rank (respond)
import CE.Similar.Rank.Contract (RankReq (..), overCap)
import CE.Similar.Rank.Cost (b, k1, minPpmi, neighbourDfRatio, rankCap, scoreFracBits, scoredDfRatio)
import CE.Similar.Rank.Math (contribution, idfFp, log2Fp, ppmiFp)
import Data.Aeson (Value (..), toJSON)
import ReferenceContract (cells)
import ReferenceRank (RReq (..), encodeReq, expected, requests)
import WireHarness (fieldsOf, refusedBy, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "the integer log2 is exact on powers, equal on equal ratios, 149 at 1.5"
    , "the idf floors at zero for a term in half the units, and rarer is worth more"
    , "the contribution is the rational definition floored once"
    , "the two fetch ratios are the zero rules they shortcut"
    , "a ranking worked by hand: shared rare terms first, the query never itself"
    , "a widening worked by hand: fetch reaches load at two bits, a capped fraction"
    , "two hundred seeded corpora agree with the reference on every reply field"
    , "the seeded corpora are not vacuous: cuts, exclusions, seen seats, idf zero and widenings"
    , "the contract's refusals name the offender"
    , "the cap counts every table"
    ]
    [ log2Cases
    , idfCases
    , contributionCases
    , ratios
    , handRanking
    , handWidening
    , all agrees requests
    , nonVacuous
    , all refused refusals
    , not (overCap (sized rankCap)) && overCap (sized (rankCap + 1))
    ]

fields :: [String]
fields = ["hits", "added", "query", "counts", "degraded"]

agrees :: RReq -> Bool
agrees r = fieldsOf respond (encodeReq r) fields == Just (expected r)

log2Cases :: Bool
log2Cases =
  log2Fp 1 1 == 0
    && log2Fp 8 1 == 3 * 256
    && log2Fp 6 4 == log2Fp 3 2
    && log2Fp 3 2 == 149
    && log2Fp 3 2 < log2Fp 7 4
    && log2Fp 7 4 < log2Fp 2 1

idfCases :: Bool
idfCases = idfFp 10 6 == 0 && idfFp 10 1 > idfFp 10 3 && idfFp 0 0 == 0

-- | The closed form the core computes (22·tf·avg over
-- 10·tf·avg + 3·avg + 9·len, the 2^16 fixed point applied once) is the
-- rational definition (k1 + 1)·tf / (tf + k1·(1 − b + b·len/avg)) with
-- k1 = 6/5 and b = 3/4, floored once.
contributionCases :: Bool
contributionCases =
  k1 == 6 / 5
    && b == 3 / 4
    && and
      [ contribution w idf tf len avg == floor (fromInteger (w * idf * tf * 2 ^ scoreFracBits) * (k1 + 1) / (fromInteger tf + k1 * (1 - b + b * fromInteger len / fromInteger avg)))
      | (tf, len, avg) <- [(1, 10, 10), (3, 7, 20), (12, 300, 45), (1, 1, 1), (0, 5, 3), (7, 0, 2)]
      , (w, idf) <- [(1, 1), (768, 149), (256, 2047)]
      ]

-- | idf > 0 exactly when n > scoredDfRatio · df; and once
-- neighbourDfRatio · n_a > n no partner reaches minPpmi.
ratios :: Bool
ratios =
  and [(idfFp n df > 0) == (n > scoredDfRatio * df) | n <- [0 .. 60], df <- [0 .. n]]
    && and
      [ ppmiFp n nab na nb < minPpmi
      | n <- [1 .. 40]
      , na <- [1 .. n]
      , neighbourDfRatio * na > n
      , nab <- [1 .. na]
      , nb <- [nab .. n]
      ]

-- | Twelve seats: fetch_user (0) is the query — fetch (10), user (11)
-- names, query (12) callee, p:1 (13) shape; load_user (1) shares user,
-- query and p:1; user_name (3) shares user; the rest share nothing.
handRanking :: Bool
handRanking =
  fmap (take 1) (fieldsOf respond (encodeReq req) ["hits"])
    == Just [Just (toJSON [[1, sc1 `div` 65536, sc1, 1, 1, 1, 0, 0, 0], [3, sc3 `div` 65536, sc3, 1, 0, 0, 0, 0, 0]])]
 where
  req =
    RReq 12 1 3 (Just 0) [] [[10, 0, 1], [11, 0, 1], [12, 2, 1], [13, 1, 1]] False [] []
      [[10, 1], [11, 3], [12, 2], [13, 2]]
      [[10, 0, 1], [11, 0, 1], [11, 1, 1], [11, 3, 1], [12, 0, 1], [12, 1, 1], [13, 0, 1], [13, 1, 1]]
      [[0, 4], [1, 4], [3, 3]]
  c w df len = contribution w (idfFp 12 df) 1 len 1
  sc1 = c (3 * 256) 3 4 + c (2 * 256) 2 4 + c 256 2 4
  sc3 = c (3 * 256) 3 3

-- | Sixteen seats; fetch (1) and load (2) share four, fetch meets user,
-- post and item (3–5) once each: PPMI (fetch, load) = log2 (4·16 /
-- (4·4)) = two bits, the floor; the once-met words fall under minCooc.
-- load joins at 3·256 · 512 / 2048 = 192, unspelled.
handWidening :: Bool
handWidening =
  fieldsOf respond (encodeReq (req False)) ["added", "query"] == Just [Just (toJSON [[2 :: Integer, 0]]), Just (toJSON [[1 :: Integer, 768]])]
    && fieldsOf respond (encodeReq (req True)) ["query"] == Just [Just (toJSON [[1 :: Integer, 768], [2, 192]])]
 where
  req w =
    RReq 16 1 5 Nothing [] [[1, 0, 1]] w [[1, 4]] [[1, 2, 4, 4], [1, 3, 1, 2], [1, 4, 1, 2], [1, 5, 1, 2]]
      [[1, 4], [2, 4]]
      ([[1, s, 1] | s <- [0 .. 3]] <> [[2, s, 1] | s <- [0 .. 3]])
      [[s, 3] | s <- [0 .. 3]]

nonVacuous :: Bool
nonVacuous =
  any (\r -> rWiden r && grew r) requests
    && any (\r -> hitCount r == fromInteger (rK r) && rK r > 0) requests
    && any (\r -> maybe False (`elem` scored r) (rExclude r)) requests
    && any (\r -> any (`elem` rSeen r) (scored r) && hitCount r > 0) requests
    && any (\r -> or [d > 0 && rN r <= 2 * d | [_, d] <- rTerms r]) requests
 where
  grew r = case expected r of
    (_ : Just (Array a) : _) -> not (null a)
    _ -> False
  hitCount r = case expected r of
    (Just (Array hs) : _) -> length hs
    _ -> 0
  scored r = [s | [_, s, _] <- rPostings r]

-- | One case per line: the request's tables as overrides of a small
-- valid request, then the message stem.
refusals :: [(RReq, String)]
refusals = [(apply over, why) | [over, why] <- cells (unlines table)]
 where
  table =
    [ "n -1 ; n: negative"
    , "avg 0 ; avg: non-positive"
    , "k -1 ; k: negative"
    , "exclude 4 ; exclude: seat out of range"
    , "seen [[1],[0]] ; seen 1: not strictly ascending"
    , "query [[5,6,1]] ; query 0: channel out of range"
    , "query [[5,0,0]] ; query 0: non-positive tf"
    , "query [[6,0,1],[5,0,1]] ; query 1: not strictly ascending"
    , "terms [[5,5]] ; terms 0: df out of range"
    , "postings [[5,4,1]] ; postings 0: seat out of range"
    , "postings [[6,0,1]] ; postings: a term without a terms row"
    , "lens [] ; postings: a seat without a lens row"
    , "terms [[5,1],[6,1]] ; terms: a scoring term without exactly df postings"
    , "query [[5,0,1],[7,0,1]] ; query: a term without a terms row"
    , "pairs [[5,6,1,1]] ; words: a spelled word term without a words row"
    , "pairs [[5,6,2,1]] ; pairs 0: margB below nab"
    , "widen 1 ; widen: an expansion term without a terms row"
    ]
  apply over = case words over of
    ["n", v] -> base {rN = read v}
    ["avg", v] -> base {rAvg = read v}
    ["k", v] -> base {rK = read v}
    ["exclude", v] -> base {rExclude = Just (read v)}
    ["seen", v] -> base {rSeen = concat (read v :: [[Integer]])}
    ["query", v] -> base {rQuery = read v}
    ["terms", v] -> base {rTerms = read v}
    ["postings", v] -> base {rPostings = read v}
    ["lens", v] -> base {rLens = read v}
    ["pairs", v] -> base {rPairs = read v}
    ["widen", _] -> widening
    _ -> base
  base = RReq 4 1 2 Nothing [] [[5, 0, 1]] False [] [] [[5, 1]] [[5, 0, 1]] [[0, 2]]
  widening = RReq 16 1 2 Nothing [] [[1, 0, 1]] True [[1, 2]] [[1, 2, 2, 2]] [[1, 2]] [[1, 0, 1], [1, 1, 1]] [[0, 1], [1, 1]]

refused :: (RReq, String) -> Bool
refused (r, why) = refusedBy respond (encodeReq r) why

sized :: Integer -> RankReq
sized n = RankReq Null 1 1 0 Nothing (replicate (fromInteger n) 0) [] False [] [] [] [] []
