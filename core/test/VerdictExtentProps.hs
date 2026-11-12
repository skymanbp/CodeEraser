-- | The extent law (plan v2.35 score amendment), a part of the verdict
-- battery: production against ReferenceExtent (which shares nothing
-- with the fold), then the law's own statements - junk never helps
-- (T1), clean-up never hurts (T2), proportion (T3), the two ends of the
-- scale, monotone in every extent, the axis lattice, the weights, row
-- order, size-free copies, and the dead set read whole (the lua case;
-- the density law it replaced fell on that move, 813 -> 789 on lua).
module VerdictExtentProps (extentLegs) where

import CE.Verdict.Extent (extentOf)
import Data.List (subsequences)
import Data.Ratio (denominator, numerator, (%))
import ReferenceExtent
import VerdictExtentGen

extentLegs :: [(String, Bool)]
extentLegs =
  [ ("extent 1: production (respond and scored) = reference on every case, score, shares, E (tenths on the reply) and L", all agrees cases)
  , ("extent 1b: the reply's tenths lie in 0 .. 10 L on every case (E <= L)", all bounded cases)
  , ("extent 1c: an exact E whose numerator passes 2^64 (nineteen coprime c over the ceiling) leaves the reply as tenths", wide)
  , ("extent 2a T1: adding a file whose every line is named never raises the score", all junk trees)
  , ("extent 2b T2: deleting a file whose every line is named never lowers the score", all cleanup trees)
  , ("extent 3 T3: one edit moves the score by at most (|a| + |b|) / L of the scale, +1", all proportion trees)
  , ("extent 4: full scale iff E = 0; k E >= L reads 0; 0 only when scale (L - k E) < L", all ends cases)
  , ("extent 5: a span, a dead flag, a cycle line, any higher c (across the ceiling, any span shape), a lower soft line never raise it", all monotone trees)
  , ("extent 5, named cases: a parent crossing over an over-ceiling child at depth 1-3, a shared boundary line, three functions", and namedCases)
  , ("extent 11, named: a prose-only file has no size term; a self reference counts only at cycle floor 1", and fileCases)
  , ("extent 12, named: a class's own soft line and ceiling replace the tree's (placed and scan-only files)", and classCases)
  , ("extent 6: E(A) <= E(B) <= sum over B of E({i}) for every A within B", all lattice randomFamily)
  , ("extent 7: equal weights = unweighted; weight 0 = axis absent; layers = per-line maximum ratio", all weighted trees)
  , ("extent 8: row order of every table is not a judgment fact", all permuted trees && all rowsReversed trees)
  , ("extent 9: two disjoint copies of a tree read what one copy reads", all copies trees)
  , ("extent 10: with a dead root that alone refers to a dead file, deleting the root never lowers the score", all deadRoot trees)
  ]
 where
  trees = exhaustive <> randomFamily
  -- the cases that broke the first two texts of the rule: the parent
  -- crossing its ceiling over an over-ceiling child (outermost-only
  -- reading), [1,2] crossing next to [2,4] that shared its line 2 and a
  -- three-function overlap (forest of the over-ceiling functions only)
  namedCases =
    [jScore (eq (crossing c d)) <= jScore (eq (crossing 11 d)) | c <- [16, 40], d <- [1, 2, 3]]
      <> [jScore (eq (sharing c)) <= jScore (eq (sharing 10)) | c <- [16, 20, 40]]
      <> [jScore (eq (threeway c)) <= jScore (eq (threeway 10)) | c <- [16, 40]]
  sharing c = MTree [(blank 4) {mFuncs = [(1, 2, c), (2, 4, 30)], mSpans = [(1, 1, 0)]}] [] 1000 15 2
  threeway c = MTree [(blank 8) {mFuncs = [(1, 5, 16), (2, 7, c), (3, 4, 25)], mSpans = [(3, 3, 0)]}] [] 1000 15 2
  crossing c d = MTree [(blank 10) {mFuncs = (1, 10, c) : take d [(3, 7, 30), (4, 6, 20), (5, 5, 50)]}] [] 1000 15 2
  -- (tree, E): a 900-line prose-only file with one 10-line duplicate
  -- span under soft line 300 reads the span, and its size share is 0;
  -- a member of a 2-cycle referring to its partner (line 2) and to
  -- itself (line 4); a singleton referring to itself (line 3)
  fileCases =
    [agrees (0, t) && jE (eq t) == e | (t, e) <- [(prose, 10), (pair 2, 1), (pair 1, 2), (single 2, 0), (single 1, 1)]]
      <> [take 1 (jShares (eq prose)) == [0]]
  prose = MTree [(blank 900) {mProse = True, mSpans = [(10, 19, 2)]}] [] 300 15 2
  pair = MTree [(blank 5) {mScc = Just 0}, (blank 5) {mScc = Just 0}] [(0, 1, 2), (0, 0, 4)] 1000 15
  single = MTree [blank 5] [(0, 0, 3)] 1000 15
  -- (tree, E): 400 lines under a class soft line of 350 in a tree at 300
  -- (placed, then scan-only): 50, not 100; a 10-line function at c 20
  -- under a class ceiling of 25 in a tree at 15: 0; under 10: m / 2
  classCases = [agrees (0, t) && jE (eq t) == e | (t, e) <- [(soft False, 50), (soft True, 50), (classCeil 25, 0), (classCeil 10, 5)]]
  soft scanOnly = MTree [(blank 400) {mPlaced = not scanOnly, mClass = Just (350, 15)}] [] 300 15 2
  classCeil c = MTree [(blank 10) {mFuncs = [(1, 10, 20)], mClass = Just (1000, c)}] [] 1000 15 2
  cases = zip [0 :: Int ..] (exhaustive <> randomFamily)

