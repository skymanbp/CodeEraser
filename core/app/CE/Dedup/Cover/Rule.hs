-- | The coverage rule C_t (v2.35 imports lane, R2): which verified
-- token runs of one file pair the T1/T2 report keeps. Replaces the
-- measuring side's line-containment pass (`dominant`, cli/src/dedup/
-- pairs.rs until v2.35), which compared LINE spans: a shorter run
-- whose lines sat inside a longer one vanished even when most of its
-- tokens were its own, and a low-diversity run could hide a diverse
-- one. This rule counts TOKENS and orders by diversity first.
--
-- Per file pair, runs r = (a0, b0, len, distinct), Ia(r) = [a0, a0+len)
-- in the first file, Ib(r) = [b0, b0+len) in the second, t the report
-- threshold, d the diversity floor:
--
--   1. two classes, D (distinct >= d) then Lo (the rest);
--   2. inside a class: len descending, then a0, then b0 ascending;
--   3. a cover per file of the pair — ONE shared cover when the two
--      files are one — empty at the start of the pair;
--   4. na = len - |Ia(r) ∩ cover(a)|, nb = len - |Ib(r) ∩ cover(b)|:
--      the run is kept iff max(na, nb) >= t, and a kept run joins the
--      cover on both sides;
--   5. Lo is folded after D, against the cover D left.
--
-- A kept D run is reported, a kept Lo run is suppressed (counted,
-- never reported), the rest is dropped. So every run of a pair either
-- is kept or has fewer than t tokens outside the cover on each side
-- (completeness), and every kept run had t tokens of its own on some
-- side when it joined (non-redundancy); a low-diversity run never
-- hides a diverse one.
module CE.Dedup.Cover.Rule (Run (..), Class (..), Cover, classify, cover, covered) where

import Data.Function (on)
import Data.List (groupBy, partition, sortOn)
import qualified Data.Map.Strict as M
import Data.Ord (Down (..))

-- | One verified run: the two files' codes, the two token offsets, its
-- length and its distinct token count.
data Run = Run {runA, runB, runA0, runB0, runLen, runDistinct :: Integer}

data Class = Reported | Suppressed | Dropped
  deriving (Eq, Show)

-- | One file's covered tokens: disjoint, non-touching half-open
-- intervals, start to end — an ordered map, so a query and an insert
-- cost a logarithm plus the intervals they meet, never a list walk.
type Cover = M.Map Integer Integer

-- | How many tokens of [s, e) the cover holds.
covered :: Cover -> Integer -> Integer -> Integer
covered c s e = sum [min e hi - max s lo | (lo, hi) <- before <> inside, hi > s, lo < e]
 where
  before = maybe [] pure (M.lookupLT s c)
  inside = M.toAscList (M.takeWhileAntitone (< e) (M.dropWhileAntitone (< s) c))

-- | The cover with [s, e) added: every interval it overlaps or touches
-- merges into one.
cover :: Integer -> Integer -> Cover -> Cover
cover s e c = M.insert s' e' (M.union lower upper)
 where
  left = case M.lookupLE s c of
    Just (lo, hi) | hi >= s -> Just (lo, hi)
    _ -> Nothing
  s' = maybe s fst left
  (lower, rest) = M.split s' c
  (absorbed, upper) = M.spanAntitone (<= e) rest
  e' = maximum (e : maybe [] (pure . snd) left <> M.elems absorbed)

-- | Every run's class, in request order. The runs arrive grouped by
-- pair (the contract's ascending order puts a pair's runs together);
-- each pair is judged on its own covers.
classify :: Integer -> Integer -> [Run] -> [Class]
classify t d runs = map snd (sortOn fst (concatMap (pairOf t d) pairs))
 where
  pairs = groupBy ((==) `on` (key . snd)) (zip [0 :: Int ..] runs)
  key r = (runA r, runB r)

-- | One pair's runs: D then Lo, each in the rule's order, folded
-- through the covers.
pairOf :: Integer -> Integer -> [(Int, Run)] -> [(Int, Class)]
pairOf t d runs = go (M.empty, M.empty) (order diverse <> order low)
 where
  (diverse, low) = partition ((>= d) . runDistinct . snd) runs
  order = sortOn (\(_, r) -> (Down (runLen r), runA0 r, runB0 r))
  same = any (\(_, r) -> runA r == runB r) runs
  go _ [] = []
  go covers@(ca, cb) ((i, r) : rest)
    | max na nb >= t = (i, kept) : go covers' rest
    | otherwise = (i, Dropped) : go covers rest
   where
    (a0, b0, n) = (runA0 r, runB0 r, runLen r)
    kept = if runDistinct r >= d then Reported else Suppressed
    cb' = if same then ca else cb
    na = n - covered ca a0 (a0 + n)
    nb = n - covered cb' b0 (b0 + n)
    covers'
      | same = let c = cover b0 (b0 + n) (cover a0 (a0 + n) ca) in (c, c)
      | otherwise = (cover a0 (a0 + n) ca, cover b0 (b0 + n) cb)
