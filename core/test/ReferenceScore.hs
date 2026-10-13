-- | The verdict family's score, second implementation (plan v2.32
-- step 7, authority-track §7): booklet 05 read as definitions. Each
-- axis's charge is the integer quotient scale·a / (a + n·b) of the
-- mass a/b over its opportunity n (the density law floor(scale ·
-- v/(v+n)) with the fraction cleared by hand); the soft-zone curve
-- is one fraction over a common denominator; the soft line's two
-- log-space middles are ORDER STATISTICS found by counting, never by
-- sorting, and its fourth root a bisection over the integers; the
-- touched-file masses are list sets (nub), the class lines list
-- lookups over the raw wire rows. Only the ScoreKnobs record — the
-- entry's own parameter — and the soft-line clamp constants are read.
module ReferenceScore (Inputs (..), refCharges, refScore, refSoftLine, refZone) where

import CE.Verdict.Score (ScoreKnobs (..))
import Data.List (nub)
import Data.Ratio (denominator, numerator, (%))

-- | The fact tables as they ride the wire: sim [u,v,kind,num,den],
-- pos [u,indeg,outdeg,scc,size,reach], churn [u,rewrite,append],
-- continuous [u,code,value(,class)], the doc-file indices, the class
-- knob rows [class,code,value] and the self-loop indices.
data Inputs = Inputs
  { iSim, iPos, iChurn, iCont :: [[Integer]]
  , iDocs :: [Integer]
  , iClassRows :: [[Integer]]
  , iLoops :: [Integer]
  }

-- | p(x) on the zone (s, h] as one fraction: pMax·(x−s)²/(h−s)²
-- inside, pMax·((h−s) + 2(x−h))/(h−s) past the wall, 0 at or under
-- s, a flat pMax when the wall does not stand above the line. The
-- line is never read above the wall (9.0.0): s past h prices as h.
refZone :: Integer -> Integer -> Integer -> Integer -> Rational
refZone line h pMax x
  | x <= s = 0
  | d <= 0 = fromInteger pMax
  | x > h = (pMax * (d + 2 * (x - h))) % d
  | otherwise = (pMax * (x - s) * (x - s)) % (d * d)
 where
  s = if line > h then h else line
  d = h - s

-- | The density law with the fraction cleared.
density :: Integer -> Rational -> Integer -> Integer
density scale v n
  | n <= 0 || v <= 0 = 0
  | otherwise = (scale * a) `div` (a + n * b)
 where
  a = numerator v
  b = denominator v

-- | A continuous row's class line for a code, the global line when the
-- class declares none.
classLine :: Inputs -> [Integer] -> Integer -> Integer -> Integer
classLine inp row code global = case lookup (cls, code) [((c, k), v) | [c, k, v] <- iClassRows inp] of
  Just v -> v
  Nothing -> global
 where
  cls = case drop 3 row of
    (c : _) -> c
    [] -> 0

