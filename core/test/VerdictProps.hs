-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family's score/ratchet battery (design §7.4, M5-3i).
-- The two F16 preconditions come FIRST and void the battery when
-- absent: every axis penalty nonzero on the fixture AND penalties
-- pairwise distinct AND the fixture weights pairwise distinct —
-- equal weights over equal penalties make the weighted mean immune
-- to exactly the perturbations this battery exists to feel. The
-- ablation, idempotence and refusal checks drive the REAL
-- CE.Verdict.respond, not a reimplementation.
module VerdictProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Ratchet
  ( Baseline (..)
  , RatchetKnobs (..)
  , Ratcheted (..)
  , ratchet
  , ratchetBound
  )
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), penalties, score, scoreBound)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as BL
import Data.List (isInfixOf, nub)
import qualified Data.Set as S

battery :: IO Bool
battery = do
  a <- check "precondition: every axis penalty nonzero and pairwise distinct" preNonzero
  b <- check "precondition: battery weights pairwise distinct" preWeights
  c <- check "each weight +1 moves the (score, violations) tuple" weightKnobs
  d <- check "each axis threshold knob moves the score" axisKnobs
  e <- check "ablating any join leg moves the candidate census" ablation
  f <- check "ratchet idempotence: newBaseline fed back judges nothing" idempotent
  g <- check "tolerance-over sets shrink as tolAbs grows (inclusion)" overMono
  h <- check "refusals name the offender with code and message" refusals
  pure (and [a, b, c, d, e, f, g, h])

check :: String -> Bool -> IO Bool
check name ok = do
  putStrLn ((if ok then "ok   " else "FAIL ") <> name)
  pure ok

-- | The score fixture: per axis 0..6 the penalties are
-- 1,2,3,4,5,6,17, each axis with one row EXACTLY on its threshold so
-- a +1 knob probe has a boundary to flip. fPos is written
-- node-ascending (16 = the near-dead row only the deadIndegCeil
-- probe counts). The cycle axis carries 17, not 7: with 1..7 the
-- weighted MEAN landed exactly on penalty 5, and a +1 weight at the
-- mean moves nothing — pairwise-distinct penalties alone do not
-- prevent one axis SITTING on the mean, so the fixture keeps the
-- mean off every penalty (all seven bumps verified to move).
facts :: Facts
facts =
  Facts
    { fSim =
        [ [3, 4, 1, 85, 100] -- clone boundary
        , [5, 6, 1, 90, 100]
        , [7, 8, 0, 100, 100]
        , [9, 10, 2, 80, 100] -- dup boundary
        , [11, 12, 2, 85, 100]
        , [13, 14, 2, 90, 100]
        , [15, 16, 2, 100, 100]
        ]
    , fPos =
        [[16, 1, 0, 16, 1, 0]]
          <> [[u, 0, 0, u, 1, 0] | u <- [17 .. 21]]
          <> [[u, 1, 1, 50, 2, 1] | u <- [22 .. 38]]
    , fChurn =
        -- five clearly rewrite-heavy, one exactly at 50/100, one under
        [[u, 30, 10, 0, 0] | u <- [39 .. 43]]
          <> [[44, 20, 20, 0, 0], [45, 1, 50, 0, 0]]
    , fCont = [[0, 0, 400], [1, 1, 20], [2, 1, 30]]
    }

battWeights :: [[Integer]]
battWeights = [[c, c + 1] | c <- [0 .. 6]]

pens :: [(Integer, Integer)]
pens = penalties scoreBound facts

preNonzero :: Bool
preNonzero =
  map fst pens == [0 .. 6]
    && all ((> 0) . snd) pens
    && length (nub (map snd pens)) == 7

preWeights :: Bool
preWeights = length (nub (map (!! 1) battWeights)) == 7

baseTuple :: (Integer, Integer)
baseTuple = score scoreBound battWeights pens

