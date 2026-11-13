{-# LANGUAGE OverloadedStrings #-}

-- | Model trees for the extent law's battery (VerdictExtentProps): an
-- exhaustive small family, a seeded random family, and the two roads
-- into production - the verdict/1 request through the real respond,
-- and the same tables straight into CE.Verdict.ExtentFacts.scored.
module VerdictExtentGen
  ( Judged (..)
  , Replied (..)
  , exhaustive
  , randomFamily
  , requestOf
  , viaRespond
  , viaScored
  , viaRows
  , rowsReversed
  , unitsOf
  , tablesOf
  ) where

import CE.Verdict (respond)
import CE.Verdict.Extent (Unit, layered, totalLines)
import CE.Verdict.ExtentFacts (Located (..), scored, units)
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), classKnobsOf, scoreBound)
import Data.Aeson (Value (..), toJSON)
import qualified Data.Aeson.KeyMap as KM
import Data.List (nub, sort, unfoldr)
import ReferenceExtent
import WireHarness (replyObjWith, setKey, tabledRequest)

-- | A production answer: score, the seven own shares, E, L.
data Judged = Judged {jScore :: Integer, jShares :: [Integer], jE :: Rational, jL :: Integer} deriving (Eq, Show)

-- | What the verdict/1 reply states: score, the seven own shares, E
-- as tenths (the exact E never leaves the fold), L.
data Replied = Replied {rScore :: Integer, rShares :: [Integer], rTenths :: Integer, rL :: Integer} deriving (Eq, Show)

