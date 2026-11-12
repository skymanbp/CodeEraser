-- | The extent law's second implementation (plan v2.35 score
-- amendment), for VerdictExtentProps. It reads a model tree, never a
-- request, and imports nothing from core/app: every file is a list of
-- per-line booleans (named or not, which
-- function owns the line), counted line by line - where the production
-- fold (CE.Verdict.Extent) merges intervals.
module ReferenceExtent
  ( MFile (..)
  , MTree (..)
  , blank
  , cycleMember
  , refExtent
  , refLayered
  , refLineMax
  , refLines
  , refScore
  , refShares
  ) where

import Data.Ratio ((%))
import Data.List (nub, sort, sortBy)

-- | One file of a model tree.
data MFile = MFile
  { mLines :: Integer
  , mProse :: Bool
  -- ^ a universe file with no size row (and no graph position)
  , mPlaced :: Bool
  -- ^ in the verdict universe; False = the scan-only arm (size only)
  , mDoc :: Bool
  , mDead :: Bool
  , mScc :: Maybe Integer
  -- ^ an import-cycle group (a member when two or more code files share it)
  , mFuncs :: [(Integer, Integer, Integer)]
  -- ^ (lo, hi, cognitive complexity), any spans
  , mChurn :: Maybe (Integer, Integer)
  -- ^ (rewritten, appended)
  , mSpans :: [(Integer, Integer, Integer)]
  -- ^ duplicate member spans (lo, hi, kind 0 / 1 clone, 2 docdup)
  , mClass :: Maybe (Integer, Integer)
  -- ^ a path class with its own (soft line, complexity ceiling); none on prose files
  }

-- | Files, reference lines (src, dst, line) by position in `tFiles`,
-- the soft line, the complexity ceiling and the cycle floor.
data MTree = MTree
  { tFiles :: [MFile]
  , tRefs :: [(Int, Int, Integer)]
  , tSoft :: Integer
  , tCeil :: Integer
  , tFloor :: Integer
  }

blank :: Integer -> MFile
blank n = MFile n False True False False Nothing [] Nothing [] Nothing

-- | A cycle member: a code file with a graph position whose group (a
-- file without one is a group of its own) holds at least the floor of
-- such files - a group of one only through a reference to itself.
cycleMember :: MTree -> Int -> Bool
cycleMember t i = positioned f && not (mDoc f) && size >= tFloor t && (size > 1 || or [s == i && d == i | (s, d, _) <- tRefs t])
 where
  f = tFiles t !! i
  size = toInteger (length [() | (j, h) <- zip [0 ..] (tFiles t), positioned h, group t j == group t i])

-- | The cycle group of file i: its own index when it has none.
group :: MTree -> Int -> Either Int Integer
group t i = maybe (Left i) Right (mScc (tFiles t !! i))

positioned :: MFile -> Bool
positioned f = mPlaced f && not (mProse f)

refLines :: MTree -> Integer
refLines = sum . map mLines . tFiles

-- | E with the axes `on` admits (codes 0 size .. 6 cycle).
refExtent :: (Integer -> Bool) -> MTree -> Rational
refExtent on t = sum [fileE on t i | i <- [0 .. length (tFiles t) - 1]]

fileE :: (Integer -> Bool) -> MTree -> Int -> Rational
fileE on t i = minimum [n % 1, maximum [located, sizeX, churnX]]
 where
  f = tFiles t !! i
  n = mLines f
  ls = [1 .. n]
  named = [lineNamed on t i l | l <- ls]
  roots = if on 1 && mPlaced f then descend f else []
  -- in(R) over R's lines: a named line no child holds counts 1; a child
  -- counts max(its excess, in(child))
  inR lineSet kids =
    toInteger (length [() | l <- lineSet, named !! fromInteger (l - 1), not (any (\(RNode a b _ _) -> a <= l && l <= b) kids)]) % 1
      + sum [max (toInteger (b - a + 1) % 1 * (if c > ceil then 1 - ceil % c else 0)) (inR [a .. b] sub) | RNode a b c sub <- kids]
  located = inR ls roots
  -- a prose-only file is not sized: no size term
  sizeX = if on 0 && not (mProse f) then max 0 (n - soft) % 1 else 0
  -- the file's class's own soft line and ceiling, the tree's without one
  (soft, ceil) = maybe (tSoft t, tCeil t) id (mClass f)
  churnX = if on 5 && positioned f then churnOf f else 0