weightSets :: [[Integer]]
weightSets = [replicate 7 1, [2, 1, 3, 0, 1, 1, 2], [0, 1, 1, 1, 1, 1, 1], replicate 7 5, [1, 0, 2, 2, 0, 3, 1]]

costs :: [Integer]
costs = [10, 25, 4]

pickFor :: Int -> ([Integer], Integer)
pickFor i = (weightSets !! (i `mod` 5), costs !! (i `mod` 3))

agrees :: (Int, MTree) -> Bool
agrees (i, t) = viaRespond ws v t == Just want && same (viaScored ws v t)
 where
  (ws, v) = pickFor i
  (s, e, l) = refScore ws (v % 10) t
  want = Replied s (refShares t) (halfUp e) l
  same j = jScore j == s && jShares j == refShares t && jE j == e && jL j == l

-- | E times ten rounded half up, spelled on the exact fraction's two
-- integers (the production rounding is a floor of 10 E + 1/2).
halfUp :: Rational -> Integer
halfUp e = (20 * numerator e + denominator e) `quot` (2 * denominator e)

bounded :: (Int, MTree) -> Bool
bounded (i, t) = maybe False (\r -> rTenths r >= 0 && rTenths r <= 10 * rL r) (viaRespond ws v t)
 where
  (ws, v) = pickFor i

-- | One file of nineteen one-line functions at the primes 17 .. 97
-- over a ceiling of 15: E = sum (1 - 15 / p), its reduced denominator
-- the product of the primes (about 2^94). No churn rides.
wide :: Bool
wide = numerator e > 2 ^ (64 :: Int) && agrees (0, t)
 where
  primes = [17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97]
  t = MTree [(blank 19) {mFuncs = [(j, j, p) | (j, p) <- zip [1 ..] primes]}] [] 1000 15 2
  (_, e, _) = refScore ws (v % 10) t
  (ws, v) = pickFor 0

eq :: MTree -> Judged
eq = viaScored (replicate 7 1) 10

-- | A placed code file all of whose lines are named: dead, or one
-- whole-file duplicate.
named :: [MFile]
named = [(blank n) {mDead = True} | n <- [1, 4, 9]] <> [(blank n) {mSpans = [(1, n, k)]} | n <- [2, 7], k <- [0, 2]]

withFile :: MTree -> MFile -> MTree
withFile t f = t {tFiles = tFiles t <> [f]}

junk :: MTree -> Bool
junk t = and [jScore (eq (withFile t f)) <= jScore (eq t) | f <- named]

-- | The tree without file i: its references go, later positions close up.
without :: Int -> MTree -> MTree
without i t = t {tFiles = take i fs <> drop (i + 1) fs, tRefs = [(down s, down d, l) | (s, d, l) <- tRefs t, s /= i, d /= i]}
 where
  fs = tFiles t
  down x = if x > i then x - 1 else x

cleanup :: MTree -> Bool
cleanup t = and [jScore (eq (without i t)) >= jScore (eq t) | (i, f) <- zip [0 ..] (tFiles t), fullyNamed f]
 where
  fullyNamed f = mPlaced f && not (mProse f) && mDead f && mScc f == Nothing

-- | Single edits: a span added to a placed code file, its lines grown by 3.
edits :: MTree -> [MTree]
edits t =
  [ t {tFiles = [if j == i then g else h | (j, h) <- zip [0 ..] (tFiles t)]}
  | (i, f) <- zip [0 :: Int ..] (tFiles t)
  , mPlaced f
  , not (mProse f)
  , g <- [f {mSpans = (1, mLines f, 0) : mSpans f}, f {mLines = mLines f + 3}]
  ]

