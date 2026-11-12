-- | The split-ROI advisory's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 08 read as definitions. Each seam (the
-- gap after every unit but the last) is priced by COUNTING — a pair
-- crosses seam u when min ≤ u < max, a clone block [s, e) is cut when
-- s ≤ line < e — where the shipped pricer folds difference maps and
-- running sums; the best seam is the one no other seam out-earns by
-- cross-multiplied ROI, the LAST of equals (the shipped
-- maximumBy's tie). Benefit rides the zone curve read as one fraction
-- (refZone, the second implementation of CE.Structure.Zone).
module ReferenceSplit (refZone, splitAnswer) where

import Data.Aeson (Value, toJSON)
import Data.Ratio ((%))

-- | splitCandidates and sizeExempt, present exactly when seamFiles
-- rode: one row per file in file order.
splitAnswer ::
  (Integer, Integer, Integer) ->
  (Integer, Integer, Integer, Integer) ->
  Maybe [[Integer]] ->
  ([[Integer]], [[Integer]], [[Integer]], [[Integer]]) ->
  [Maybe Value]
splitAnswer zone prices files tables = case files of
  Nothing -> [Nothing, Nothing]
  Just fs ->
    let rows = [priceFile zone prices tables f total | [f, total] <- fs]
     in [Just (toJSON [r | Left r <- rows]), Just (toJSON [r | Right r <- rows])]

-- | Left a candidate [file, unit, benefit, cost]; Right an exemption
-- [file, benefit, cost] (0, 0 when the file has no seam).
priceFile ::
  (Integer, Integer, Integer) ->
  (Integer, Integer, Integer, Integer) ->
  ([[Integer]], [[Integer]], [[Integer]], [[Integer]]) ->
  Integer ->
  Integer ->
  Either [Integer] [Integer]
priceFile (s, h, pMax) (refP, phiP, cloneP, churnP) (units, refs, clones, churn) f total =
  case [p | p <- priced, all (\q -> gain p * price q >= gain q * price p) priced] of
    [] -> Right [f, 0, 0]
    best -> case last best of
      (u, b, c)
        | b >= c -> Left [f, u, b, c]
        | otherwise -> Right [f, b, c]
 where
  mine = [(u, e) | [f', u, _, e] <- units, f' == f]
  seams = take (length mine - 1) mine
  pairsOf t = [(a, b) | [f', a, b] <- t, f' == f]
  blocks = [(bs, be) | [f', bs, be] <- clones, f' == f]
  crossing u = toInteger . length . filter (\(a, b) -> min a b <= u && u < max a b)
  cut line = toInteger (length [() | (bs, be) <- blocks, bs <= line, line < be])
  z = refZone s h pMax
  priced =
    [ (u, max 0 (floor (1000 * (z total - z end - z (total - end)))), crossing u (pairsOf refs) * refP + cut end * cloneP + crossing u (pairsOf churn) * churnP + phiP)
    | (u, end) <- seams
    ]
  gain (_, b, _) = b
  price (_, _, c) = c

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
