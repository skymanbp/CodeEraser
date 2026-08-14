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

import CE.Verdict.Ratchet (Baseline (..))
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.Aeson.Types as AT
import Data.Foldable (asum)

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

-- | First boundary-contract offender, if any.
violation :: VerdictReq -> Maybe String
violation req =
  asum
    [ asum (zipWith tierRow [0 :: Int ..] (reqTier req))
    , table "sim" (pairRow 5) 2 (reqSim req)
    , asum (zipWith posRow [0 :: Int ..] (reqPos req)) <|> ascendingBy "pos" 1 (reqPos req)
    , table "churn" (nodeRow 5) 1 (reqChurn req)
    , table "cochange" (pairRow 3) 2 (reqCochange req)
    , table "continuous" contRow 2 (reqCont req)
    , asum (zipWith discEntry [0 :: Int ..] (reqDisc req))
    , ascendingBy "discrete" 1 (map pure (reqDisc req))
    , either Just (const Nothing) (parseBaseline (reqBaseline req))
    , weightsOffence (reqWeights req)
    , floorOffence (reqFloor req)
    ]
 where
  n = toInteger (length (reqTier req))
  table name rowCheck idWidth rows =
    asum (zipWith (rowCheck name) [0 :: Int ..] rows) <|> ascendingBy name idWidth rows
  tierRow i row = case row of
    [u, code]
      | u /= toInteger i -> Just ("tier " <> show i <> ": index mismatch")
      | code < 0 || code > 1 -> Just ("tier " <> show i <> ": unknown tier code")
      | otherwise -> Nothing
    _ -> Just ("tier " <> show i <> ": malformed row (need [u,tier])")
  pairRow arity name i row = case row of
    (u : v : _)
      | length row /= arity -> Just (label name i <> "malformed row")
      | any (< 0) row -> Just (label name i <> "negative field")
      | u >= n || v >= n -> Just (label name i <> "endpoint out of range")
      | u >= v -> Just (label name i <> "pair not ascending")
      | otherwise -> Nothing
    _ -> Just (label name i <> "malformed row")
  nodeRow arity name i row = case row of
    (u : _)
      | length row /= arity -> Just (label name i <> "malformed row")
      | any (< 0) row -> Just (label name i <> "negative field")
      | u >= n -> Just (label name i <> "node out of range")
      | otherwise -> Nothing
    _ -> Just (label name i <> "malformed row")
  posRow i row = case row of
    [u, indeg, outdeg, sccId, sccSize, reachIn]
      | any (< 0) [u, indeg, outdeg, sccId, sccSize, reachIn] ->
          Just (label "pos" i <> "negative field")
      | u >= n -> Just (label "pos" i <> "node out of range")
      | tierOf u /= Just 0 -> Just (label "pos" i <> "unit-tier node")
      | otherwise -> Nothing
    _ -> Just (label "pos" i <> "malformed row (need 6 fields)")
  -- continuous entities are FINGERPRINTS (u64), not tier indexes:
  -- the ratchet joins current-vs-baseline on (u, code) across runs,
  -- and a tier index shifts whenever a file lands — so u here is
  -- range-checked against u64, never against the node universe
  contRow name i row = case row of
    [u, code, v]
      | any (< 0) [u, code, v] -> Just (label name i <> "negative field")
      | u >= 18446744073709551616 -> Just (label name i <> "outside u64")
      | code > 6 -> Just (label name i <> "unknown metric code")
      | otherwise -> Nothing
    _ -> Just (label name i <> "malformed row")
  discEntry i x
    | x < 0 || x >= 18446744073709551616 = Just (label "discrete" i <> "outside u64")
    | otherwise = Nothing
  tierOf u = case [code | [t, code] <- reqTier req, t == u] of
    (c : _) -> Just c
    [] -> Nothing

label :: String -> Int -> String
label name i = name <> " " <> show i <> ": "

-- | Strictly ascending on the first `k` fields — the row's IDENTITY;
-- duplicate identities are refused by implication, whatever the
-- payload says.
ascendingBy :: String -> Int -> [[Integer]] -> Maybe String
ascendingBy name k rows =
  asum
    [ if take k prev < take k cur
        then Nothing
        else Just (label name i <> "not strictly ascending")
    | (i, (prev, cur)) <- zip [1 ..] (zip rows (drop 1 rows))
    ]

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

-- | Weight rows [axis, numerator], ascending by axis (unique). A
-- request may zero SOME axes (unlisted ones default to 1); zeroing
-- all seven leaves the score with no divisor and is refused.
weightsOffence :: [[Integer]] -> Maybe String
weightsOffence rows =
  asum
    [ asum (zipWith one [0 :: Int ..] rows)
    , ascendingBy "weights" 1 rows
    , if length rows == 7 && all zeroed rows
        then Just "weights: every axis zeroed"
        else Nothing
    ]
 where
  one i row = case row of
    [code, w]
      | code < 0 || w < 0 -> Just (label "weights" i <> "negative field")
      | code > 6 -> Just (label "weights" i <> "unknown axis code")
      | otherwise -> Nothing
    _ -> Just (label "weights" i <> "malformed row (need [axis,weight])")
  zeroed row = case row of
    [_, w] -> w == 0
    _ -> False

floorOffence :: Maybe Integer -> Maybe String
floorOffence Nothing = Nothing
floorOffence (Just f)
  | f < 0 || f > 1000 = Just "floor: outside per-mille range"
  | otherwise = Nothing
