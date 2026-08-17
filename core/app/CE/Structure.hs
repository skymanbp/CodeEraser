-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | structure.request handler (M6 S2, design booklet §4): decode
-- the tree-scale fact tables — dense directory nodes, name-pattern
-- distributions, convention bits, per-file reference splits —
-- enforce the node cap (over-cap = a complete degraded reply that
-- FAILS, the P1 posture), machine-check the boundary contract in
-- request order, then judge the five S2 axes and the headline
-- entropy rows. Names and paths never cross (§5.9.2): the report's
-- vocabulary is dense ids, codes and counts, re-labelled by the
-- Rust side that kept the names. Knob rows ride the established
-- [code, value] grammar; ce.toml is the source, Cost.hs the
-- defaults, and the reply echoes the effective set whole.
module CE.Structure (respond) where

import CE.Structure.Axes (Facts (..), Knobs (..), axes, bound, entropyRows, findings)
import CE.Structure.Cost (structNodeCap)
import CE.Wire (Family (..), applyRows, ascendingOn, respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

data StructReq = StructReq
  { reqId :: Value
  , reqNodes :: [[Integer]]
  , reqPatterns :: [[Integer]]
  , reqConventions :: [[Integer]]
  , reqFileRefs :: [[Integer]]
  , reqKnobs :: [[Integer]]
  }

instance FromJSON StructReq where
  parseJSON = withObject "StructReq" $ \o ->
    StructReq
      <$> o .: "id"
      <*> o .: "nodes"
      <*> o .:? "patterns" .!= []
      <*> o .:? "conventions" .!= []
      <*> o .:? "fileRefs" .!= []
      <*> o .:? "knobs" .!= []

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "structure"
      , famId = reqId
      , famOverCap = \req -> toInteger (length (reqNodes req)) > structNodeCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req (effective []) True
      , famJudged = \req -> reply proto req (effective (reqKnobs req)) False
      }

-- | First boundary-contract offender in request order — the three
-- dir-keyed tables walk ONE loop over their spec rows (the twelfth
-- bite's repayment shape: the per-table asum/ascending pair was the
-- clone).
violation :: StructReq -> Maybe String
violation req =
  asum
    ( asum (zipWith nodeRow [0 :: Int ..] (reqNodes req))
        : [ asum
              [ asum (zipWith (dirRow n spec) [0 :: Int ..] rows)
              , ascendingOn nm proj rows
              ]
          | (spec@(_, nm, _), proj, rows) <- dirTables
          ]
        <> [knobsOffence (reqKnobs req)]
    )
 where
  n = toInteger (length (reqNodes req))
  dirTables =
    [ ((3, "pattern", patternOk), take 2, reqPatterns req)
    , ((2, "convention", convOk), take 1, reqConventions req)
    , ((4, "fileRefs", refsOk), take 3, reqFileRefs req)
    ]
  patternOk row = case row of
    [_, code, count] | code > 6 -> Just "unknown pattern code"
                     | count < 1 -> Just "count below 1"
                     | otherwise -> Nothing
    _ -> Nothing
  convOk row = case row of
    [_, bits] | bits < 1 || bits > 3 -> Just "bits outside 1..3"
              | otherwise -> Nothing
    _ -> Nothing
  refsOk row = case row of
    [_, _, _, count] | count < 1 -> Just "count below 1"
                     | otherwise -> Nothing
    _ -> Nothing

-- | One dense node row: id == index, parent < id (root 0 loops to
-- itself), depth == parent depth + 1 checked by position — the
-- shape that makes the tree a tree by construction.
nodeRow :: Int -> [Integer] -> Maybe String
nodeRow i row = case row of
  [nid, parent, depth, subdirs, files]
    | any (< 0) [nid, parent, depth, subdirs, files] -> Just (label <> "negative field")
    | nid /= toInteger i -> Just (label <> "index mismatch")
    | i == 0 && (parent /= 0 || depth /= 0) -> Just (label <> "root must self-loop at depth 0")
    | i > 0 && parent >= toInteger i -> Just (label <> "parent not before child")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [id,parent,depth,subdirs,files])")
 where
  label = "node " <> show i <> ": "

