-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates family (plan v2.33 W3): the two bounds sit exactly
-- on the 85/100 boundary in both directions and each owns its tally,
-- S2 pairs one key across files within a language and merges its bit
-- into a fingerprint pair, an anchor lands on the innermost unit (the
-- earliest on a tie) or is counted unowned, identical sets share every
-- band, a hot fingerprint group chains, S5 windows, cuts and never
-- duplicates, the shortcut MinHash equals the byte-by-byte one, the
-- floor-aware intersection agrees with the bound, two hundred seeded
-- requests agree with ReferenceCandidates, the contract's refusals
-- agree with the reference's predicates, and every cap holds.
module CandidatesProps (battery) where

import CE.Candidates (respond)
import CE.Candidates.Contract (CandReq (..), overCap)
import CE.Candidates.Cost (candidateNearCap, candidatePrintCap, candidateSigCap, candidateUnitCap)
import CE.Candidates.Lsh (sets, signatureOf)
import CE.Candidates.Units (columns, interUpTo)
import CE.Clone.Prefilter (reachFloor, sizeBelow)
import Data.Aeson (Key, Value (..), object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import Data.Array.Unboxed (elems)
import Data.Bits (shiftR, xor, (.&.))
import qualified Data.Map.Strict as M
import Data.Word (Word64)
import ReferenceCandidates (Req (..), expected, requests)
import ReferenceContract (answers, cells)
import WireHarness (fieldsOf, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "the bounds sit on the 85/100 boundary and each owns its tally"
    , "S2 pairs one key across files within a language and merges its bit into a fingerprint pair"
    , "an anchor lands on the innermost unit, the earliest on a tie, or is counted unowned"
    , "identical sets share all 32 bands and a hot fingerprint group chains"
    , "S5 windows by size, cuts by label, never duplicates, and its pairs carry bit 4 alone"
    , "the shortcut MinHash equals fnv1a over the element and salt bytes"
    , "reachFloor is the size bound's threshold and the floor-aware walk decides the label bound alike"
    , "two hundred seeded requests agree with the reference on every reply field"
    , "the seeded requests are not vacuous: every source finds, bounds cut, anchors miss, groups chain"
    , "the contract's refusals agree with the reference's predicates"
    , "the unit cap holds on both sides and the other three caps are counted"
    ]
    [ all boundary boundaries
    , s2Case
    , ownerCase
    , bandCase && hotCase
    , s5Case
    , minhashLaw
    , floorLaw && floorWalk
    , all agrees requests
    , nonVacuous
    , all agrees refusals
    , unitCap && capsCounted
    ]

request :: Req -> Value
request r =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("candidates.request" :: String)
    , "id" .= (1 :: Int)
    , "units" .= rUnits r
    , "sigs" .= rSigs r
    , "prints" .= rPrints r
    , "near" .= rNear r
    , "exhaustive" .= rEx r
    ]

fields :: [String]
fields = ["pairs", "raw", "bandGroups", "counts", "degraded", "reason"]

agrees :: Req -> Bool
agrees r = answers respond fields (request r) (expected r)

-- | The sixteen counts, every one zero but those named.
countsWith :: [(Key, Integer)] -> Maybe Value
countsWith named = Just (object [k .= M.findWithDefault 0 k (M.fromList named) | k <- names])
 where
  names =
    [ "units", "prints", "near", "unowned", "selfPairs", "printHot", "bandHot", "union", "crossLanguage"
    , "prunedSize", "prunedLabel", "survivors", "s5Windowed", "s5PrunedLabel", "s5Already", "s5New"
    ]

-- | The reply's pairs, per-source counts, bucket sizes and counts.
replied :: Req -> Maybe [Maybe Value]
replied r = fieldsOf respond (request r) ["pairs", "raw", "bandGroups", "counts"]

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

-- | Distinct one-element sets, one per unit: no two share a band.
own :: [[Integer]] -> [[Integer]]
own us = [[k + 101] | k <- [0 .. toInteger (length us) - 1]]

