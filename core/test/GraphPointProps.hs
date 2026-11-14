{-# LANGUAGE OverloadedStrings #-}
-- the second family is folded once and never held: no floating of its
-- list to a top-level constant
{-# OPTIONS_GHC -fno-full-laziness #-}

-- | The dead-file gate (plan v2.35, the dead-file amendment, phase 5B):
-- CE.Graph.Point against a reference written from the plan's sentences.
-- D is the unreferenced files; every member of D has a state; M is the
-- members whose state the mask names; a file blocks when it is a root OF
-- M (CE.Graph.Roots' law with M in the place of D).
--
-- The reference spells the law apart from the product, as
-- GraphRootsProps' does: no component algorithm, the support relation
-- read off the boolean transitive closure (GraphRootsProps.closureOf,
-- Warshall's loop), f blocking when f is in M and every member of M that
-- supports f is supported by f.
--
-- Two families. The first: every arc set on three nodes without
-- self-loops (64), the entry set empty or one node (4), the file nodes
-- any subset (8) -- 2,048 graphs; for each, every comparison point over
-- its file nodes' ids (no point, or each file's id absent / in `files`
-- only / in `files` and `dead`, beside one id no node has) and every
-- block mask (0..3). The second: every arc set on four nodes (4,096),
-- no entry (every node unreferenced), the file nodes any subset (16) --
-- every assignment of the states to up to four unreferenced files
-- (untold for all, or each file standing / orphaned / born) under every
-- mask: about 4.5 million gates, folded once.
module GraphPointProps (battery) where

import CE.Graph.Build (build, reachFrom)
import CE.Graph.Cost (underCap)
import qualified CE.Graph.Point as Point
import qualified CE.Graph.Roots as Roots
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Bits (testBit)
import qualified Data.IntSet as IS
import Data.List (sort)
import GraphRootsProps (closureOf)
import WireHarness (runLegs)

-- | One gate: the graph's dead files and roots, the file nodes, the
-- point, the mask, the product's roots-of-a-set and the reference's
-- support relation (g supports f: a path of length >= 1 from g to f).
data Gate = Gate
  { gDead :: [Int]
  , gRoots :: [Int]
  , gFiles :: [Int]
  , gPoint :: Maybe ([Integer], [Integer])
  , gMask :: Integer
  , gRootsOf :: [Int] -> [Int]
  , gSup :: Int -> Int -> Bool
  }

battery :: IO Bool
battery =
  runLegs
    ( [ "every unreferenced file's state equals the reference's (2,048 graphs, every point, every mask)"
      , "`states` is one row per unreferenced file, ascending, with its state code"
      , "blocked equals the reference's roots of M"
      , "mask 0 blocks nothing"
      , "a standing file is never blocked"
      , "no point and a mask: unanswered, nothing blocked"
      , "deadNow equals the reference's two lists"
      , "the family holds every state, a blocked file of each blockable state, a blocked dependent and the partial-mask sibling"
      ]
        <> fourNames
    )
    ( [ every (\g -> map (state g) (gDead g) == map (refCode g) (gDead g))
      , every (\g -> lookup "states" (Point.gKeys (run g)) == Just (toJSON [[toInteger d, refCode g d] | d <- gDead g]))
      , every (\g -> Point.gBlocked (run g) == refBlocked g)
      , every (\g -> gMask g /= 0 || null (Point.gBlocked (run g)))
      , every (\g -> all ((/= "standing") . refName g) (Point.gBlocked (run g)))
      , every (\g -> gPoint g /= Nothing || gMask g == 0 || (null (Point.gBlocked (run g)) && lookup "blockUnanswered" (Point.gKeys (run g)) == Just (Bool True)))
      , every (\g -> lookup "deadNow" (Point.gKeys (run g)) == Just (object ["files" .= map idOf (gFiles g), "dead" .= sort (map idOf (gDead g))]))
      , coverage
      ]
        <> fourLegs
    )
 where
  every p = all p gates
  state g = Point.gState (run g)

-- | The product's gate over one case.
run :: Gate -> Point.Gate
run g = Point.gate (Just [[toInteger n, idOf n] | n <- gFiles g]) (fmap pointValue (gPoint g)) (gMask g) (gDead g) (gRootsOf g)
 where
  pointValue (fs, ds) = object ["files" .= fs, "dead" .= ds]

-- | A file node's id: far from the node numbers, so an index read as an
-- id would show.
idOf :: Int -> Integer
idOf n = 1000 + toInteger n

-- | The reference, from the plan's sentences: no point -> untold; the id
-- among the point's dead -> standing; among its files and not dead ->
-- orphaned; not among its files -> born.
refName :: Gate -> Int -> String
refName g r = case gPoint g of
  Nothing -> "untold"
  Just (fs, ds) -> head' ([s | (s, True) <- [("standing", idOf r `elem` ds), ("orphaned", idOf r `elem` fs)]] <> ["born"])
 where
  head' xs = case xs of
    x : _ -> x
    [] -> "born"

refCode :: Gate -> Int -> Integer
refCode g r = case lookup (refName g r) (zip ["untold", "standing", "orphaned", "born"] [0 ..]) of
  Just c -> c
  Nothing -> -1

-- | The states a mask names: bit 0 orphaned, bit 1 born.
masked :: Integer -> [String]
masked m = [s | (b, s) <- [(0, "orphaned"), (1, "born")], testBit m b]

-- | M: the unreferenced files whose state the mask names.
refM :: Gate -> [Int]
refM g = [d | d <- gDead g, refName g d `elem` masked (gMask g)]

-- | The reference's blocked files: the members of M that every member
-- of M supporting them is supported by.
refBlocked :: Gate -> [Int]
refBlocked g = [f | f <- m, all (\h -> not (gSup g h f) || gSup g f h) m]
 where
  m = refM g

gates :: [Gate]
gates =
  [ Gate ds (Roots.gRoots (Roots.group underCap b reach ds)) fs point mask (Roots.gRoots . Roots.group underCap b reach) sup
  | m <- [0 .. 63 :: Int]
  , e <- [] : [[v] | v <- [0 .. 2]]
  , fsBits <- [0 .. 7 :: Int]
  , let arcs = [a | (i, a) <- zip [0 ..] arcPool, testBit m i]
        fs = [v | v <- [0 .. 2], testBit fsBits v]
        b = build 5 [] 3 [[toInteger s, toInteger d, 0, 1] | (s, d) <- arcs]
        reach = reachFrom b e
        ds = [f | f <- fs, not (IS.member f reach)]
        sup = supportsIn (closureOf 3 arcs)
  , point <- Nothing : map Just (points fs)
  , mask <- [0 .. 3]
  ]
 where
  arcPool = [(s, d) | s <- [0 .. 2 :: Int], d <- [0 .. 2], s /= d]

-- | g supports f: a path of length >= 1 from g to f, g /= f.
supportsIn :: [[Bool]] -> Int -> Int -> Bool
supportsIn cl g f = g /= f && cl !! g !! f

-- | The shapes the first family must hold.
coverage :: Bool
coverage =
  all (\s -> any (\g -> any ((== s) . refName g) (gDead g)) gates) ["untold", "standing", "orphaned", "born"]
    && all (\s -> any (\g -> any ((== s) . refName g) (Point.gBlocked (run g))) gates) ["orphaned", "born"]
    -- the reported hole: a blocked file that is a dependent in D
    && any (\g -> any (`notElem` gRoots g) (Point.gBlocked (run g))) gates
    -- the partial-mask sibling: born only, a born file blocks whose
    -- every supporter in D is orphaned
    && any sibling gates
 where
  sibling g =
    gMask g == 2
      && any (\f -> refName g f == "born" && f `elem` Point.gBlocked (run g) && not (null (sups g f)) && all ((== "orphaned") . refName g) (sups g f)) (gDead g)
  sups g f = [h | h <- gDead g, gSup g h f]

-- | Every point over the files: each id absent (0), in `files` only (1)
-- or in `files` and `dead` (2), beside the stray id 7 in `files`.
points :: [Int] -> [([Integer], [Integer])]
points fs =
  [ (sort (7 : [idOf f | (f, k) <- zip fs ks, k >= 1]), sort [idOf f | (f, k) <- zip fs ks, k == 2])
  | ks <- mapM (const [0 :: Int, 1, 2]) fs
  ]

fourNames :: [String]
fourNames =
  [ "four files, every state assignment, every mask: blocked is empty iff M is empty"
  , "blocked is a subset of M"
  , "no blocked file is supported by a member of M outside its own component"
  , "when M = D the blocked set is D's roots (the roots-only law)"
  , "every root of D whose state is in the mask is blocked"
  , "blocked equals the reference's roots of M"
  ]

-- | The second family, folded once (strictly: its 4.5 million gates are
-- never held).
fourLegs :: [Bool]
fourLegs = foldl' step (replicate (length fourNames) True) (fourGates 4)
 where
  step acc g = force (zipWith (&&) acc (legs g))
  force xs = foldr seq () xs `seq` xs
  legs g =
    let bl = Point.gBlocked (run g)
        m = refM g
     in [ null bl == null m
        , all (`elem` m) bl
        , all (\f -> all (\h -> not (gSup g h f) || gSup g f h) m) bl
        , m /= gDead g || bl == gRoots g
        , all (`elem` bl) [r | r <- gRoots g, r `elem` m]
        , bl == refBlocked g
        ]

fourGates :: Int -> [Gate]
fourGates n =
  [ Gate fs (Roots.gRoots (Roots.group underCap b IS.empty fs)) fs point mask (Roots.gRoots . Roots.group underCap b IS.empty) sup
  | m <- [0 .. 2 ^ (n * (n - 1)) - 1 :: Int]
  , let arcs = [a | (i, a) <- zip [0 ..] arcPool, testBit m i]
        b = build 5 [] n [[toInteger s, toInteger d, 0, 1] | (s, d) <- arcs]
        sup = supportsIn (closureOf n arcs)
  , fsBits <- [0 .. 2 ^ n - 1 :: Int]
  , let fs = [v | v <- [0 .. n - 1], testBit fsBits v]
  , point <- Nothing : map Just (points fs)
  , mask <- [0 .. 3]
  ]
 where
  arcPool = [(s, d) | s <- [0 .. n - 1], d <- [0 .. n - 1], s /= d]
