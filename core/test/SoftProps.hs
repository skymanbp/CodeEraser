-- | The soft line read in log space (9.0.0): S = ⌊m·r^k⌋ with m the
-- log-space median of the LOC multiset and r the log-space median of
-- every value's ratio to m, an even count's middles taken
-- geometrically. Three readings hold the product to that definition
-- over seeded multisets (sizes 1..200, LOC 1..4000, three seeds of
-- 2,000 each, both parities): the definition itself in Rational
-- fourth powers brackets S; an odd count reproduces the pre-9.0.0
-- arithmetic formula exactly (one middle, nothing to average); and a
-- Double spelling of median + k·MAD over log x lands within one.
module SoftProps (battery) where

import CE.Verdict.Soft (median, softLine)
import Data.List (sort)
import Data.Ratio ((%))
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("6,000 LOC multisets: S⁴ <= m⁴·r^(4k) < (S+1)⁴, the definition in Rational fourth powers", all bracketed samples)
    , ("odd counts: S equals the pre-9.0.0 floor (m * spread ^ k) over the arithmetic median", all oddAgrees (filter (odd . length . snd) samples))
    , ("a Double spelling (exp of median + k·MAD over log x) agrees within one on every sample", all logAgrees samples)
    , ("both parities are sampled", any (even . length . snd) samples && any (odd . length . snd) samples)
    , ("{150, 200, 250, 300} at k = 2 derives 335 (the arithmetic middles gave 339, lenient)", handCase [150, 200, 250, 300] 335 339)
    , ("{34, 57, 78, 183, 222, 224} at k = 2 derives 416 (the arithmetic middles gave 381, strict)", handCase [34, 57, 78, 183, 222, 224] 416 381)
    ]

-- | (k, LOC multiset): k alternates 1 and 2, the calibrated default.
samples :: [(Integer, [Integer])]
samples = concatMap (\seed -> take 2000 (draws (stream seed))) [11, 4242, 987654321]
 where
  draws (a : b : rest) =
    let size = fromInteger (1 + a `mod` 200)
        (locs, more) = splitAt size rest
     in (1 + b `mod` 2, map (\v -> 1 + v `mod` 4000) locs) : draws more
  draws _ = []

-- | A 64-bit linear congruential stream, its top bits only.
stream :: Integer -> [Integer]
stream = map (`div` 65536) . drop 1 . iterate (\x -> (x * 6364136223846793005 + 1442695040888963407) `mod` 18446744073709551616)

unclamped :: Integer -> [Integer] -> Integer
unclamped k xs = maybe (-1) id (softLine k 0 (10 ^ (40 :: Int)) xs)

-- | m⁴·r^{4k} from the definition: m² the product of the two middles
-- of the sorted values, r⁴ the product of the two middles of the sorted
-- folded squared ratios ρ² = max(x²/m², m²/x²).
fourth :: Integer -> [Integer] -> Rational
fourth k xs = mm * mm * (pick rho * pick' rho) ^ k
 where
  sorted = sort (map (% 1) xs)
  n = length xs
  pick vs = vs !! ((n - 1) `div` 2)
  pick' vs = vs !! (n `div` 2)
  mm = pick sorted * pick' sorted
  rho = sort [max (v * v / mm) (mm / (v * v)) | v <- sorted]

bracketed :: (Integer, [Integer]) -> Bool
bracketed (k, xs) = s4 s <= q && q < s4 (s + 1)
 where
  s = unclamped k xs
  q = fourth k xs
  s4 v = fromInteger v ^ (4 :: Int)

oddAgrees :: (Integer, [Integer]) -> Bool
oddAgrees (k, xs) = unclamped k xs == floor (m * spread ^ k)
 where
  vs = map (% 1) xs
  m = median vs
  spread = median [max (v / m) (m / v) | v <- vs]

logAgrees :: (Integer, [Integer]) -> Bool
logAgrees (k, xs) = abs (unclamped k xs - floor (exp (mid + fromInteger k * mad))) <= 1
 where
  logs = map (log . fromInteger) xs :: [Double]
  mid = middle logs
  mad = middle [abs (l - mid) | l <- logs]
  middle ls =
    let s = sort ls
        n = length s
     in (s !! ((n - 1) `div` 2) + s !! (n `div` 2)) / 2

-- | An even count at k = 2, unclamped: the definition's S and the
-- pre-9.0.0 arithmetic reading. The two cases miss in opposite
-- directions — the arithmetic m sits above the geometric one, but r
-- is then measured around a different centre, so the old S was not
-- one-sidedly lenient.
handCase :: [Integer] -> Integer -> Integer -> Bool
handCase locs geometric old = softLine 2 1 100000 locs == Just geometric && arithmetic == old
 where
  vs = map (% 1) locs :: [Rational]
  m = median vs
  arithmetic = floor (m * median [max (v / m) (m / v) | v <- vs] ^ (2 :: Int)) :: Integer