-- | Units on line 1 of their own files, one near run between units 0
-- and 1. Rows 1–2: the size bound at 85 vs 84 against 100; rows 3–4:
-- equal sizes, the label multisets sharing 84 (cut) vs 85 (kept) at
-- 100. Each tally is [prunedSize, prunedLabel, survivors].
boundaries :: [([[Integer]], [Integer])]
boundaries =
  [ ([[0, 0, 0, 85, 1, 1, 1, 85], [0, 1, 1, 100, 1, 1, 1, 100]], [0, 0, 1])
  , ([[0, 0, 0, 84, 1, 1, 1, 84], [0, 1, 1, 100, 1, 1, 1, 100]], [1, 0, 0])
  , ([[0, 0, 0, 100, 1, 1, 1, 84, 2, 16], [0, 1, 1, 100, 1, 1, 1, 100]], [0, 1, 0])
  , ([[0, 0, 0, 100, 1, 1, 1, 85, 2, 15], [0, 1, 1, 100, 1, 1, 1, 100]], [0, 0, 1])
  ]

boundary :: ([[Integer]], [Integer]) -> Bool
boundary (us, t) =
  fmap (drop 3) (replied (Req False us (own us) [] [[0, 1, 1, 1]]))
    == Just [countsWith ([("units", 2), ("near", 1), ("union", 1)] <> zip ["prunedSize", "prunedLabel", "survivors"] t)]

-- | A one-kind unit row: language, file, key, nodes, span; the
-- histogram is kind 1 counting every node.
unit :: Integer -> Integer -> Integer -> Integer -> (Integer, Integer) -> [Integer]
unit lang file key nodes (from, to) = [lang, file, key, nodes, from, to, 1, nodes]

-- | Three units of one key: two in different files of one language
-- (an S2 pair), the third in another language (two crossings); then
-- two instances of one hash on those units' lines: the S2 pair also
-- found by S3 is one union pair carrying both bits.
s2Case :: Bool
s2Case =
  replied (Req False three (own three) [] [])
    == Just [rows [[0, 1, 2]], rows [[1, 0, 1]], rows [], shared []]
    && replied (Req False three (own three) [[77, 0, 1, 0], [77, 1, 1, 3]] [])
      == Just [rows [[0, 1, 6]], rows [[1, 0, 1], [2, 0, 1]], rows [], shared [("prints", 2)]]
 where
  three = [unit lang file 5 100 (1, 1) | (lang, file) <- [(0, 0), (0, 1), (1, 2)]]
  shared extra = countsWith (extra <> [("units", 3), ("union", 1), ("crossLanguage", 2), ("survivors", 1)])

-- | File 0 holds u0 on lines 1–10 and u1, u2 both on 3–5; file 1
-- holds u3 on 1–9. Near runs from line 4 (u1, the earliest of the
-- narrowest) and line 8 (u0) to u3; a third from line 20 is unowned.
ownerCase :: Bool
ownerCase =
  fmap (take 1) answered == Just [rows [[0, 3, 1], [1, 3, 1]]]
    && fmap (drop 3) answered
      == Just [countsWith [("units", 4), ("near", 3), ("unowned", 1), ("union", 2), ("survivors", 2)]]
 where
  us = [unit 0 file key 100 reach | (file, key, reach) <- [(0, 0, (1, 10)), (0, 1, (3, 5)), (0, 2, (3, 5)), (1, 3, (1, 9))]]
  answered = replied (Req False us (own us) [] [[0, line, 1, 1] | line <- [4, 8, 20]])

-- | Two units with one set share all 32 bands: one S4 pair, 32 buckets
-- of two.
bandCase :: Bool
bandCase =
  replied (Req False [unit 0 k k 100 (1, 1) | k <- [0, 1]] [[5, 9], [5, 9]] [] [])
    == Just [rows [[0, 1, 8]], rows [[3, 0, 1]], rows [[2, 32]], countsWith [("units", 2), ("union", 1), ("survivors", 1)]]