-- | The seven axis charges in code order.
refCharges :: ScoreKnobs -> Maybe Integer -> Inputs -> [(Integer, Integer)]
refCharges k soft inp = zip [0 ..] (zipWith (density (sScoreScale k)) masses opportunities)
 where
  line = maybe (sSizeCeil k) id soft
  sizeRows = [r | r@(_ : 0 : _ : _) <- iCont inp]
  fnRows = [r | r@(_ : 1 : _ : _) <- iCont inp]
  value r = case drop 2 r of
    (v : _) -> v
    [] -> 0
  codeFiles = [u | (u : _) <- iPos inp, u `notElem` iDocs inp]
  verified kinds (num, den) = [[u, v] | [u, v, kind, n, d] <- iSim inp, kind `elem` kinds, n % d >= num % den]
  touched pairs = fromIntegral (length (nub (concat pairs)))
  heavy rw ap = rw + ap > 0 && rw % (rw + ap) >= sRewriteNum k % sRewriteDen k
  masses =
    [ sum [refZone (classLine inp r 0 line) (classLine inp r 2 (sSizeHard k)) (sSizePMax k) (value r) | r <- sizeRows]
    , fromIntegral (length [() | r <- fnRows, value r > classLine inp r 1 (sCocCeil k)])
    , touched (verified [0, 1] (sCloneNum k, sCloneDen k))
    , touched (verified [2] (sDupNum k, sDupDen k))
    , fromIntegral (length [() | [_, indeg, _, _, _, reach] <- iPos inp, reach == 0, indeg <= sDeadIndegCeil k])
    , fromIntegral (length [() | [_, rw, ap] <- iChurn inp, heavy rw ap])
    , fromIntegral (length [() | [u, _, _, _, size, _] <- iPos inp, u `elem` codeFiles, size >= sCycleFloor k, size > 1 || u `elem` iLoops inp])
    ]
  opportunities =
    map genericLength [sizeRows, fnRows]
      <> [genericLength codeFiles, genericLength (iDocs inp), genericLength (iPos inp), genericLength (iChurn inp), genericLength codeFiles]
  genericLength xs = toInteger (length xs)

-- | (per-mille, total charge): each axis's effective weight is the
-- first wire row naming it, else the default; the weighted mean of
-- the charges, scaled by the strictness dial over its neutral value.
refScore :: ScoreKnobs -> Integer -> [[Integer]] -> [(Integer, Integer)] -> (Integer, Integer)
refScore k neutral weights charges = (max 0 (sScoreScale k - spent `div` (neutral * totalWeight)), sum (map snd charges))
 where
  weightOf c = maybe (sDefaultWeight k) id (lookup c [(c', w) | [c', w] <- weights])
  totalWeight = sum [weightOf c | (c, _) <- charges]
  spent = sum [weightOf c * p | (c, p) <- charges] * sViolCost k

-- | S = clamp(floor(m·r^k), [lo, hi]) over the positive LOC values in
-- log space (9.0.0): m the log-space median of the values and r the
-- log-space median of every value's ratio to m folded to at least one
-- — an even count's middle is the geometric mean of its two middles —
-- read in fourth powers, m⁴ = (x_a·x_b)² and r⁴ = ρ_a²·ρ_b², so S is
-- the largest integer whose fourth power stays at or under m⁴·r^{4k};
-- Nothing when no value is positive.
refSoftLine :: Integer -> Integer -> Integer -> [Integer] -> Maybe Integer
refSoftLine kExp lo hi locs = case [fromInteger x | x <- locs, x > 0] of
  [] -> Nothing
  vs ->
    let mSq = logMiddleSquared vs
        r4 = logMiddleSquared [max (v * v / mSq) (mSq / (v * v)) | v <- vs]
     in Just (max lo (min hi (fourthRootFloor (mSq * mSq * r4 ^ kExp))))

-- | The square of a log-space median: the product of the two middle
-- order statistics (the one middle squared when the count is odd).
logMiddleSquared :: [Rational] -> Rational
logMiddleSquared vs = rankAt ((n - 1) `div` 2) vs * rankAt (n `div` 2) vs
 where
  n = length vs

-- | The largest integer s >= 0 with s⁴ <= q, by doubling then halving
-- the bracket [s, s + step).
fourthRootFloor :: Rational -> Integer
fourthRootFloor q = narrow 0 (until (\b -> fromInteger (b ^ (4 :: Int)) > q) (* 2) 1)
 where
  narrow s step
    | step <= 1 = s
    | fromInteger ((s + half) ^ (4 :: Int)) <= q = narrow (s + half) (step - half)
    | otherwise = narrow s half
   where
    half = step `div` 2

-- | The order statistic of rank j: the value with at most j values
-- strictly below it and more than j at or below it.
rankAt :: Int -> [Rational] -> Rational
rankAt j xs = case [x | x <- xs, length (filter (< x) xs) <= j, length (filter (<= x) xs) > j] of
  (x : _) -> x
  [] -> 0
