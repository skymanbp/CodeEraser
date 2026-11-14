-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | cover.request handler (v2.35 imports lane, R2): the T1/T2 report's
-- selection over the verified token runs. The measuring side finds the
-- runs — anchors from the winnowed fingerprints, verified by exact
-- extension over the token streams (cli/src/dedup/pairs.rs) — and sends
-- them with the run's two filters; this family answers which are
-- reported, which suppressed and which dropped (CE.Dedup.Cover.Rule).
--
--   minTokens     t, the report threshold (>= 1)
--   minDistinct   d, the diversity floor (>= 0; 0 = the floor is off)
--   runs          [fileA, fileB, a0, b0, len, distinct], file codes by
--                 path rank, strictly ascending on (fileA, fileB, a0, b0)
--
-- Pairs are independent, so the measuring side may split its run set
-- between pairs under the cap and the answers concatenate exactly; a
-- pair is never split. Every offence is a named refusal, the cap
-- included: a request over it is a planning fault of the measuring
-- side, not a degraded judgment. No knob — t and d are facts of the run.
module CE.Dedup.Cover (respond) where

import CE.Dedup.Cost (coverRowCap)
import CE.Dedup.Cover.Rule (Class (..), Run (..), classify)
import CE.Wire (Respond, rowCheck, tableOffence, tabled)
import qualified CE.Wire.Families as F
import Data.Aeson (FromJSON (..), Value, withObject, (.:), (.=))
import Data.Aeson.Types (Pair)
import Data.Foldable (asum)

data CoverReq = CoverReq
  { reqId :: Value
  , minTokens :: Integer
  , minDistinct :: Integer
  , runs :: [[Integer]]
  }

instance FromJSON CoverReq where
  parseJSON = withObject "CoverReq" $ \o ->
    CoverReq <$> o .: "id" <*> o .: "minTokens" <*> o .: "minDistinct" <*> o .: "runs"

respond :: Respond
respond = tabled F.cover reqId (const False) offence answer

-- | The cap, the two filters, then each run's shape and the table's
-- order — the first offender by name.
offence :: CoverReq -> Maybe String
offence r =
  asum
    [ if rows > coverRowCap then Just ("runs: " <> show rows <> " rows over the cap of " <> show coverRowCap) else Nothing
    , if minTokens r < 1 then Just "minTokens: below 1" else Nothing
    , if minDistinct r < 0 then Just "minDistinct: negative" else Nothing
    , tableOffence "runs" (take 4) (rowCheck "runs" "malformed run (need [fileA,fileB,a0,b0,len,distinct])" 6 runRow) (runs r)
    ]
 where
  rows = toInteger (length (runs r))

runRow :: [Integer] -> Maybe String
runRow row = case row of
  [fa, fb, a0, b0, len, distinct]
    | fa < 0 || a0 < 0 || b0 < 0 -> Just "negative element"
    | fa > fb -> Just "fileA after fileB"
    | len < 1 -> Just "len below 1"
    | distinct < 1 || distinct > len -> Just "distinct outside 1..len"
    | fa == fb && a0 + len > b0 -> Just "a same-file run overlaps itself (a0 + len > b0)"
  _ -> Nothing

-- | The cover.result object: the row indices of each class, ascending
-- (together every index once), the counts, and the two filters echoed.
answer :: Bool -> CoverReq -> ([Pair], [Pair])
answer _ r =
  ( ["reported" .= indicesOf Reported, "suppressed" .= indicesOf Suppressed, "dropped" .= indicesOf Dropped, "minTokens" .= minTokens r, "minDistinct" .= minDistinct r]
  , ["rows" .= length classes, "reported" .= length (indicesOf Reported), "suppressed" .= length (indicesOf Suppressed), "dropped" .= length (indicesOf Dropped)]
  )
 where
  classes = classify (minTokens r) (minDistinct r) [Run fa fb a0 b0 n dist | [fa, fb, a0, b0, n, dist] <- runs r]
  indicesOf c = [i | (i, k) <- zip [0 :: Int ..] classes, k == c]