-- | Named on line l of file i: a span of an admitted kind, the dead file,
-- or a reference line to a file of the same cycle group (to itself only
-- at a floor of 1).
lineNamed :: (Integer -> Bool) -> MTree -> Int -> Integer -> Bool
lineNamed on t i l =
  or
    [ or [lo <= l && l <= hi && on (if kind <= 1 then 2 else 3) | mPlaced f, (lo, hi, kind) <- mSpans f]
    , on 4 && positioned f && mDead f
    , on 6 && cycleMember t i && or [cycleArc t i d | (s, d, line) <- tRefs t, s == i, line == l]
    ]
 where
  f = tFiles t !! i

-- | A reference from member i to file d stays in i's cycle group: d a
-- positioned file of the group, itself only at a floor of 1.
cycleArc :: MTree -> Int -> Int -> Bool
cycleArc t i d = d < length fs && group t d == group t i && positioned (fs !! d) && (d /= i || tFloor t <= 1)
 where
  fs = tFiles t

-- | A function of the forest: its lines after the cut, c, its children.
data RNode = RNode Integer Integer Integer [RNode]

-- | Every function of the file as a forest, by recursive descent over
-- them in (lo ascending, hi descending) order: inside a parent spanning
-- [pLo, pHi], a function that does not lie in it ends the parent's run;
-- one that begins on a line the previous sibling holds begins after it;
-- none begins before its parent; one left without a line is skipped.
descend :: MFile -> [RNode]
descend f = fst (level 1 n Nothing (sortBy byStart [(max 1 lo, hi, c) | (lo, hi, c) <- mFuncs f, lo <= n]))
 where
  n = mLines f
  byStart (a, b, _) (a', b', _) = compare (a, negate b) (a', negate b')
  level pLo pHi prev ((lo, hi0, c) : rest)
    | lo > pHi || hi > pHi = ([], (lo, hi0, c) : rest)
    | start > hi = level pLo pHi prev rest
    | otherwise = (RNode start hi c kids : sibs, rest'')
   where
    hi = min n hi0
    start = max pLo (maybe lo (\h -> if lo <= h then h + 1 else lo) prev)
    (kids, rest') = level start hi Nothing rest
    (sibs, rest'') = level pLo pHi (Just hi) rest'
  level _ _ _ [] = ([], [])

-- | n * (1 - 1/2 * (rw + ap) / rw) when rw / (rw + ap) >= 1/2 (the
-- shipped rewrite threshold 50/100).
churnOf :: MFile -> Rational
churnOf f = case mChurn f of
  Just (rw, ap) | rw + ap > 0 && 2 * rw >= rw + ap -> mLines f % 1 * (1 - (1 % 2) * ((rw + ap) % rw))
  _ -> 0

-- | The layer sum, spelled from the bottom: each distinct positive ratio
-- r above the previous one adds (r - previous) * E(axes at or above r).
refLayered :: [Integer] -> MTree -> Rational
refLayered ws t = go 0 (sort (nub [w % top | w <- ws, w > 0]))
 where
  top = maximum (1 : ws)
  ratioOf a = (ws !! fromInteger a) % top
  go _ [] = 0
  go prev (r : rest) = (r - prev) * refExtent (\a -> ratioOf a >= r) t + go r rest

-- | (score, E, L) with seven weights and the violation factor k.
refScore :: [Integer] -> Rational -> MTree -> (Integer, Rational, Integer)
refScore ws k t = (s, e, l)
 where
  e = refLayered ws t
  l = refLines t
  s = if l == 0 then 1000 else max 0 (floor (1000 * (1 - k * e / (l % 1))))

-- | On a tree without excess terms: the sum over every line of the
-- largest weight ratio among the axes that name it.
refLineMax :: [Integer] -> MTree -> Rational
refLineMax ws t =
  sum
    [ maximum (0 : [(ws !! fromInteger a) % top | a <- [2, 3, 4, 6], lineNamed (== a) t i l])
    | (i, f) <- zip [0 ..] (tFiles t)
    , l <- [1 .. mLines f]
    ]
 where
  top = maximum (1 : ws)

refShares :: MTree -> [Integer]
refShares t = [if l == 0 then 0 else floor (1000 * refExtent (== a) t / (l % 1)) | a <- [0 .. 6]]
 where
  l = refLines t