-- | Shared shape for the dir-keyed tables — the table's identity
-- travels as ONE spec tuple (arity, name, extra rule), which also
-- keeps the checker under the repo's own param gate.
dirRow :: Integer -> (Int, String, [Integer] -> Maybe String) -> Int -> [Integer] -> Maybe String
dirRow n (arity, name, extra) i row = case row of
  (d : _)
    | length row /= arity -> Just (label <> "malformed row")
    | any (< 0) row -> Just (label <> "negative field")
    | d >= n -> Just (label <> "dir out of range")
    | Just why <- extra row -> Just (label <> why)
    | otherwise -> Nothing
  [] -> Just (label <> "malformed row")
 where
  label = name <> " " <> show i <> ": "

knobsOffence :: [[Integer]] -> Maybe String
knobsOffence rows =
  asum [asum (zipWith one [0 :: Int ..] rows), ascendingOn "knob" (take 1) rows]
 where
  one i row = case row of
    [code, v]
      | code < 0 || code > 8 -> Just (label <> "unknown structure knob")
      | v < 1 -> Just (label <> "knob below 1")
      | otherwise -> Nothing
    _ -> Just (label <> "malformed row (need [code,value])")
   where
    label = "knob " <> show i <> ": "

-- | ONE knob authority: (code, getter, setter) — the effective fold
-- and the reply's echo read the SAME table, so a knob cannot exist
-- in one direction only (the weights lesson, designed in).
knobTable :: [(Integer, Knobs -> Integer, Integer -> Knobs -> Knobs)]
knobTable =
  [ (0, kDepthCeil, \v k -> k {kDepthCeil = v})
  , (1, kFanoutCeil, \v k -> k {kFanoutCeil = v})
  , (2, kNamingMin, \v k -> k {kNamingMin = v})
  , (3, kNamingCeil, \v k -> k {kNamingCeil = v})
  , (4, kMixRefFloor, \v k -> k {kMixRefFloor = v})
  , (5, kMisplaceMin, \v k -> k {kMisplaceMin = v})
  , (6, kBigDirFloor, \v k -> k {kBigDirFloor = v})
  , (7, kViolCost, \v k -> k {kViolCost = v})
  , (8, kScale, \v k -> k {kScale = v})
  ]

-- | Knob rows over the Cost defaults (the effectiveKnobs pattern):
-- absent rows keep the defaults; the fold grammar is CE.Wire's.
effective :: [[Integer]] -> Knobs
effective rows = applyRows [(c, s) | (c, _, s) <- knobTable] rows bound

-- | The judged reply: five axes, the Score.hs fold at equal weight
-- over the judged axis count, the headline entropy rows and the
-- sparse findings — plus the FULL effective knob echo (table form).
-- fail = degraded alone in S2 (the report-only stance: the CLI
-- gates nothing until a score floor lands with S3+); a degraded
-- reply carries fail=true (P1) and echoes the defaults.
reply :: String -> StructReq -> Knobs -> Bool -> B8.ByteString
reply proto req k degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("structure.result" :: String)
    , "id" .= reqId req
    , "axes" .= [[c, p] | (c, p) <- pens]
    , "score" .= score
    , "entropy" .= entropyRows facts
    , "findings" .= findings k facts
    , "fail" .= degraded
    , "knobs" .= [[c, g k] | (c, g, _) <- knobTable]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("structure_too_large" :: String) | degraded]
 where
  facts =
    if degraded
      then Facts [] [] [] []
      else Facts (reqNodes req) (reqPatterns req) (reqConventions req) (reqFileRefs req)
  pens = axes k facts
  raw = sum [p * kViolCost k | (_, p) <- pens]
  score = max 0 (kScale k - raw `div` toInteger (length pens))
