{-# LANGUAGE BangPatterns #-}

-- | Deterministic MinHash and LSH banding over shingle sets (design
-- vol.1 §1.1, §5.3; plan v2.33 W3) — the ONE implementation the T3
-- structural source S4 and the docdup coarse filter both read, so the
-- two estimators cannot drift. No RNG and no clock: permutation i IS
-- the salt — the element's eight little-endian bytes followed by i's
-- four, hashed with fnv1a64; a band's key is the fnv1a64 of its rows'
-- little-endian bytes. Until W3 the measuring side computed both.
--
-- Cost: hashing x ‖ i is the element's eight-byte prefix hashed once,
-- then four more fnv steps per permutation; for i < 256 the three high
-- bytes of i are zero, and a zero byte's step is a bare multiply, so
-- the four collapse into one xor and one multiply by the prime's fourth
-- power — the same 64-bit value, one step per (element, permutation).
module CE.Candidates.Lsh (Sets (..), sets, bandBuckets, signatureOf) where

import Control.Monad (forM_)
import Control.Monad.ST (ST, runST)
import Data.Array.Base (unsafeAt, unsafeFreeze, unsafeRead, unsafeWrite)
import Data.Array.ST (STUArray, newArray)
import Data.Array.Unboxed (UArray, listArray)
import Data.Bits (shiftR, xor, (.&.))
import qualified Data.IntMap.Strict as IM
import Data.Word (Word64)

-- | Many shingle sets in one unboxed array: set i is entries
-- [offsets ! i, offsets ! (i + 1)) of `elems`.
data Sets = Sets {setCount :: !Int, offsets :: !(UArray Int Int), elems :: !(UArray Int Word64)}

-- | The sets of a request's rows (every element within u64: the
-- contract admitted them).
sets :: [[Integer]] -> Sets
sets rows = Sets n (listArray (0, n) (scanl (+) 0 (map length rows))) (listArray (0, total - 1) (map fromInteger (concat rows)))
 where
  n = length rows
  total = sum (map length rows)

prime, basis, prime4 :: Word64
prime = 1099511628211
basis = 14695981039346656037
prime4 = prime * prime * prime * prime

-- | fnv1a continued over the eight little-endian bytes of a word.
absorb :: Word64 -> Word64 -> Word64
absorb h0 x = go 0 h0
 where
  go :: Int -> Word64 -> Word64
  go !k !h
    | k == 8 = h
    | otherwise = go (k + 1) ((h `xor` ((x `shiftR` (8 * k)) .&. 255)) * prime)

-- | fnv1a continued over the four little-endian bytes of salt i.
salted :: Word64 -> Int -> Word64
salted h i
  | i < 256 = (h `xor` fromIntegral i) * prime4
  | otherwise = foldl (\acc k -> (acc `xor` ((fromIntegral i `shiftR` (8 * k)) .&. 255)) * prime) h [0 .. 3 :: Int]

-- | sig[i] = min over the set's elements x of fnv1a(x ‖ i), for i below
-- `perms`; an empty set gives u64::MAX rows.
signatureOf :: Int -> Sets -> Int -> UArray Int Word64
signatureOf perms ss s = runST $ do
  sig <- newArray (0, perms - 1) maxBound :: ST st (STUArray st Int Word64)
  forM_ [offsets ss `unsafeAt` s .. offsets ss `unsafeAt` (s + 1) - 1] $ \e -> do
    let hx = absorb basis (elems ss `unsafeAt` e)
        go !i
          | i >= perms = pure ()
          | otherwise = do
              old <- unsafeRead sig i
              let v = salted hx i
              if v < old then unsafeWrite sig i v else pure ()
              go (i + 1)
    go 0
  unsafeFreeze sig

-- | The key of band b: fnv1a over its `rows` signature words.
bandKey :: Int -> UArray Int Word64 -> Int -> Word64
bandKey rows sig b = foldl (\h r -> absorb h (sig `unsafeAt` (b * rows + r))) basis [0 .. rows - 1]

-- | Every bucket of more than one member, per band in band order and
-- by key within a band, each bucket's members ascending — the sets
-- whose signatures agree on some band (the LSH candidate condition).
-- Only the sets `admit` keeps are hashed.
bandBuckets :: (Int, Int, Int) -> (Int -> Bool) -> Sets -> [[Int]]
bandBuckets (perms, bands, rows) admit ss =
  [ members
  | band <- IM.elems byBand
  , members <- IM.elems band
  , length members > 1
  ]
 where
  keyed =
    [ (b, (fromIntegral (bandKey rows sig b) :: Int, s))
    | s <- reverse (filter admit [0 .. setCount ss - 1])
    , let sig = signatureOf perms ss s
    , b <- [0 .. bands - 1]
    ]
  byBand = IM.fromListWith (IM.unionWith (++)) [(b, IM.singleton k [s]) | (b, (k, s)) <- keyed]
