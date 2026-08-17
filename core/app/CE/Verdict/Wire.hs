-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | verdict.request shape and boundary contract (design §2.2), split
-- from CE.Verdict at the 300-line core gate. The tier table is the
-- node universe (dense: row i names node i); every fact table
-- indexes into it, strictly ascending on its IDENTITY PREFIX (a
-- duplicate identity with a different payload is exactly the drift
-- this refuses to let in), and the first offender is reported in a
-- deterministic order: universe first, then facts as the request
-- lists them (the Graph.hs convention).
module CE.Verdict.Wire
  ( VerdictReq (..)
  , violation
  , parseBaseline
  ) where

import CE.Verdict.Cost (scoreScale)
import CE.Verdict.Ratchet (Baseline (..))
import CE.Verdict.Table
  ( ascendingBy
  , ceilingsOffence
  , label
  , table
  , thresholdsOffence
  , toleranceOffence
  , weightsOffence
  )
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.Aeson.Types as AT
import Data.Foldable (asum)
import qualified Data.IntSet as IS

data VerdictReq = VerdictReq
  { reqId :: Value
  , reqSim :: [[Integer]]
  , reqPos :: [[Integer]]
  , reqTier :: [[Integer]]
  , reqChurn :: [[Integer]]
  , reqCochange :: [[Integer]]
  , reqCont :: [[Integer]]
  , reqDisc :: [Integer]
  , reqBaseline :: Value
  , reqWeights :: [[Integer]]
  , reqFloor :: Maybe Integer
  , -- ADR-008 first step (2.3.0, additive — absent parses as []):
    -- [axisCode, ceiling] rows overriding the size (0) and coc (1)
    -- axis ceilings; ce.toml is the source, this wire is the road,
    -- and the Cost.hs values become DEFAULTS instead of the second
    -- half of an uncheckable mirror (M5-close audit D2)
    reqCeilings :: [[Integer]]
  , -- ADR-008 P4 (2.4.0, additive): the remaining verdict-family
    -- knobs speak the same [code, value] grammar — thresholds codes
    -- 0..6 (deadIndegCeil / rewriteNum / rewriteDen / cochangeFloor
    -- / violCost / defaultWeight / scoreScale) and the ADR-006
    -- tolerance legs 0..2 (tolNum / tolDen / tolAbs). Absent = []
    -- = every knob at its Cost.hs DEFAULT.
    reqThresholds :: [[Integer]]
  , reqTolerance :: [[Integer]]
  , -- ADR-008 P2 (2.6.0, additive): the dedup budget pair
    -- [blocks, budget] — the second ratchet's verdict inputs, sent
    -- by `ce dedup --check` alone. Absent = the condition is not
    -- evaluated (the ce check road is untouched).
    reqDedup :: Maybe [Integer]
  }

instance FromJSON VerdictReq where
  parseJSON = withObject "VerdictReq" $ \o ->
    VerdictReq
      <$> o .: "id"
      <*> o .: "sim"
      <*> o .: "pos"
      <*> o .: "tier"
      <*> o .: "churn"
      <*> o .: "cochange"
      <*> o .: "continuous"
      <*> o .: "discrete"
      <*> o .: "baseline"
      <*> o .: "weights"
      <*> o .: "floor"
      <*> o .:? "ceilings" .!= []
      <*> o .:? "thresholds" .!= []
      <*> o .:? "tolerance" .!= []
      <*> o .:? "dedup"

-- | First boundary-contract offender, if any. The row checkers are
-- top-level functions taking the universe size n (the M5-close warn
-- repayment: a 64-line where block was the E01 offender, and the
-- checkers never needed the closure — only n and the tier table).
-- The baseline arrives PRE-PARSED: CE.Verdict parses it exactly once
-- and both the row cap and this check consume that result (the
-- M5-close LOW "parseBaseline runs twice per request", repaid
-- together with the baseline cap escape).
violation :: Either String (Maybe Baseline) -> VerdictReq -> Maybe String
violation parsed req =
  asum
    [ asum (zipWith tierRow [0 :: Int ..] (reqTier req))
    , table "sim" (simRow n) 2 (reqSim req)
    , asum (zipWith (posRow unitTier n) [0 :: Int ..] (reqPos req))
        <|> ascendingBy "pos" 1 (reqPos req)
    , table "churn" (nodeRow n 5) 1 (reqChurn req)
    , table "cochange" (pairRow n 3) 2 (reqCochange req)
    , table "continuous" contRow 2 (reqCont req)
    , asum (zipWith discEntry [0 :: Int ..] (reqDisc req))
    , ascendingBy "discrete" 1 (map pure (reqDisc req))
    , either Just (const Nothing) parsed
    , weightsOffence (reqWeights req)
    , ceilingsOffence (reqCeilings req)
    , thresholdsOffence (reqThresholds req)
    , toleranceOffence (reqTolerance req)
    , floorOffence (reqThresholds req) (reqFloor req)
    , dedupOffence (reqDedup req)
    ]
 where
  n = toInteger (length (reqTier req))
  -- built lazily, consulted only after the tier element of the asum
  -- has proven density (row i names node i) — the review HIGH-2
  -- repayment: the old per-row list scan re-derived that index and
  -- cost F²/2 across a legal request
  unitTier =
    IS.fromList
      [i | (i, [_, code]) <- zip [0 :: Int ..] (reqTier req), code /= 0]

tierRow :: Int -> [Integer] -> Maybe String
tierRow i row = case row of
  [u, code]
    | u /= toInteger i -> Just ("tier " <> show i <> ": index mismatch")
    | code < 0 || code > 1 -> Just ("tier " <> show i <> ": unknown tier code")
    | otherwise -> Nothing
  _ -> Just ("tier " <> show i <> ": malformed row (need [u,tier])")

