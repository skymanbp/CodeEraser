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
import Data.Aeson (Value (..), toJSON)
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
    , "forged depths, two pattern roads and an odd inside sum are refused as the reference reads them"
    , "an over-cap request degrades over no facts and the default knobs"
    , "modularity agrees on every three-directory graph of unit edges and intra masses, at four floors"
    ]
    [ all (\c -> fieldsOf respond (encode' c) keys == Just (expected False c)) cases
    , nonVacuous
    , and [shapeCode b == style b | b <- [0 .. 127]]
    , and [perMille (tsallis2Norm cs) == diversity cs | k <- [0 .. 4], cs <- sequence (replicate k [0 .. 4])]
    , all forged cases
    , fieldsOf respond big (keys <> ["degraded", "reason"]) == Just (expected True blank <> [Just (Bool True), Just (String "structure_too_large")])
    , modularGrid
    ]
 where
  keys = ["axes", "score", "entropy", "findings", "divergence", "deviations", "splitCandidates", "sizeExempt", "patternShapes"]
  blank = SIn [[0, 0, 0, 0, 0]] Nothing Nothing [] [] [] Nothing [] Nothing Nothing Nothing ([], [], [], []) []
  big = setKey "seamUnits" (toJSON (replicate (fromInteger structNodeCap) [0, 0, 1, 1 :: Integer])) (encode' blank)

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
    SIn [[0, 0, 0, 2, 0], [1, 0, 1, 0, 0], [2, 0, 1, 0, 0]] Nothing Nothing [] refs [] Nothing [] Nothing (Just edges) Nothing ([], [], [], []) [[19, fl], [20, mass]]

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

-- | Three forgeries of a legal case, each with the offender the
-- reference names: node 1's depth moved off its parent's, both
-- pattern roads riding, and directory 0's inside sum made odd while a
-- dir-edge table rides. The unforged case itself is judged, not
-- refused.
forged :: SIn -> Bool
forged c =
  fieldsOf respond (encode' c) ["proto"] /= Nothing
    && and [refusedBy respond (encode' f) why | (f, why) <- forgeries]
 where
  forgeries =
    [ (c {sNodes = moved}, "node 1: depth is not parent depth + 1")
    | moved@(_ : _ : _) <- [[if take 1 r == [1] then bump 2 5 r else r | r <- sNodes c]]
    ]
      <> [(c {sPatterns = Just [], sShapes = Just []}, "patternShapes: rides beside patterns (one road)")]
      <> [ (c {sRefs = [if take 1 r == [0] then bump 1 1 r else r | r <- sRefs c]}, "dirEdges: directory 0 fileRefs inside sum")
         | Just _ <- [sDirEdges c]
         , any ((== [0]) . take 1) (sRefs c)
         ]
  bump i by r = [if j == i then x + by else x | (j, x) <- zip [0 :: Int ..] r]
