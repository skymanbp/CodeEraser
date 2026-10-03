-- | The file-name pattern classifier (7.2.0, plan v2.30 step 7b): the
-- S1 axis judges a directory's file-name STYLE distribution, and the
-- style of one name is a rule over seven character-class facts of its
-- stem. Rust measured the facts AND applied the rule until this step
-- (structure/tree.rs, the STYLE table); now the facts ride the wire as
-- `patternShapes` rows [dirId, bits, count] and the rule is this
-- module's. The legacy `patterns` road — codes pre-classified by the
-- producer — kept its bytes until 8.0.0 retired it (plan v2.32 step 6;
-- CE.Structure refuses it by name).
module CE.Structure.Shape (foldShapes, shapeBitsCap, shapeCode, styleTable) where

import Data.Bits (testBit, (.&.))
import qualified Data.Map.Strict as M

-- | The seven stem facts, one bit each (frozen positions; the low
-- four are the style key): 0 underscore present / 1 dash present /
-- 2 lowercase letter present / 3 uppercase letter present /
-- 4 digit-led / 5 first char uppercase / 6 unclassifiable (an empty
-- stem, or a char outside letters, digits, dash and underscore).
shapeBitsCap :: Integer
shapeBitsCap = 127

-- | The style decision as a table indexed by the low four bits
-- (upper<<3 | lower<<2 | dash<<1 | under): 0 lower_snake / 1
-- lower_kebab / 2 camel / 3 pascal / 4 upper_snake / 5 digit_led /
-- 6 other. Key 12 (letters of both cases, no separator) reads 3 and
-- splits to camel by the first-char bit in shapeCode; every
-- mixed-signal key answers 6. The producer's own scan gate caught the
-- fused (CC 25), nested (CoC 16) and flat-guard (CC 18) forms of this
-- decision before it became data (tree.rs, 2026-08).
styleTable :: [Integer]
styleTable = [6, 0, 1, 6, 0, 0, 1, 6, 4, 4, 6, 6, 3, 6, 6, 6]

-- | One stem's pattern code from its shape bits: unclassifiable and
-- digit-led first, then the style key, with the pascal/camel split
-- on the first char — the same order the producer applied.
shapeCode :: Integer -> Integer
shapeCode bits
  | testBit bits 6 = 6
  | testBit bits 4 = 5
  | code == 3 && not (testBit bits 5) = 2
  | otherwise = code
 where
  code = styleTable !! fromInteger (bits .&. 15)

-- | [dirId, bits, count] rows folded to the [dirId, code, count]
-- distribution the axes read: counts summed per (dir, code),
-- ascending — the rows a pre-7.2.0 producer classified for itself.
foldShapes :: [[Integer]] -> [[Integer]]
foldShapes rows =
  [ [d, c, n]
  | ((d, c), n) <- M.toAscList (M.fromListWith (+) [((d, shapeCode b), n) | [d, b, n] <- rows])
  ]
