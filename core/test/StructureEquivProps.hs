-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The structure family against its second implementation (plan
-- v2.32 step 7): two hundred seeded requests through the shipped
-- respond and through ReferenceStructure / ReferenceSplit must agree
-- on the axis charges, the score, the headline entropy, the findings,
-- the declared-layout keys, the split-ROI rows and the shape echo; the
-- style rule over all 128 shape words and the diversity over every
-- small count vector agree; three forged requests per case are refused
-- by the reference's own reading, and an over-cap request degrades.
module StructureEquivProps (battery) where

import CE.Structure (respond)
import CE.Structure.Cost (structNodeCap)
import CE.Structure.Entropy (perMille, tsallis2Norm)
import qualified CE.Structure.Modularity as Mod
import CE.Structure.Shape (shapeCode)
import Data.Aeson (Result (..), Value (..), fromJSON, object, toJSON, (.=))
import qualified Data.Map.Strict as M
import ReferenceStructure (SIn (..), diversity, expected, flagged, style)
import StructureGen (cases, encode')
import WireHarness (fieldsOf, refusedBy, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "two hundred seeded trees agree on axes, score, entropy, findings, layout, seams and shape echo"
    , "the seeded trees are not vacuous: every axis flags, both seam outcomes and both divergence forms occur"
    , "the style rule agrees with the shipped classifier on all 128 shape words"
    , "the diversity agrees with the shipped Tsallis-2 on every vector of up to four bins of 0..4"
    , "forged depths, the retired pattern road and an odd inside sum are refused as the reference reads them"
    , "an over-cap request degrades over no facts and the default knobs"
    , "every table the core reads is priced: seven tables each under the cap but together over it, and a path form's own arcs, degrade by name (v2.35 V6)"
    , "modularity agrees on every three-directory graph of unit edges and intra masses, at four floors"
    , "booklet 04's range: no axis lists a directory twice, no charge passes half the scale, and at the neutral dial the score is in [scale/2, scale]"
    ]
    [ all (\c -> fieldsOf respond (encode' c) keys == Just (expected False c)) cases
    , nonVacuous
    , and [shapeCode b == style b | b <- [0 .. 127]]
    , and [perMille (tsallis2Norm cs) == diversity cs | k <- [0 .. 4], cs <- sequence (replicate k [0 .. 4])]
    , all forged cases
    , fieldsOf respond big (keys <> ["degraded", "reason"]) == Just (expected True blank <> [Just (Bool True), Just (String "structure_too_large")])
    , all tooLarge [spread, arcs]
    , modularGrid
    , all bounded cases && any neutralFlagged cases
    ]
 where
  keys = ["axes", "score", "entropy", "findings", "divergence", "deviations", "splitCandidates", "sizeExempt", "patternShapes"]
  blank = SIn [[0, 0, 0, 0, 0]] Nothing [] [] [] Nothing [] Nothing Nothing Nothing ([], [], [], []) []
  big = setKey "seamUnits" (toJSON (replicate (fromInteger structNodeCap) [0, 0, 1, 1 :: Integer])) (encode' blank)
  tooLarge r = fieldsOf respond r ["degraded", "reason"] == Just [Just (Bool True), Just (String "structure_too_large")]
  -- the seven tables the cap missed until V6, one seventh of the cap
  -- and one more row each
  spread = foldr (\(k, row) -> setKey k (toJSON (replicate each row))) (encode' blank) unpriced
  each = fromInteger (structNodeCap `div` 7 + 1)
  unpriced =
    [ ("patternShapes", [0, 1, 1 :: Integer]), ("conventions", [0, 1]), ("fileRefs", [0, 0, 0, 1])
    , ("declared", [0, 1]), ("staleDocRows", [0, 0]), ("staleEdgeRows", [0, 1]), ("redundancy", [0, 1, 0])
    ]
  -- one file and the cap's count of arcs from it to itself: the built
  -- tables stay one row each, the path form's own rows are the price
  arcs =
    object
      [ "proto" .= ("9.0.0" :: String), "type" .= ("structure.request" :: String), "id" .= (1 :: Int)
      , "paths" .= ["a.py" :: String], "refPaths" .= ["a.py" :: String]
      , "refPairs" .= replicate (fromInteger structNodeCap) [0, 0 :: Integer]
      ]

-- | Every directed unit-edge set over three directories, every intra
-- mass 0..2 per directory, the floors 1, 250, 500 and 1000 per mille
-- (ρ takes some of them exactly, so the strict comparison is
-- exercised at its boundary) and two mass
-- floors: the shipped integer inequality against the reference's ρ.
modularGrid :: Bool
modularGrid =
  and
    [ Mod.unmodular (fl, mass) refs edges [0, 1, 2] == concat [ds | (7, ds) <- flagged (tree refs edges fl mass)]
    | es <- sequence (replicate 6 [False, True])
    , intra <- sequence (replicate 3 [0, 1, 2])
    , fl <- [1, 250, 500, 1000]
    , mass <- [1, 3]
    , let edges = [[a, b, 1] | ((a, b), True) <- zip arcs es]
    , let refs = [[d, 2 * e, toInteger (length [() | [a, b, _] <- edges, a == d || b == d]), 1] | (d, e) <- zip [0 ..] intra]
    ]
 where
  arcs = [(a, b) | a <- [0 .. 2], b <- [0 .. 2], a /= b]
  tree refs edges fl mass =
    SIn [[0, 0, 0, 2, 0], [1, 0, 1, 0, 0], [2, 0, 1, 0, 0]] Nothing [] refs [] Nothing [] Nothing (Just edges) Nothing ([], [], [], []) [[19, fl], [20, mass]]

-- | Booklet 04 (the density fold): each axis flags DISTINCT directories
-- of the N, so v <= N and floor(scale * v / (v + N)) <= scale / 2; the
-- neutral-dial fold is then at least scale - floor(scale / 2).
bounded :: SIn -> Bool
bounded c = case fieldsOf respond (encode' c) ["axes", "score", "findings"] of
  Just [Just a, Just s, Just f]
    | Success axs <- (fromJSON a :: Result [[Integer]])
    , Success sc <- (fromJSON s :: Result Integer)
    , Success fs <- (fromJSON f :: Result [[Integer]]) ->
        all (\r -> case r of [_, p] -> 2 * p <= scale; _ -> False) axs
          && M.size (M.fromList [(r, ()) | r <- fs]) == length fs
          && (not (neutral c) || (2 * sc >= scale && sc <= scale))
  _ -> False
 where
  scale = knobOf c 8 1000

neutralFlagged :: SIn -> Bool
neutralFlagged c = neutral c && any (\(_, ds) -> not (null ds)) (flagged c)

neutral :: SIn -> Bool
neutral c = knobOf c 7 10 == 10

-- | The effective knob: the last declared row, else the default.
knobOf :: SIn -> Integer -> Integer -> Integer
knobOf c code dflt = last (dflt : [v | [k, v] <- sKnobs c, k == code])

nonVacuous :: Bool
nonVacuous =
  and [any (\c -> any (\(a', ds) -> a' == a && not (null ds)) (flagged c)) cases | a <- [0 .. 7]]
    && all (\i -> any (filled i) cases) [6, 7]
    && any (\c -> filled 4 c) cases
    && any (\c -> not (null (sDeclared c)) && not (filled 4 c)) cases
 where
  filled i c = case drop i (expected False c) of
    (Just (Array xs) : _) -> not (null xs)
    _ -> False

-- | Two forgeries of a legal case, each with the offender the
-- reference names: node 1's depth moved off its parent's, and
-- directory 0's inside sum made odd while a dir-edge table rides. The
-- unforged case itself is judged, not refused. (The retired `patterns`
-- table is refused once, alone and beside patternShapes, in
-- StructureProps.shapeRoad.)
forged :: SIn -> Bool
forged c =
  fieldsOf respond (encode' c) ["proto"] /= Nothing
    && and [refusedBy respond (encode' f) why | (f, why) <- forgeries]
 where
  forgeries =
    [ (c {sNodes = moved}, "node 1: depth is not parent depth + 1")
    | moved@(_ : _ : _) <- [[if take 1 r == [1] then bump 2 5 r else r | r <- sNodes c]]
    ]
      <> [ (c {sRefs = [if take 1 r == [0] then bump 1 1 r else r | r <- sRefs c]}, "dirEdges: directory 0 fileRefs inside sum")
         | Just _ <- [sDirEdges c]
         , any ((== [0]) . take 1) (sRefs c)
         ]
  bump i by r = [if j == i then x + by else x | (j, x) <- zip [0 :: Int ..] r]