-- | The request tables of a tree, by name, plus the tier size.
tablesOf :: MTree -> ([(String, [[Integer]])], Integer)
tablesOf t =
  ( [ ("sim", []), ("pos", pos), ("tier", [[u, 0] | u <- [0 .. n - 1]]), ("churn", churn), ("cochange", [])
    , ("continuous", map fst cont), ("contSpans", map snd cont), ("dupSpans", dups), ("refLines", refs)
    , ("proseLines", [[u, mLines f] | (u, f) <- placed, mProse f])
    , ("classKnobs", sort [[i, knob, v] | (i, (s, c)) <- zip [1 ..] classes, (knob, v) <- [(0, s), (1, c)]])
    ]
  , n
  )
 where
  fs = tFiles t
  placed = zip [0 ..] [f | f <- fs, mPlaced f]
  n = toInteger (length placed)
  uOf = [if mPlaced f then length (filter mPlaced (take p fs)) else -1 | (p, f) <- zip [0 ..] fs]
  code f = mPlaced f && not (mProse f)
  -- one class id per distinct (S, C), from 1; with any class every row
  -- carries the column (one arity per table), 0 = the tree's knobs
  classes = nub [c | f <- fs, not (mProse f), Just c <- [mClass f]]
  classCol f = [sum [toInteger i | Just c <- [mClass f], (i, c') <- zip [1 :: Int ..] classes, c' == c] | not (null classes)]
  group g = [() | f <- fs, code f, mScc f == Just g]
  indeg u = length (nub [s | (s, d, _) <- tRefs t, toInteger (uOf !! d) == u, s /= d])
  pos = [[u, toInteger (indeg u), maybe u (+ 100000) (mScc f), maybe 1 (toInteger . length . group) (mScc f), if mDead f then 0 else 1] | (u, f) <- placed, code f]
  churn = [[u, rw, ap] | (u, f) <- placed, code f, Just (rw, ap) <- [mChurn f]]
  cont =
    concat
      [ ([100 * (p + 1), 0, mLines f] <> classCol f, [toInteger u, 0, 0]) : [([100 * (p + 1) + j, 1, c] <> classCol f, [toInteger u, lo, hi]) | (j, (lo, hi, c)) <- zip [1 ..] (mFuncs f)]
      | (p, f, u) <- zip3 [0 ..] fs uOf
      , not (mProse f)
      ]
  dups = sort (nub [[u, lo, hi, kind] | (u, f) <- placed, (lo, hi, kind) <- mSpans f])
  refs = sort (nub [[toInteger (uOf !! s), toInteger (uOf !! d), l] | (s, d, l) <- tRefs t])

-- | The knobs a tree is judged with: its soft line as the size ceiling
-- (no baseline, no LOC multiset: the fallback is the line), its
-- complexity ceiling, the cycle floor, the violation cost.
knobsOf :: MTree -> Integer -> ScoreKnobs
knobsOf t v = scoreBound {sSizeCeil = tSoft t, sCocCeil = tCeil t, sCycleFloor = tFloor t, sViolCost = v}

-- | The reply read as it stands: an `extent` object of exactly the two
-- keys `tenths` and `lines`, or Nothing.
viaRespond :: [Integer] -> Integer -> MTree -> Maybe Replied
viaRespond ws v t = do
  o <- replyObjWith respond (requestOf ws v t)
  Number s <- KM.lookup "score" o
  axes <- KM.lookup "axes" o
  Object ext <- KM.lookup "extent" o
  [Number tn, Number l] <- if KM.size ext == 2 then traverse (`KM.lookup` ext) ["tenths", "lines"] else Nothing
  shares <- rows axes
  pure (Replied (truncate s) shares (truncate tn) (truncate l))
 where
  rows (Array xs) = traverse (\x -> case x of Array r | [_, Number p] <- foldr (:) [] r -> Just (truncate p); _ -> Nothing) (foldr (:) [] xs)
  rows _ = Nothing

-- | The verdict/1 request of a tree under seven weights and a violation cost.
requestOf :: [Integer] -> Integer -> MTree -> Value
requestOf ws v t =
  foldr
    (uncurry setKey)
    (tabledRequest "10.0.0" "verdict.request" (tables <> [("weights", [[a, w] | (a, w) <- zip [0 ..] ws]), ("ceilings", [[0, tSoft t], [1, tCeil t]]), ("thresholds", [[4, v], [7, tFloor t]])]))
    ([("discrete", toJSON ([] :: [Integer])), ("baseline", Null), ("floor", Null), ("docFiles", toJSON (docs t))] <> [("cycleSelfLoops", toJSON (loopsOf t)) | tFloor t == 1])
 where
  (tables, _) = tablesOf t

-- | The placed code files with a reference to themselves (the self-arc
-- table, sent at floor 1 only).
loopsOf :: MTree -> [Integer]
loopsOf t = sort (nub [toInteger (length (filter mPlaced (take s fs))) | (s, d, _) <- tRefs t, s == d, mPlaced (fs !! s), not (mProse (fs !! s))])
 where
  fs = tFiles t

docs :: MTree -> [Integer]
docs t = [u | (u, f) <- zip [0 ..] [f | f <- tFiles t, mPlaced f], mDoc f, not (mProse f)]

factsOf :: MTree -> (Facts, Located)
factsOf t = (Facts (get "pos") (get "churn") (get "continuous") (docs t) (classKnobsOf (get "classKnobs")) (loopsOf t), loc)
 where
  (tables, n) = tablesOf t
  get k = maybe [] id (lookup k tables)
  loc = Located n (get "contSpans") (get "dupSpans") (get "refLines") (get "proseLines")

viaScored :: [Integer] -> Integer -> MTree -> Judged
viaScored ws v t = Judged s (map snd axes) (layered (zip [0 ..] ws) us) (totalLines us)
 where
  (f, loc) = factsOf t
  (s, axes, _) = scored (knobsOf t v) Nothing [[a, w] | (a, w) <- zip [0 ..] ws] f loc
  us = unitsOf t

-- | The score under weight rows as sent (an axis no row names weighs
-- what the core gives it: the dead-file amendment's opt-in reads here)
-- at the neutral violation cost.
viaRows :: [[Integer]] -> MTree -> Integer
viaRows ws t = s
 where
  (f, loc) = factsOf t
  (s, _, _) = scored (knobsOf t 10) Nothing ws f loc

-- | The score with every table's rows reversed before the fold (the
-- continuous table and its spans together, as they are aligned).
rowsReversed :: MTree -> Bool
rowsReversed t = run f loc == run f' loc'
 where
  (f, loc) = factsOf t
  f' = f {fPos = reverse (fPos f), fChurn = reverse (fChurn f), fCont = reverse (fCont f), fDocFiles = reverse (fDocFiles f)}
  loc' = Located (lUniverse loc) (reverse (lSpans loc)) (reverse (lDups loc)) (reverse (lRefs loc)) (reverse (lProse loc))
  run g l = let (s, axes, _) = scored (knobsOf t 10) Nothing [] g l in (s, axes)

unitsOf :: MTree -> [Unit]
unitsOf t = let (f, loc) = factsOf t in units (knobsOf t 10) Nothing f loc

-- | One code file of four lines through every combination of its own
-- facts, beside a partner file that may share its cycle and receive a
-- reference line (or the file refers to itself), at two soft lines:
-- 3,456 trees. The function shapes: nested at depth 1 to 3, a shared
-- end line, identical one-line spans, siblings sharing a boundary line,
-- three meeting on one line, an under-ceiling parent over an
-- over-ceiling child, a c = 0 function cutting an over-ceiling one, an
-- over-ceiling one inside one at the ceiling (c = C) inside an
-- over-ceiling one, an over-ceiling one inside a c = 0 one.
exhaustive :: [MTree]
exhaustive =
  [ MTree [f, partner] refs s 15 2
  | dead <- [False, True]
  , spans <- [[], [(1, 2, 0)], [(2, 4, 2)], [(1, 2, 0), (2, 4, 2)]]
  , funcs <- [[], [(1, 3, 20)], [(1, 4, 16), (2, 3, 30)], [(1, 4, 16), (4, 4, 30)], [(1, 4, 10), (2, 3, 30)], [(1, 4, 16), (2, 4, 20), (3, 4, 30)], [(2, 2, 20), (2, 2, 30)], [(1, 2, 20), (2, 4, 30)], [(1, 3, 16), (3, 3, 25), (3, 4, 30)], [(1, 2, 0), (2, 4, 30)], [(1, 4, 30), (2, 3, 15), (3, 3, 40)], [(1, 4, 0), (2, 3, 30)]]
  , churn <- [Nothing, Just (3, 1), Just (1, 3)]
  , cyc <- [Nothing, Just 0]
  , refs <- [[], [(0, 1, 2)], [(0, 0, 3)]]
  , s <- [2, 4]
  , let f = (blank 4) {mDead = dead, mSpans = spans, mFuncs = funcs, mChurn = churn, mScc = cyc}
  , let partner = (blank 3) {mScc = Just 0}
  ]

-- | Two hundred seeded trees of one to seven files: placed code files,
-- prose files and scan-only files; functions at c = 0, at the ceiling,
-- below and above it, with any spans; spans, dead flags, cycle groups,
-- churn, reference lines (self references too), cycle floors 1 and 2,
-- and two path classes with their own soft line and ceiling on some
-- placed code and scan-only files.
randomFamily :: [MTree]
randomFamily = take 200 (unfoldr (Just . tree) 20261111)
 where
  next s = (s * 6364136223846793005 + 1442695040888963407) `mod` (2 ^ (64 :: Int))
  pick s m = (next s, (next s `div` 65536) `mod` m)
  draws s0 k = take k (drop 1 (iterate (\(s, _) -> pick s 1000003) (s0, 0)))
  tree s0 =
    let (s1, nf) = pick s0 7
        fs = [file (map snd (draws (s1 + 7919 * i) 24)) | i <- [0 .. nf]]
        (s2, nr) = pick s1 5
        code = [i | (i, f) <- zip [0 ..] fs, mPlaced f, not (mProse f)]
        refs = [(src, dst, 1 + l `mod` mLines (fs !! src)) | not (null code), (_, x) <- draws s2 (fromInteger nr), let src = code !! fromInteger (x `mod` toInteger (length code)), let dst = code !! fromInteger ((x `div` 7) `mod` toInteger (length code)), let l = x `div` 49]
        (s3, si) = pick s2 4
        (s4, fl) = pick s3 2
        cs = map snd (draws s4 4)
        from xs j = xs !! fromInteger ((cs !! j) `mod` 4)
        two = [(from [2, 6, 12, 40] 0, from [5, 10, 15, 25] 1), (from [3, 9, 20, 400] 2, from [8, 16, 20, 30] 3)]
        classed = [if mProse f then f else f {mClass = [Nothing, Just (two !! 0), Just (two !! 1)] !! fromInteger (x `mod` 3)} | (f, (_, x)) <- zip fs (draws (s4 + 104729) (length fs))]
     in (MTree classed refs ([3, 8, 15, 30] !! fromInteger si) 15 (1 + fl), s4)
  file (a : b : c : d : e : g : h : i : j : k : l : m : more) =
    let n = 1 + a `mod` 20
        kind = b `mod` 10
        top = 1 + c `mod` n
        fn = [(1, top, [0, 5, 14, 15, 16, 20, 30] !! fromInteger (d `mod` 7)) | e `mod` 3 /= 0]
        nested = [(2, top - 1, 25) | e `mod` 3 == 1, top >= 3] <> [(top + 1, n + 2, 18) | g `mod` 2 == 0, top < n]
        -- up to four more with any spans: nesting at any depth, shared
        -- boundary lines, partial overlaps, identical spans
        free = [(lo, lo + w `mod` 6, [0, 10, 15, 16, 25, 40] !! fromInteger (x `mod` 6)) | (u, w, x) <- triples (take (3 * fromInteger (e `mod` 5)) more), let lo = 1 + u `mod` n]
        triples (u : w : x : rest) = (u, w, x) : triples rest
        triples _ = []
        spans = [(1 + h `mod` n, min n (1 + h `mod` n + i `mod` 5), j `mod` 3) | i `mod` 4 /= 0]
     in case kind of
          0 -> (blank n) {mPlaced = False}
          1 -> (blank n) {mProse = True, mSpans = [(lo, hi, 2) | (lo, hi, _) <- spans]}
          _ ->
            (blank n)
              { mDoc = k `mod` 7 == 0, mDead = k `mod` 4 == 1, mScc = if l `mod` 3 == 0 then Just (l `mod` 2) else Nothing
              , mFuncs = fn <> nested <> free, mChurn = if m `mod` 3 == 0 then Just (m `mod` 5, 1 + m `mod` 3) else Nothing, mSpans = spans
              }
  file _ = blank 1
