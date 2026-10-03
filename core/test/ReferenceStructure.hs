-- | The structure family's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 04 read as definitions over the raw
-- request tables. Every axis is a predicate asked of each directory id
-- in turn (never a Map grouping); Tsallis-2 is the probability that
-- two draws land in DIFFERENT bins, summed over ordered bin pairs; the
-- χ² divergence is Σ p²/q − 1; modularity is Newman's ratio ρ =
-- q / qMax compared as a Rational against the per-mille floor (the
-- shipped side clears it to one integer inequality); ownership walks
-- each directory's parent chain; the file-name style is a rule over
-- the four style bits rather than a sixteen-entry table. Only the
-- family's Cost defaults are read.
module ReferenceStructure (SIn (..), diversity, expected, flagged, style) where

import CE.Structure.Cost
import Data.Aeson (Value, toJSON)
import Data.Bits (testBit)
import Data.List (nub, sort)
import Data.Ratio ((%))
import ReferenceSplit (splitAnswer)

-- | The request's tables as they ride.
data SIn = SIn
  { sNodes :: [[Integer]]
  , sShapes :: Maybe [[Integer]]
  , sConventions, sRefs, sDeclared :: [[Integer]]
  , sStaleDocs :: Maybe [[Integer]]
  , sStaleEdges :: [[Integer]]
  , sRedundancy, sDirEdges, sSeamFiles :: Maybe [[Integer]]
  , sSeams :: ([[Integer]], [[Integer]], [[Integer]], [[Integer]])
  , sKnobs :: [[Integer]]
  }

-- | The effective knob by code: the last declared row, else the
-- family's default.
knob :: SIn -> Integer -> Integer
knob inp code = last (dflt : [v | [c, v] <- sKnobs inp, c == code])
 where
  dflt = maybe 0 id (lookup code defaults)
  defaults =
    zip [0 ..] [depthCeil, fanoutCeil, namingMin, namingCeil, mixRefFloor, misplaceMin, bigDirFloor, structViolCost, structScale, dupMin, deadMin, staleMin, seamSoft, seamHard, seamPMax, roiRefMilli, roiPhiMilli, roiCloneMilli, roiChurnMilli, modFloor, modMassFloor]

-- | One stem's style from its seven facts: unclassifiable (6) and
-- digit-led (5) first; then dash with underscore, or both cases with a
-- separator, are mixed (6); both cases alone are pascal (3) or camel
-- (2) by the first char; upper alone is upper_snake (4) unless dashed;
-- otherwise a dash is kebab (1), an underscore or lowercase is snake
-- (0), and no letter and no separator is other (6).
style :: Integer -> Integer
style bits
  | on 6 = 6
  | on 4 = 5
  | otherwise = letters (on 0) (on 1) (on 2 && on 3) (on 3)
 where
  on = testBit bits
  letters under dash both upper
    | under && dash = 6
    | both = if under || dash then 6 else if on 5 then 3 else 2
    | upper = if dash then 6 else 4
    | dash = 1
    | under || on 2 = 0
    | otherwise = 6

-- | Gini–Simpson, normalized: the chance two draws differ over its
-- n-bin maximum, as a per-mille floor.
diversity :: [Integer] -> Integer
diversity cs
  | bins <= 1 = 0
  | otherwise = floor (1000 * differ / (1 - 1 % bins))
 where
  live = filter (> 0) cs
  bins = toInteger (length live)
  total = sum live
  differ = sum [(a * b) % (total * total) | (i, a) <- zip [0 :: Int ..] live, (j, b) <- zip [0 :: Int ..] live, i /= j]

