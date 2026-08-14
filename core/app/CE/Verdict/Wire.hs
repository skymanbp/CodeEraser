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

-- | First boundary-contract offender, if any. The row checkers are
-- top-level functions taking the universe size n (the M5-close warn
-- repayment: a 64-line where block was the E01 offender, and the
-- checkers never needed the closure — only n and the tier table).
violation :: VerdictReq -> Maybe String
violation req =
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
    , either Just (const Nothing) (parseBaseline (reqBaseline req))
    , weightsOffence (reqWeights req)
    , floorOffence (reqFloor req)
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

table :: String -> (String -> Int -> [Integer] -> Maybe String) -> Int -> [[Integer]] -> Maybe String
table name rowCheck idWidth rows =
  asum (zipWith (rowCheck name) [0 :: Int ..] rows) <|> ascendingBy name idWidth rows

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
