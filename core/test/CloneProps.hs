-- | The T3 property battery (design §4.3 R2 + §10 3e exit): over the
-- EXHAUSTIVE small-tree family, the production Zhang-Shasha equals
-- the mapping-definition brute force, the two admissible prune
-- bounds hold against brute-force ted, ted is a metric, and the
-- threshold knob is alive (perturbing it moves the verdict count,
-- with the nonemptiness precondition asserted — F16), and the
-- handler's `decide` rows (plan v2.33 W3: the verdict cache's replayed
-- rows) answer the same bit as the judging road. Since ADR-008
-- P1 every verdict assertion runs through Cost.cloneDecides — the
-- binding whose bit crosses the wire. CI walks n ≤ 4; CE_DEEP_TED=1
-- extends to n = 5 (nightly).
{-# LANGUAGE OverloadedStrings #-}

module CloneProps (battery) where

import CE.Clone (respond)
import CE.Clone.Cost (cloneDecides, cloneDecidesWith, tsedDen, tsedNum)
import CE.Clone.Prefilter (histo, provablyBelow)
import CE.Clone.Ted (Tree (..), ted)
import Data.Aeson (Value, decodeStrict, encode, object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import Data.Array.Unboxed (listArray)
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S
import ReferenceTed (family, labelInterOf, refTed)
import System.Environment (lookupEnv)

battery :: IO Bool
battery = do
  deep <- lookupEnv "CE_DEEP_TED"
  let maxN = if deep == Just "1" then 5 else 4
      fam = family maxN
      pairs = [(a, b) | a <- fam, b <- fam]
      teds = [(zs a b, a, b) | (a, b) <- pairs]
  let bad = [(x, y, v, refTed x y) | (v, x, y) <- teds, v /= refTed x y]
  a <- check "zhang-shasha ≡ mapping brute force (whole family)" (null bad)
  mapM_
    (\(x, y, z, r) -> putStrLn ("     first mismatch: " <> show (x, y, z, r)))
    (take 1 bad)
  b <- check "label bound: ref ted ≥ max − I" (all labelBound teds)
  c <- check "size bound: ref ted ≥ |n1 − n2|" (all sizeBound teds)
  d <- check "metric: identity and symmetry" (identitySym fam)
  e <- check "metric: triangle inequality" (triangle fam)
  f <- knobAlive teds
  -- the SHIPPED predicate, not just its math (M5-close review: the
  -- transcription had zero executed coverage — the bounds were
  -- asserted via ReferenceTed's own tallies while provablyBelow's
  -- one call site was production)
  g <- check "prefilter: provablyBelow implies not-a-clone under real ted" (all pruneOk teds)
  -- the ADR-008 P1 counterfactual: the SHIPPED verdict binding sits
  -- exactly on the threshold in both directions, and a perturbed
  -- knob flips the boundary pair — the bit the wire now carries is
  -- this formula's output, so this lever is the migration's proof
  h <-
    check
      "verdict boundary: ted 15/16 at max 100, and 86/100 flips it"
      ( cloneDecides 15 100 90
          && not (cloneDecides 16 100 90)
          && not (cloneDecidesWith (86, 100) 15 100 90)
      )
  i <- decideLegs teds
  putStrLn ("     clone family: maxN " <> show maxN <> ", trees " <> show (length fam))
  pure (a && b && c && d && e && f && g && h && i)

check :: String -> Bool -> IO Bool
check name ok = putStrLn ((if ok then "ok   " else "FAIL ") <> name) >> pure ok

type T = ([Int], [Int])

toTree :: T -> Tree
toTree (labs, llds) =
  Tree (listArray (0, length labs - 1) labs) (listArray (0, length llds - 1) llds) (length labs) (histo labs)

zs :: T -> T -> Integer
zs a b = ted (toTree a) (toTree b)

labelBound :: (Integer, T, T) -> Bool
labelBound (v, a, b) = v >= mx a b - fromIntegral (labelInterOf a b)

sizeBound :: (Integer, T, T) -> Bool
sizeBound (v, a, b) = v >= abs (size a - size b)

-- | Admissibility of the shipped prune against the real ted and the
-- SHIPPED verdict binding (ADR-008 P1: cloneDecides is the formula
-- whose bit crosses the wire — asserting through it, not a local
-- transcription, is what makes this admissibility executed coverage).
pruneOk :: (Integer, T, T) -> Bool
pruneOk (v, a, b) =
  not (provablyBelow (fst a) (fst b)) || not (cloneDecides v (size a) (size b))

size :: T -> Integer
size = fromIntegral . length . fst

mx :: T -> T -> Integer
mx a b = max (size a) (size b)

identitySym :: [T] -> Bool
identitySym fam =
  all (\a -> zs a a == 0) fam
    && and [zs a b == zs b a | (a : rest) <- tailsOf fam, b <- rest]
 where
  tailsOf xs = takeWhile (not . null) (iterate (drop 1) xs)

-- | Triangle over the family via a precomputed distance matrix —
-- the definition-side guarantee TSED consumers inherit.
triangle :: [T] -> Bool
triangle fam =
  and [d i k <= d i j + d j k | i <- idx, j <- idx, k <- idx]
 where
  idx = [0 .. length fam - 1]
  arr = IM.fromList [(i * length fam + j, zs (fam !! i) (fam !! j)) | i <- idx, j <- idx]
  d i j = arr IM.! (i * length fam + j)

-- | `decide` over every distinct (ted, sizes) of the family (one
-- request, under the pair cap): the reply's bits
-- are cloneDecides in row order; a request without the key carries
-- no `decided`; a malformed or negative row is refused by name.
decideLegs :: [(Integer, T, T)] -> IO Bool
decideLegs teds = do
  let rows = S.toList (S.fromList [[v, size a, size b] | (v, a, b) <- teds])
      want = [cloneDecides v n1 n2 | [v, n1, n2] <- rows]
      ask extra = respond "test" (BL.toStrict (encode (object (["id" .= (1 :: Int), "trees" .= ([] :: [Value]), "pairs" .= ([] :: [[Int]])] <> extra))))
      decided r = case r of
        Right bs -> decodeStrict bs >>= (KM.lookup "decided" :: KM.KeyMap Value -> Maybe Value)
        Left _ -> Nothing
      refused r = either (\(_, _, m) -> m) (const "") r
  a <- check "decide: bits = cloneDecides over the family" (decided (ask ["decide" .= rows]) == Just (toJSON want))
  b <- check "decide: absent key, no decided" (decided (ask []) == Nothing)
  c <- check "decide: negative ted refused" (refused (ask ["decide" .= [[-1, 2, 2 :: Integer]]]) == "decide 0: negative ted")
  d <- check "decide: short row refused" (refused (ask ["decide" .= [[1, 2 :: Integer]]]) == "decide 0: malformed row (need [ted,n1,n2])")
  pure (a && b && c && d && or want && not (and want))

-- | The clone verdict at a given ratio, counted over the family; the
-- production knob must separate from a perturbed one on a nonempty
-- flip set — a dead threshold cannot hide behind an empty family.
-- Counted through cloneDecidesWith: ONE formula (P1), so the probe
-- perturbs the production comparison, never a re-implementation.
knobAlive :: [(Integer, T, T)] -> IO Bool
knobAlive teds = do
  let count num den =
        length [() | (v, a, b) <- teds, cloneDecidesWith (num, den) v (size a) (size b)]
      (at85, at75) = (count tsedNum tsedDen, count 75 100)
  a <- check "knob nonemptiness: clones exist at 85/100" (at85 > 0)
  b <- check "knob alive: 85/100 vs 75/100 flips a nonempty set" (at75 > at85)
  pure (a && b)