proportion :: MTree -> Bool
proportion t = and [step (eq t) (eq t') | t' <- edits t]
 where
  step a b = abs (jScore a - jScore b) % 1 <= 1000 * (abs (jE b - jE a) + abs (jL b - jL a) % 1) / (max (jL a) (jL b) % 1) + 1

ends :: (Int, MTree) -> Bool
ends (i, t) = (e == 0) == (s == 1000) && (l == 0 || ((k * e < l % 1 || s == 0) && (s /= 0 || 1000 * (l % 1 - k * e) < l % 1)))
 where
  (ws, v) = pickFor i
  k = v % 10
  Judged s _ e l = viaScored ws v t

-- | Moves at fixed L that only add to an extent: a span, a dead flag, a
-- higher c, a cycle line, a lower soft line, more rewritten lines.
monotone :: MTree -> Bool
monotone t = and [jScore (eq t') <= jScore (eq t) | t' <- fileMoves t <> treeMoves t]

fileMoves :: MTree -> [MTree]
fileMoves t = [t {tFiles = [if j == i then g else h | (j, h) <- zip [0 ..] fs]} | (i, f) <- zip [0 :: Int ..] fs, mPlaced f, not (mProse f), g <- moves f]
 where
  fs = tFiles t
  moves f =
    [f {mSpans = (1, 1, 2) : mSpans f}, f {mDead = True}]
      <> [f {mFuncs = [(lo, hi, if j == k then c + d else c) | (j, (lo, hi, c)) <- zip [0 :: Int ..] (mFuncs f)]} | k <- [0 .. length (mFuncs f) - 1], d <- [5, 30]]
      -- churn: more rewritten lines at the same appended ones, and a
      -- file crossing the rewrite threshold from below (at it, above it)
      <> [f {mChurn = Just ch} | Just (rw, ap) <- [mChurn f], ch <- [(rw + 1, ap), (rw + 3, ap)] <> [(rw', ap) | 2 * rw < rw + ap, rw' <- [ap, ap + 2]]]

treeMoves :: MTree -> [MTree]
treeMoves t =
  [t {tRefs = (i, d, l) : tRefs t} | i <- [0 .. length fs - 1], cycleMember t i, d <- [0 .. length fs - 1], d /= i, mScc (fs !! d) == mScc (fs !! i), code (fs !! d), l <- [1, mLines (fs !! i)]]
    <> [t {tSoft = tSoft t - 1} | tSoft t > 1]
 where
  fs = tFiles t
  code f = mPlaced f && not (mProse f)

lattice :: MTree -> Bool
lattice t = and [ex a <= ex b && ex b <= sum [ex [x] | x <- b] | b <- subsequences [0 .. 6], a <- subsequences b, length b <= 3 || length a == length b - 1]
 where
  us = unitsOf t
  ex as = extentOf (`elem` as) us

weighted :: MTree -> Bool
weighted t =
  jE (viaScored (replicate 7 3) 10 t) == jE (eq t)
    && and [jE (viaScored [if a == i then 0 else 1 | a <- [0 .. 6]] 10 t) == extentOf (/= i) (unitsOf t) | i <- [0 .. 6]]
    && and [jE (viaScored ws 10 plain) == refLineMax ws plain | ws <- weightSets]
 where
  plain = t {tFiles = [f {mFuncs = [], mChurn = Nothing, mClass = Nothing} | f <- tFiles t], tSoft = 1000}

-- | The same tree with every list reversed: files, spans, functions,
-- references (positions remapped), so every table's rows arrive in
-- another order to the fold.
permuted :: MTree -> Bool
permuted t = eq t == eq t'
 where
  n = length (tFiles t)
  t' = t {tFiles = reverse [f {mSpans = reverse (mSpans f), mFuncs = reverse (mFuncs f)} | f <- tFiles t], tRefs = reverse [(n - 1 - s, n - 1 - d, l) | (s, d, l) <- tRefs t]}

copies :: MTree -> Bool
copies t = jScore a == jScore b && jShares a == jShares b && (jL a == 0 || jE a / (jL a % 1) == jE b / (jL b % 1))
 where
  n = length (tFiles t)
  twin = [f {mScc = fmap (+ 1000) (mScc f)} | f <- tFiles t]
  a = eq t
  b = eq t {tFiles = tFiles t <> twin, tRefs = tRefs t <> [(s + n, d + n, l) | (s, d, l) <- tRefs t]}

-- | A dead root A (lines a) referring to a dead file B, appended to the
-- tree; deleting A.
rootCase :: MTree -> [(MTree, MTree)]
rootCase t = [(with, without (n + 0) with) | (a, b) <- [(3, 5), (8, 2)], let with = add a b]
 where
  n = length (tFiles t)
  add a b = t {tFiles = tFiles t <> [(blank a) {mDead = True}, (blank b) {mDead = True}], tRefs = tRefs t <> [(n, n + 1, 1)]}

deadRoot :: MTree -> Bool
deadRoot t = and [jScore (eq after) >= jScore (eq before) | (before, after) <- rootCase t]
