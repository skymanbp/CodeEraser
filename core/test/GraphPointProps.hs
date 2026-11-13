{-# LANGUAGE OverloadedStrings #-}

-- | The dead-file gate (plan v2.35, the dead-file amendment, phase 5B):
-- CE.Graph.Point against a reference written from the plan's four
-- sentences, exhaustively over a smaller family than GraphRootsProps'.
-- The family: every arc set on three nodes without self-loops (2^6 = 64),
-- the entry set empty or one node (4), the file nodes any subset (8) --
-- 2,048 graphs; for each, every comparison point over its file nodes'
-- ids (no point, or each file's id absent / in `files` only / in `files`
-- and `dead`, beside one id no node has: up to 28) and every block mask
-- (0..3) -- 229,376 gates at most. The roots are CE.Graph.Roots' (its
-- own battery holds them to a reference); this battery holds what the
-- gate adds: states, the blocked set, the unanswered marker, `deadNow`.
module GraphPointProps (battery) where

import CE.Graph.Build (build, reachFrom)
import CE.Graph.Cost (underCap)
import qualified CE.Graph.Point as Point
import qualified CE.Graph.Roots as Roots
import Data.Aeson (Value (..), object, (.=))
import Data.Bits (testBit)
import qualified Data.IntSet as IS
import Data.List (sort)
import WireHarness (runLegs)

-- | One gate: the graph's dead files and roots, the ids, the point, the mask.
data Gate = Gate {gDead :: [Int], gRoots :: [Int], gFiles :: [Int], gPoint :: Maybe ([Integer], [Integer]), gMask :: Integer}

battery :: IO Bool
battery =
  runLegs
    [ "the state function is total and equals the reference's (2,048 graphs, every point, every mask)"
    , "blocked is exactly the roots whose state the mask names"
    , "mask 0 blocks nothing"
    , "a standing root is never blocked"
    , "no point and a mask: unanswered, nothing blocked"
    , "deadNow equals the reference's two lists"
    , "the family holds every state and a blocked root of each blockable state"
    ]
    [ every (\g -> map (state g) (gRoots g) == map (refCode g) (gRoots g))
    , every (\g -> Point.gBlocked (run g) == [r | r <- gRoots g, refName g r `elem` masked (gMask g)])
    , every (\g -> gMask g /= 0 || null (Point.gBlocked (run g)))
    , every (\g -> all ((/= "standing") . refName g) (Point.gBlocked (run g)))
    , every (\g -> gPoint g /= Nothing || gMask g == 0 || (null (Point.gBlocked (run g)) && lookup "blockUnanswered" (Point.gKeys (run g)) == Just (Bool True)))
    , every (\g -> lookup "deadNow" (Point.gKeys (run g)) == Just (object ["files" .= map idOf (gFiles g), "dead" .= sort (map idOf (gDead g))]))
    , all (\s -> any (\g -> any ((== s) . refName g) (gRoots g)) gates) ["untold", "standing", "orphaned", "born"]
        && all (\s -> any (\g -> any ((== s) . refName g) (Point.gBlocked (run g))) gates) ["orphaned", "born"]
    ]
 where
  every p = all p gates
  state g = Point.gState (run g)

-- | The product's gate over one case.
run :: Gate -> Point.Gate
run g = Point.gate (Just [[toInteger n, idOf n] | n <- gFiles g]) (fmap pointValue (gPoint g)) (gMask g) (gDead g) (gRoots g)
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

gates :: [Gate]
gates =
  [ Gate ds (Roots.gRoots (Roots.group underCap b reach ds)) fs point mask
  | m <- [0 .. 63 :: Int]
  , e <- [] : [[v] | v <- [0 .. 2]]
  , fsBits <- [0 .. 7 :: Int]
  , let arcs = [a | (i, a) <- zip [0 ..] arcPool, testBit m i]
        fs = [v | v <- [0 .. 2], testBit fsBits v]
        b = build 5 [] 3 [[toInteger s, toInteger d, 0, 1] | (s, d) <- arcs]
        reach = reachFrom b e
        ds = [f | f <- fs, not (IS.member f reach)]
  , point <- Nothing : map Just (points fs)
  , mask <- [0 .. 3]
  ]
 where
  arcPool = [(s, d) | s <- [0 .. 2 :: Int], d <- [0 .. 2], s /= d]

-- | Every point over the files: each id absent (0), in `files` only (1)
-- or in `files` and `dead` (2), beside the stray id 7 in `files`.
points :: [Int] -> [([Integer], [Integer])]
points fs =
  [ (sort (7 : [idOf f | (f, k) <- zip fs ks, k >= 1]), sort [idOf f | (f, k) <- zip fs ks, k == 2])
  | ks <- mapM (const [0 :: Int, 1, 2]) fs
  ]
