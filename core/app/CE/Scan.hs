-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan.request handler (ADR-008 P3): decode measurement rows
-- [code, value] plus optional grade overrides [code, warn, fail],
-- enforce the row cap (over-cap = a complete degraded reply that
-- FAILS — the P1 posture), machine-check the boundary contract in
-- request order — then grade every row through the ONE graded
-- verdict table. Levels return positionally (row i answers level i)
-- and the fail bit is the exit-code semantic: any level-2 row.
-- Measurement and report rendering stay in Rust; only codes and
-- values cross the wire — subjects, names and paths never do
-- (§5.9.2 index privacy).
module CE.Scan (respond) where

import CE.Scan.Cost (gradeTable, gradeWith, scanRowCap)
import CE.Wire (Family (..), ascendingOn, respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

data ScanReq = ScanReq
  { reqId :: Value
  , reqRows :: [[Integer]]
  , reqGrades :: [[Integer]]
  }

instance FromJSON ScanReq where
  parseJSON = withObject "ScanReq" $ \o ->
    ScanReq <$> o .: "id" <*> o .: "rows" <*> o .:? "grades" .!= []

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "scan"
      , famId = reqId
      , -- grades count toward the cap too (review C15: the declared
        -- ceiling missed the second request dimension)
        famOverCap = \req ->
          toInteger (length (reqRows req) + length (reqGrades req)) > scanRowCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req [] True
      , famJudged = \req ->
          let eff = effective (reqGrades req)
           in reply proto req (map (grade eff) (reqRows req)) False
      }

-- | First boundary-contract offender in request order (Clone.hs
-- posture: the message names the violator deterministically); the
-- ascending pass compares CODES alone (warn values legitimately
-- vary), through CE.Wire's shared checker.
violation :: ScanReq -> Maybe String
violation req =
  asum
    [ asum (zipWith rowShape [0 :: Int ..] (reqRows req))
    , asum (zipWith gradeShape [0 :: Int ..] (reqGrades req))
    , ascendingOn "grade" (take 1) (reqGrades req)
    ]

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [code, v]
    | code < 0 || code > 6 -> Just (label <> "unknown metric code")
    | v < 0 -> Just (label <> "negative value")
    | v >= 18446744073709551616 -> Just (label <> "value outside u64")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [code,value])")
 where
  label = "row " <> show i <> ": "

-- | A grade override must stay a coherent ladder: a hard line BELOW
-- the warn line is refused, never silently reordered. fail == warn
-- is deliberately legal (review C19 ruling): it is the single-line
-- config — every breach grades 2 and the warn band is empty by the
-- user's own choice, which gradeWith honors on both sides of the
-- mirror.
gradeShape :: Int -> [Integer] -> Maybe String
gradeShape i row = case row of
  [code, warn, failLine]
    | code < 0 || code > 6 -> Just (label <> "unknown metric code")
    | warn < 0 || failLine < 0 -> Just (label <> "negative field")
    | failLine /= 0 && failLine < warn -> Just (label <> "fail line below warn")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [code,warn,fail])")
 where
  label = "grade " <> show i <> ": "

-- | The effective grade table: every default row, overridden per
-- code by the request (the effectiveKnobs pattern — absent rows
-- keep the Cost.hs DEFAULTS; ce.toml is the source on the Rust
-- side, this wire is the road).
effective :: [[Integer]] -> [(Integer, (Integer, Integer))]
effective overrides =
  [(c, pick c (w, f)) | (c, w, f) <- gradeTable]
 where
  pick c dflt = last (dflt : [(w, f) | [c', w, f] <- overrides, c' == c])

grade :: [(Integer, (Integer, Integer))] -> [Integer] -> Integer
grade table row = case row of
  [code, v] | Just wf <- lookup code table -> gradeWith wf v
  _ -> error "row shape enforced by violation"

-- | levels ride positionally; the effective grade table is echoed
-- whole so the Rust client asserts the round trip (the P4 knob-echo
-- pattern, table form). A degraded reply carries fail=true — a gate
-- that could not judge must never pass, said by the core.
reply :: String -> ScanReq -> [Integer] -> Bool -> B8.ByteString
reply proto req levels degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("scan.result" :: String)
    , "id" .= reqId req
    , "levels" .= levels
    , "counts"
        .= object
          [ "rows" .= length (reqRows req)
          , "warns" .= count 1
          , "fails" .= count 2
          ]
    , "fail" .= (degraded || count 2 > 0)
    , -- a degraded reply echoes the DEFAULTS: its overrides were
      -- never validated, and an unvalidated table must not be
      -- presented as effective (review C14; the Verdict tooLarge
      -- posture)
      "grades" .= [[c, w, f] | (c, (w, f)) <- effective (if degraded then [] else reqGrades req)]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("scan_too_large" :: String) | degraded]
 where
  count l = length (filter (== l) levels)
