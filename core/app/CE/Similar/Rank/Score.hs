-- | The advisor's ranking (plan v2.33 W3; booklet 15 §3–§4): a query
-- bag weighted per channel, widened by its spelled words' top PPMI
-- neighbours, scored by integer BM25 over the postings the measuring
-- side fetched, and cut to the top k by score then seat (seat order is
-- identity order). A term in more than half the units (idf 0) is neither
-- score nor evidence; an expansion adds score but never evidence.
module CE.Similar.Rank.Score (QTerm (..), Hit (..), bare, expansion, rankTop) where

import CE.Similar.Rank.Cost (chanWeight, minCooc, minPpmi, neighbourDfRatio, ppmiCap, ppmiScale, topM, wUnit, wordChannel)
import CE.Similar.Rank.Math (contribution, idfFp, ppmiFp)
import qualified Data.IntMap.Strict as IM
import Data.List (sortBy)
import qualified Data.Map.Strict as M
import Data.Ord (Down (..), comparing)
import qualified Data.Set as S

-- | One query term: hash, channel, weight in 1/wUnit, and whether the
-- query spells it (an expansion does not).
data QTerm = QTerm {qTerm, qChan, qWeight :: !Integer, qSpelled :: !Bool}

-- | One kept candidate: its seat, its fixed-point score, and the
-- distinct spelled terms it shares per channel [N, P, C, D, S, L].
data Hit = Hit {hSeat :: !Int, hScore :: !Integer, hHits :: ![Integer]}

-- | The bare query from [term, channel, tf] rows: tf × channel weight
-- × wUnit.
bare :: [[Integer]] -> [QTerm]
bare rows = [QTerm t ch (tf * chanWeight ch * wUnit) True | [t, ch, tf] <- rows]

-- | The expansion of a query, in query order: each spelled word term's
-- top-m neighbours that the query does not spell and no earlier term
-- added, at its weight × min (ppmi, cap) / scale. `margs` are the
-- words' unit counts, `pairsBy a` the [b, nab, margB] rows of a in
-- ascending b.
expansion :: Integer -> [QTerm] -> M.Map Integer Integer -> M.Map Integer [[Integer]] -> [QTerm]
expansion n query margs pairsBy = go [q | q <- query, qSpelled q, wordChannel (qChan q)] S.empty
 where
  spelled = S.fromList (map qTerm query)
  go [] _ = []
  go (q : qs) added =
    let fresh = [(t, p) | (t, p) <- neighbours (qTerm q), S.notMember t spelled]
        new = dedupe added fresh
        terms = [QTerm t (qChan q) (qWeight q * min p ppmiCap `div` ppmiScale) False | (t, p) <- new]
     in terms <> go qs (foldr (S.insert . fst) added new)
  dedupe _ [] = []
  dedupe seen ((t, p) : rest)
    | S.member t seen = dedupe seen rest
    | otherwise = (t, p) : dedupe (S.insert t seen) rest
  neighbours a
    | na == 0 || neighbourDfRatio * na > n = []
    | otherwise =
        take topM . sortBy (comparing (Down . snd) <> comparing fst) $
          [ (b, p)
          | [b, nab, nb] <- M.findWithDefault [] a pairsBy
          , nab >= minCooc
          , let p = ppmiFp n nab na nb
          , p >= minPpmi
          ]
   where
    na = M.findWithDefault 0 a margs

-- | The top k by score descending then seat ascending, the excluded
-- seat out before the cut, the seen seats out after it.
rankTop ::
  Integer -> Integer -> Int -> Maybe Int -> S.Set Int -> M.Map Integer Integer -> M.Map Integer [(Int, Integer)] -> IM.IntMap Integer -> [QTerm] -> [Hit]
rankTop n avg k excl seenSeats dfOf postingsOf lenOf query =
  filter (\h -> S.notMember (hSeat h) seenSeats)
    . take k
    . sortBy (comparing (Down . hScore) <> comparing hSeat)
    $ [Hit s sc hs | (s, (sc, hs)) <- IM.toList acc, Just s /= excl]
 where
  acc = IM.fromListWith add [(s, one q idf s tf) | q <- query, let idf = idfFp n (M.findWithDefault 0 (qTerm q) dfOf), idf > 0, (s, tf) <- M.findWithDefault [] (qTerm q) postingsOf]
  one q idf s tf = (contribution (qWeight q) idf tf (IM.findWithDefault 0 s lenOf) avg, [if qSpelled q && toInteger c == qChan q then 1 else 0 | c <- [0 .. 5 :: Int]])
  add (a, x) (c, y) = (a + c, zipWith (+) x y)