-- | Sixty-five instances of one hash, one per line 1..65 of file 0,
-- written in descending token order, over 65 one-line units: the group
-- is past the hot cap, so it chains by (file, tok) — 64 adjacent pairs,
-- one chained group.
hotCase :: Bool
hotCase =
  fmap (drop 3) (replied (Req False us (own us) prints []))
    == Just [countsWith [("units", 65), ("prints", 65), ("printHot", 1), ("union", 64), ("survivors", 64)]]
 where
  us = [unit 0 0 k 100 (k + 1, k + 1) | k <- [0 .. 64]]
  prints = [[3, 0, l, 100 - l] | l <- [1 .. 65]]

-- | Four units: three of 100 nodes (two sharing every kind, one
-- sharing none), one of 84 outside every window; the near run keeps
-- (0,1), which S5 counts as already kept and never repeats. Then two
-- units of 100 and 90 sharing everything and nothing found: the new
-- pair lands with bit 4 alone.
s5Case :: Bool
s5Case =
  windowed four [[0, 1, 1, 1]]
    == Just [rows [[0, 1, 1]], countsWith [("units", 4), ("near", 1), ("union", 1), ("survivors", 1), ("s5Windowed", 3), ("s5PrunedLabel", 2), ("s5Already", 1)]]
    && windowed [unit 0 0 0 100 (1, 1), unit 0 1 1 90 (1, 1)] []
      == Just [rows [[0, 1, 16]], countsWith [("units", 2), ("s5Windowed", 1), ("s5New", 1)]]
 where
  four = [unit 0 0 0 100 (1, 1), unit 0 1 1 100 (1, 1), [0, 2, 2, 100, 1, 1, 2, 100], unit 0 3 3 84 (1, 1)]
  windowed us near = fmap (\fs -> take 1 fs <> drop 3 fs) (replied (Req True us (own us) [] near))

-- | Over sets of up to three values spread across the u64 range and
-- salts 0..300: the shortcut signature equals the byte-by-byte fnv1a.
minhashLaw :: Bool
minhashLaw = and [elems (signatureOf 300 (sets [set]) 0) == [slow set i | i <- [0 .. 299 :: Int]] | set <- samples]
 where
  samples = [[0], [2 ^ (64 :: Int) - 1], [255, 256, 65537], [12345678901234567, 2 ^ (63 :: Int) + 9]]
  slow set i = minimum [fnv (bytes 8 x <> bytes 4 (toInteger i)) | x <- set]
  bytes w x = [fromInteger ((x `shiftR` (8 * k)) .&. 255) | k <- [0 .. w - 1]] :: [Word64]
  fnv = foldl (\h b -> (h `xor` b) * 1099511628211) (14695981039346656037 :: Word64)

-- | sizeBelow q mx ⇔ q < reachFloor mx over every q, mx in 0..300.
floorLaw :: Bool
floorLaw = and [sizeBelow q mx == (q < reachFloor mx) | mx <- [0 .. 300 :: Int], q <- [0 .. 300]]

-- | Over every pair of the seeded requests' units and every floor up to
-- the larger size: the floor-aware walk is below the floor exactly when
-- the full intersection is (the full one is the walk with floor 0).
floorWalk :: Bool
floorWalk =
  and
    [ (interUpTo us f a b < f) == (interUpTo us 0 a b < f)
    | r <- take 60 requests
    , let us = columns (rUnits r)
    , let n = length (rUnits r)
    , a <- [0 .. n - 1]
    , b <- [0 .. n - 1]
    , f <- [0 .. 70]
    ]