-- | The name-pattern rows the axes read: the shape facts by style.
patterns :: SIn -> [[Integer]]
patterns inp =
  [ [d, c, sum [n | [d', b, n] <- rows, d' == d, style b == c]]
  | let rows = maybe [] id (sShapes inp)
  , (d, c) <- nub [(d, style b) | [d, b, _] <- rows]
  ]

-- | Each judged axis with its flagged directories, ascending.
flagged :: SIn -> [(Integer, [Integer])]
flagged inp =
  [(a, filter (holds a) dirs) | a <- [0 .. 4]]
    <> [(5, filter (stale rows) dirs) | Just rows <- [sStaleDocs inp]]
    <> [(6, [d | [d, dup, dead] <- rows, dup >= k 9 || dead >= k 10]) | Just rows <- [sRedundancy inp]]
    <> [(7, filter (unmodular es) dirs) | Just es <- [sDirEdges inp]]
 where
  k = knob inp
  dirs = [d | (d : _) <- sNodes inp]
  node d = head' [r | r@(i : _) <- sNodes inp, i == d]
  refsOf d = [(i, o, n) | [d', i, o, n] <- sRefs inp, d' == d]
  ins d = sum [i * n | (i, _, n) <- refsOf d]
  outs d = sum [o * n | (_, o, n) <- refsOf d]
  bits d = sum [b | [d', b] <- sConventions inp, d' == d]
  holds a d = case (a, node d) of
    (0, [_, _, depth, sub, files]) -> depth > k 0 || sub + files > k 1
    (1, _) -> let cs = [c | [d', _, c] <- patterns inp, d' == d] in sum cs >= k 2 && diversity cs > k 3
    (2, _) -> ins d + outs d >= k 4 && outs d > ins d
    (3, _) -> or [o >= k 5 && o > 2 * i | (i, o, _) <- refsOf d]
    (4, [_, _, _, _, files]) -> (files >= k 6 && even (bits d)) || (d == 0 && bits d < 2)
    _ -> False
  stale rows d =
    let docs = [(i, ts) | (i, [d', ts]) <- zip [0 ..] rows, d' == d]
        isStale (i, ts) = or [ts == 0 || t > ts | [j, t] <- sStaleEdges inp, j == i]
     in not (null docs) && toInteger (length (filter isStale docs)) >= k 11
  unmodular es d =
    let e = ins d `div` 2
        cross = sum [c | [a, b, c] <- es, a == d || b == d]
        out = e + sum [c | [a, _, c] <- es, a == d]
        inn = e + sum [c | [_, b, c] <- es, b == d]
        mu = e + cross
        m = sum [ins x `div` 2 | x <- dirs] + sum [c | [_, _, c] <- es]
     in mu >= k 20 && mu < m && (e * m - out * inn) % (mu * (m - mu)) < k 19 % 1000
  head' xs = case xs of
    (x : _) -> x
    [] -> []

-- | The A-layer keys when a layout rode: the divergence row (or none
-- while undeclared territory holds files) and the deviation rows.
declared :: SIn -> [Maybe Value]
declared inp
  | null (sDeclared inp) = [Nothing, Nothing]
  | otherwise = [Just (toJSON (if null unowned then [chi2] else [])), Just (toJSON (sort (map (\i -> [i, 0]) unowned <> [[d, 1] | [d, _] <- sDeclared inp, owned d == 0])))]
 where
  decl = [d | [d, _] <- sDeclared inp]
  parent i = sum (take 1 [p | [i', p, _, _, _] <- sNodes inp, i' == i])
  chain i = if i == 0 then [0] else i : chain (parent i)
  owner i = take 1 [d | d <- chain i, d `elem` decl]
  owned d = sum [f | [i, _, _, _, f] <- sNodes inp, owner i == [d]]
  unowned = [i | [i, _, _, _, f] <- sNodes inp, f > 0, null (owner i)]
  total = sum [owned d | d <- decl]
  weight = sum [w | [_, w] <- sDeclared inp]
  chi2 :: Integer
  chi2
    | total == 0 = 1000
    | otherwise = floor (1000 * (sum [((owned d % total) ^ (2 :: Int)) / (w % weight) | [d, w] <- sDeclared inp] - 1))

-- | The reply fields (axes, score, entropy, findings, divergence,
-- deviations, splitCandidates, sizeExempt, patternShapes); a degraded
-- request answers over no facts and the default knobs.
expected :: Bool -> SIn -> [Maybe Value]
expected isDegraded inp0 =
  [ Just (toJSON [[a, p] | (a, p) <- charges])
  , Just (toJSON (max 0 (k 8 - sum [p * k 7 | (_, p) <- charges] `div` (structViolCostNeutral * toInteger (length charges)))))
  , Just (toJSON [[0, diversity globalCodes], [1, diversity [f | [_, _, _, _, f] <- sNodes inp, f > 0]]])
  , Just (toJSON [[d, a] | (a, ds) <- axes, d <- ds])
  ]
    <> declared inp
    <> splitAnswer (k 12, k 13, k 14) (k 15, k 16, k 17, k 18) (sSeamFiles inp) (sSeams inp)
    <> [if isDegraded then Nothing else toJSON . length <$> sShapes inp]
 where
  inp = if isDegraded then SIn [] Nothing [] [] [] Nothing [] Nothing Nothing Nothing ([], [], [], []) [] else inp0
  k = knob inp
  axes = flagged inp
  dirCount = toInteger (length (sNodes inp))
  charges = [(a, density (toInteger (length ds))) | (a, ds) <- axes]
  density v = if v <= 0 || dirCount <= 0 then 0 else (k 8 * v) `div` (v + dirCount)
  globalCodes = [sum [n | [_, c', n] <- patterns inp, c' == c] | c <- nub [c | [_, c, _] <- patterns inp]]