weightKnobs :: Bool
weightKnobs = and [score scoreBound (bump c) pens /= baseTuple | c <- [0 .. 6]]
 where
  bump c = [[c', if c' == c then w + 1 else w] | [c', w] <- battWeights]

axisKnobs :: Bool
axisKnobs =
  all
    (\k -> score k battWeights (penalties k facts) /= baseTuple)
    [ scoreBound {sSizeCeil = sSizeCeil scoreBound + 100}
    , scoreBound {sCocCeil = sCocCeil scoreBound + 5}
    , scoreBound {sCloneNum = sCloneNum scoreBound + 1}
    , scoreBound {sDupNum = sDupNum scoreBound + 1}
    , scoreBound {sDeadIndegCeil = sDeadIndegCeil scoreBound + 1}
    , scoreBound {sRewriteNum = sRewriteNum scoreBound + 1}
    , scoreBound {sCycleFloor = sCycleFloor scoreBound + 1}
    ]

-- | A wire request whose candidates cover merge, delete and hotspot;
-- the ablations empty one leg each and the census must move.
wireReq :: [[Integer]] -> [[Integer]] -> [[Integer]] -> [[Integer]] -> Value
wireReq sim pos churn coch =
  object
    [ "proto" .= ("2.2.0" :: String)
    , "type" .= ("verdict.request" :: String)
    , "id" .= (1 :: Int)
    , "sim" .= sim
    , "pos" .= pos
    , "tier" .= [[u, 0] | u <- [0 .. 5 :: Int]]
    , "churn" .= churn
    , "cochange" .= coch
    , "continuous" .= ([] :: [Value])
    , "discrete" .= ([] :: [Integer])
    , "baseline" .= Null
    , "weights" .= ([] :: [Value])
    , "floor" .= Null
    ]

wSim, wPos, wChurn, wCoch :: [[Integer]]
wSim = [[0, 1, 1, 90, 100], [2, 3, 1, 90, 100], [4, 5, 2, 85, 100]]
wPos =
  [ [0, 1, 0, 0, 1, 1]
  , [1, 2, 0, 1, 1, 1]
  , [2, 0, 0, 2, 1, 0]
  , [3, 1, 0, 3, 1, 1]
  , [4, 1, 0, 4, 1, 1]
  , [5, 1, 0, 4, 1, 1]
  ]
wChurn = [[4, 30, 10, 0, 0], [5, 30, 10, 0, 0]]
wCoch = [[4, 5, 3]]

candidateCensus :: Value -> Maybe [Int]
candidateCensus req = do
  bytes <- rightBytes (respond "0.0.1" (BL.toStrict (encode req)))
  Object o <- decodeStrict bytes
  Array cands <- KM.lookup "candidates" o
  rows <- mapM verdictOfRow (foldr (:) [] cands)
  pure [length [() | v <- rows, v == c] | c <- [0 .. 3]]
 where
  rightBytes (Right b) = Just b
  rightBytes (Left _) = Nothing
  verdictOfRow v = case fromJSON v :: Result [Integer] of
    Success (_ : _ : code : _) -> Just code
    _ -> Nothing

ablation :: Bool
ablation = case candidateCensus (wireReq wSim wPos wChurn wCoch) of
  Nothing -> False
  Just full ->
    drop 1 full == [1, 1, 1] -- one merge, one delete, one hotspot
      && all
        (moved full)
        [ wireReq [[u, v, k, 0, 100] | [u, v, k, _, _] <- wSim] wPos wChurn wCoch
        , wireReq wSim [] wChurn wCoch
        , wireReq wSim wPos [] []
        ]
 where
  moved full v = case candidateCensus v of
    Just c -> c /= full
    Nothing -> False

-- | Round trip: judge with no baseline, feed the returned
-- newBaseline into the SAME facts — the ratchet must find nothing.
idempotent :: Bool
idempotent = case reply (req Null) >>= KM.lookup "newBaseline" of
  Nothing -> False
  Just nb -> case reply (req nb) of
    Just o2 ->
      field o2 "added" == Just (Array mempty)
        && field o2 "over" == Just (Array mempty)
        && field o2 "toleranceDrawn" == Just (Array mempty)
        && field o2 "fail" == Just (Bool False)
    Nothing -> False
 where
  req base =
    object
      [ "proto" .= ("2.2.0" :: String)
      , "type" .= ("verdict.request" :: String)
      , "id" .= (2 :: Int)
      , "sim" .= ([] :: [Value])
      , "pos" .= ([] :: [Value])
      , "tier" .= [[u, 0] | u <- [0 .. 2 :: Int]]
      , "churn" .= ([] :: [Value])
      , "cochange" .= ([] :: [Value])
      , "continuous" .= [[0, 0, 310 :: Integer], [1, 1, 20], [2, 0, 50]]
      , "discrete" .= [3, 7 :: Integer]
      , "baseline" .= base
      , "weights" .= ([] :: [Value])
      , "floor" .= Null
      ]
  reply r = do
    bytes <- rb (respond "0.0.1" (BL.toStrict (encode r)))
    Object o <- decodeStrict bytes
    pure o
  rb (Right b) = Just b
  rb (Left _) = Nothing
  field o sub = do
    Object inner <- KM.lookup "ratchet" o
    KM.lookup (Key.fromString sub) inner

-- | Monotonicity as SET INCLUSION (never count comparison): growing
-- the absolute tolerance leg can only shrink the over set, checked
-- on (entity, metric) identities across seeded fact tables.
overMono :: Bool
overMono = all one [1 .. 60 :: Integer]
 where
  one i =
    let cont = [[u, 0, 100 + ((i * (u + 1) * 7) `mod` 40)] | u <- [0 .. 9]]
        base = Baseline [[u, 0, 100] | u <- [0 .. 9]] []
        ids k = S.fromList [(u, c) | [u, c, _, _] <- rOver (ratchet k (Just base) cont [])]
        wide = ratchetBound {rTolAbs = rTolAbs ratchetBound + 15}
     in ids wide `S.isSubsetOf` ids ratchetBound

refusals :: Bool
refusals =
  and
    [ refused (setKey "discrete" (toJSON [9, 7 :: Integer]) base) "not strictly ascending"
    , refused (setKey "tier" (toJSON [[1, 0 :: Integer]]) base) "index mismatch"
    , refused posReq "unit-tier node"
    , refused (setKey "weights" (toJSON [[c, 0] | c <- [0 .. 6 :: Integer]]) base) "every axis zeroed"
    , -- the sim domain pair (M5-close review): an out-of-enum kind
      -- and a zero denominator each refuse by name
      refused (simReq [[0, 1, 3, 50, 100]]) "unknown sim kind"
    , refused (simReq [[0, 1, 0, 50, 0]]) "zero denominator"
    ]
 where
  simReq rows =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 0]] :: [[Integer]]))
      (setKey "sim" (toJSON (rows :: [[Integer]])) base)
  base = wireReq [] [] [] []
  posReq =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 1], [2, 0], [3, 0], [4, 0], [5, 0]] :: [[Integer]]))
      (setKey "pos" (toJSON [[1, 0, 0, 0, 1, 0 :: Integer]]) base)
  setKey k v (Object o) = Object (KM.insert (Key.fromString k) v o)
  setKey _ _ v = v
  refused r want = case respond "0.0.1" (BL.toStrict (encode r)) of
    Left (_, code, msg) -> code == "contract" && want `isInfixOf` msg
    Right _ -> False
