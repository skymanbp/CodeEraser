-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The moves.request shape and its boundary contract (plan v2.33 W3,
-- fourclass L1). The measuring side keeps the text: per changed file
-- pair it sends each side's lines as [content code, trimmed-content
-- fnv1a64, alphanumeric width] (equal code = equal trimmed text; width 0
-- = an insignificant line), the 0-based lines its line
-- diff changed on that side, and the side's units as [key, kind, start,
-- end, stack] — key a request-local code (codes rank the key texts, so
-- code order is text order), the span 1-based inclusive, stack 1 when
-- the key may carry stacking identity. `keys` maps each code to the
-- key's fnv1a64. The first offender in request order is named.
module CE.FourClass.Moves.Contract (MovesReq (..), PairIn (..), Side (..), overCap, offence) where

import CE.FourClass.Moves.Cost (movesLineCap, movesUnitCap)
import CE.Wire (rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Foldable (asum)

-- | One side of a pair: its line rows, its changed line indices, its
-- unit rows.
data Side = Side {sideLines, sideUnits :: [[Integer]], sideChanged :: [Integer]}

instance FromJSON Side where
  parseJSON = withObject "Side" $ \o ->
    Side <$> o .:? "lines" .!= [] <*> o .:? "units" .!= [] <*> o .:? "changed" .!= []

data PairIn = PairIn {pairBefore, pairAfter :: Side}

instance FromJSON PairIn where
  parseJSON = withObject "PairIn" $ \o -> PairIn <$> o .: "before" <*> o .: "after"

data MovesReq = MovesReq {reqId :: Value, keyHashes :: [Integer], pairsIn :: [PairIn]}

instance FromJSON MovesReq where
  parseJSON = withObject "MovesReq" $ \o ->
    MovesReq <$> o .: "id" <*> o .:? "keys" .!= [] <*> o .:? "pairs" .!= []

-- | Every side of every pair, labelled for the refusals.
sides :: MovesReq -> [(String, Side)]
sides req =
  concat
    [ [("pair " <> show p <> " before", pairBefore x), ("pair " <> show p <> " after", pairAfter x)]
    | (p, x) <- zip [0 :: Int ..] (pairsIn req)
    ]

-- | Lines over their ceiling, or units over theirs, across every side.
overCap :: MovesReq -> Bool
overCap req =
  total sideLines > movesLineCap || total sideUnits > movesUnitCap
 where
  total f = toInteger (sum [length (f s) | (_, s) <- sides req])

-- | The keys, then each side in request order: its lines, its changed
-- indices (in range, strictly ascending), its units.
offence :: MovesReq -> Maybe String
offence req =
  asum
    ( asum (zipWith keyShape [0 :: Int ..] (keyHashes req))
        : [sideOffence (length (keyHashes req)) label s | (label, s) <- sides req]
    )

keyShape :: Int -> Integer -> Maybe String
keyShape i h
  | word h = Nothing
  | otherwise = Just ("key " <> show i <> ": outside u64")

sideOffence :: Int -> String -> Side -> Maybe String
sideOffence nKeys label s =
  asum
    [ asum (zipWith (lineShape label) [0 ..] (sideLines s))
    , tableOffence (label <> " changed") id (inRange (length (sideLines s))) (sideChanged s)
    , asum (zipWith (unitShape label nKeys) [0 ..] (sideUnits s))
    ]
 where
  inRange n i x
    | x < 0 || x >= toInteger n = Just (label <> " changed " <> show i <> ": out of range")
    | otherwise = Nothing

-- | [code, hash, width]: the code non-negative, the hash a u64, the
-- width non-negative.
lineShape :: String -> Int -> [Integer] -> Maybe String
lineShape label = rowCheck (label <> " line") "malformed line (need [code,hash,width])" 3 checks
 where
  checks row = case row of
    [c, h, w]
      | c < 0 -> Just "negative content code"
      | not (word h) -> Just "hash outside u64"
      | w < 0 -> Just "negative width"
    _ -> Nothing

-- | [key, kind, start, end, stack]: the key a code, the kind
-- non-negative, the span ordered from line 1, stack a bit.
unitShape :: String -> Int -> Int -> [Integer] -> Maybe String
unitShape label nKeys = rowCheck (label <> " unit") "malformed unit (need [key,kind,start,end,stack])" 5 checks
 where
  checks row = case row of
    [key, kind, from, to, stack]
      | key < 0 || key >= toInteger nKeys -> Just "key outside the key table"
      | kind < 0 -> Just "negative kind"
      | from < 1 || to < from -> Just "span not 1-based and ordered"
      | stack < 0 || stack > 1 -> Just "stack not a bit"
    _ -> Nothing

word :: Integer -> Bool
word x = x >= 0 && x < 2 ^ (64 :: Int)