nonVacuous :: Bool
nonVacuous =
  all (\k -> any (positive k) requests) ["prunedSize", "prunedLabel", "crossLanguage", "s5New", "s5PrunedLabel", "s5Already", "unowned", "selfPairs", "printHot"]
    && all (\s -> any (hasSource s) requests) [0, 1, 2, 3 :: Integer]
 where
  counted r = case expected r of
    Right [_, Just raw, _, Just (Object c), _, _] -> Just (raw, c)
    _ -> Nothing
  positive k r = maybe False (\(_, c) -> maybe False (/= Number 0) (KM.lookup k c)) (counted r)
  hasSource s r = maybe False (\(raw, _) -> any (\row -> take 1 row == [s]) (asRows raw)) (counted r)
  asRows v = case v of
    Array xs -> [[n | Number n' <- elemsOf x, let n = truncate n'] | x <- foldr (:) [] xs]
    _ -> []
  elemsOf x = case x of
    Array ys -> foldr (:) [] ys
    _ -> []

-- | One case per line: units ; sigs ; prints ; near.
refusals :: [Req]
refusals = [Req False (read u) (read s) (read p) (read n) | [u, s, p, n] <- cells (unlines table)]
 where
  two = "[[0,0,0,1,1,1,0,1],[0,1,1,1,1,1,0,1]] ; [[1],[2]]"
  table =
    [ "[[-1,0,0,1,1,1,0,1]] ; [[1]] ; [] ; []"
    , "[[0,-1,0,1,1,1,0,1]] ; [[1]] ; [] ; []"
    , "[[0,0,-1,1,1,1,0,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0,0,1,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0,1,0,1,0,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0,1,3,2,0,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0,2,1,1,0]] ; [[1]] ; [] ; []"
    , "[[0,0,0,2,1,1,-1,2]] ; [[1]] ; [] ; []"
    , "[[0,0,0,2,1,1,0,0,1,2]] ; [[1]] ; [] ; []"
    , "[[0,0,0,2,1,1,1,1,0,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0,3,1,1,0,1,1,1]] ; [[1]] ; [] ; []"
    , "[[0,0,0]] ; [[1]] ; [] ; []"
    , "[[0,0,0,1,1,1,0,1]] ; [] ; [] ; []"
    , "[[0,0,0,1,1,1,0,1]] ; [[]] ; [] ; []"
    , "[[0,0,0,1,1,1,0,1]] ; [[-1]] ; [] ; []"
    , "[[0,0,0,1,1,1,0,1]] ; [[18446744073709551616]] ; [] ; []"
    , two <> " ; [[1,0,1]] ; []"
    , two <> " ; [[18446744073709551616,0,1,0]] ; []"
    , two <> " ; [[1,0,-1,0]] ; []"
    , two <> " ; [] ; [[0,1,1]]"
    , two <> " ; [] ; [[0,1,-1,1]]"
    ]

-- | At the unit cap exactly the core judges; one past it the reply is
-- the reference's degraded one. One key and one set per unit, so S2
-- and S4 pair nothing (the reference's comprehensions are quadratic:
-- it answers only the degraded side, which it decides by length alone).
unitCap :: Bool
unitCap =
  fieldsOf respond (request (atCap 0)) ["degraded"] == Just [Just (Bool False)]
    && agrees (atCap 1)
 where
  atCap extra = let us = [[0, 0, k, 1, 1, 1, 0, 1] | k <- [1 .. candidateUnitCap + extra]] in Req False us (own us) [] []

-- | The set, instance and near-run caps, counted without a request of
-- millions of rows on the wire: the contract's own overCap at each cap
-- and one past it.
capsCounted :: Bool
capsCounted =
  and
    [ not (overCap (at k cap)) && overCap (at k (cap + 1))
    | (k, cap) <- [(0 :: Int, candidateSigCap), (1, candidatePrintCap), (2, candidateNearCap)]
    ]
 where
  at k n = case k of
    0 -> CandReq Null False [[]] [replicate (fromInteger n) 0] [] []
    1 -> CandReq Null False [] [] (replicate (fromInteger n) []) []
    _ -> CandReq Null False [] [] [] (replicate (fromInteger n) [])