pairRow :: Integer -> Int -> String -> Int -> [Integer] -> Maybe String
pairRow n arity name i row = case row of
  (u : v : _)
    | length row /= arity -> Just (label name i <> "malformed row")
    | any (< 0) row -> Just (label name i <> "negative field")
    | u >= n || v >= n -> Just (label name i <> "endpoint out of range")
    | u >= v -> Just (label name i <> "pair not ascending")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

-- | sim rows carry the ONE enum + ratio the module used to leave
-- unchecked (review MED: Join routed an out-of-enum kind to the
-- clone bar while Score scored it zero, and den = 0 made the
-- cross-multiplication vacuously true — a certain clone from 0/0).
simRow :: Integer -> String -> Int -> [Integer] -> Maybe String
simRow n name i row =
  pairRow n 5 name i row <|> case row of
    [_, _, kind, _, den]
      | kind > 2 -> Just (label name i <> "unknown sim kind")
      | den == 0 -> Just (label name i <> "zero denominator")
      | otherwise -> Nothing
    _ -> Nothing

nodeRow :: Integer -> Int -> String -> Int -> [Integer] -> Maybe String
nodeRow n arity name i row = case row of
  (u : _)
    | length row /= arity -> Just (label name i <> "malformed row")
    | any (< 0) row -> Just (label name i <> "negative field")
    | u >= n -> Just (label name i <> "node out of range")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

posRow :: IS.IntSet -> Integer -> Int -> [Integer] -> Maybe String
posRow unitTier n i row = case row of
  [u, indeg, outdeg, sccId, sccSize, reachIn]
    | any (< 0) [u, indeg, outdeg, sccId, sccSize, reachIn] ->
        Just (label "pos" i <> "negative field")
    | u >= n -> Just (label "pos" i <> "node out of range")
    -- range-checked above and the tier table is dense by the time
    -- this runs, so set membership IS "tier code /= 0"
    | IS.member (fromInteger u) unitTier -> Just (label "pos" i <> "unit-tier node")
    | otherwise -> Nothing
  _ -> Just (label "pos" i <> "malformed row (need 6 fields)")

-- | continuous entities are FINGERPRINTS (u64), not tier indexes:
-- the ratchet joins current-vs-baseline on (u, code) across runs,
-- and a tier index shifts whenever a file lands — so u here is
-- range-checked against u64, never against the node universe.
contRow :: String -> Int -> [Integer] -> Maybe String
contRow name i row = case row of
  [u, code, v]
    | any (< 0) [u, code, v] -> Just (label name i <> "negative field")
    | u >= 18446744073709551616 -> Just (label name i <> "outside u64")
    | code > 6 -> Just (label name i <> "unknown metric code")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

discEntry :: Int -> Integer -> Maybe String
discEntry i x
  | x < 0 || x >= 18446744073709551616 = Just (label "discrete" i <> "outside u64")
  | otherwise = Nothing

-- | The one reader of ce-baseline.json bytes: null = establish;
-- otherwise {continuous, discrete} with the same row discipline as
-- the live tables. Entities are NOT range-checked against the tier
-- universe — a baseline may outlive the files it measured.
parseBaseline :: Value -> Either String (Maybe Baseline)
parseBaseline Null = Right Nothing
parseBaseline v = case AT.parse bl v of
  AT.Error e -> Left ("baseline: " <> e)
  AT.Success (cont, disc) ->
    case asum
      [ asum (zipWith contShape [0 :: Int ..] cont)
      , ascendingBy "baseline.continuous" 2 cont
      , ascendingBy "baseline.discrete" 1 (map pure disc)
      ] of
      Just why -> Left why
      Nothing -> Right (Just (Baseline cont disc))
 where
  bl = withObject "baseline" $ \o -> (,) <$> o .: "continuous" <*> o .: "discrete"
  contShape i row = case row of
    [_, code, _]
      | any (< 0) row -> Just (label "baseline.continuous" i <> "negative field")
      | code > 6 -> Just (label "baseline.continuous" i <> "unknown metric code")
      | otherwise -> Nothing
    _ -> Just (label "baseline.continuous" i <> "malformed row")

-- knob-table offences live in CE.Verdict.Table (the shared row
-- grammar), split from this module at the 300-line law.

-- | The floor is bounded by the EFFECTIVE score scale (review C7:
-- the 1000 literal survived scoreScale becoming a knob in the same
-- batch — a floor above the scale can never pass, and one above
-- every reachable score would fail forever undiagnosed; both refuse
-- by name now). The thresholds table is validated before this row
-- of the asum, so the [6, v] scan reads checked rows only.
floorOffence :: [[Integer]] -> Maybe Integer -> Maybe String
floorOffence _ Nothing = Nothing
floorOffence thrs (Just f)
  | f < 0 || f > scale = Just "floor: outside the effective score scale"
  | otherwise = Nothing
 where
  scale = last (scoreScale : [v | [6, v] <- thrs])

-- | ADR-008 P2: the dedup budget pair is two in-range counts or
-- nothing — a malformed pair must never read as "under budget".
dedupOffence :: Maybe [Integer] -> Maybe String
dedupOffence Nothing = Nothing
dedupOffence (Just row) = case row of
  [blocks, budget]
    | blocks < 0 || budget < 0 -> Just "dedup: negative field"
    | blocks >= u64 || budget >= u64 -> Just "dedup: outside u64"
    | otherwise -> Nothing
  _ -> Just "dedup: malformed pair (need [blocks,budget])"
 where
  u64 = 18446744073709551616
