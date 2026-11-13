-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Every surviving verdict knob is a live lever (v2.35, 2D C1): the
-- density battery's "each knob moves the score" legs retired with that
-- law, and a mutation sweep that made the judgment ignore one knob at a
-- time (the echo still reporting it) found six knobs no leg turned red
-- for — ceiling codes 1 (cocCeil) and 2 (sizeHard), threshold codes
-- 1, 2 (the rewrite ratio), 3 (cochangeFloor) and 5 (defaultWeight).
-- Each leg here sends one request twice, differing only in that knob's
-- row, through the real respond, and asks the judgment to move: the
-- score for the five the extent law reads, the join candidates for the
-- cochange floor.
--
-- The tree: three files. File 0 is 120 lines (under every soft line, so
-- no size term) with one function on lines 10..109 at cognitive 30 —
-- 100 · (1 − C/30) lines over the ceiling C; files 1 and 2 are prose
-- (no size term), file 1 rewrite-heavy (30 rewritten, 10 appended:
-- 100 · (1 − t · 40/30) lines at rewrite ratio t), the pair (1, 2) a
-- clone pair that co-changed three times. One weight row (churn at 3)
-- makes the layered extent read the default weight.
module VerdictLeverProps (battery) where

import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import VerdictTables (respondV)
import WireHarness (replyObjWith, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("ceilings 1 (cocCeil): the function's lines over the ceiling move the score", moves "score" "ceilings" [[1, 20]] tree)
    , ("ceilings 2 (sizeHard): the derived soft line's fence moves the score at establish", moves "score" "ceilings" [[2, 450]] established)
    , ("thresholds 1 (rewriteNum): the rewrite-heavy file's excess moves the score", moves "score" "thresholds" [[1, 60]] tree)
    , ("thresholds 2 (rewriteDen): the rewrite-heavy file's excess moves the score", moves "score" "thresholds" [[2, 120]] tree)
    , ("thresholds 3 (cochangeFloor): the co-change leg moves the join candidates", moves "candidates" "thresholds" [[3, 9]] tree)
    , ("thresholds 5 (defaultWeight): the unlisted axes' weight moves the layered score", moves "score" "thresholds" [[5, 2]] tree)
    ]

-- | The reply's `field` without the knob row and with it differ, and
-- both replies are judged (a refusal would also "differ").
moves :: Key -> String -> [[Integer]] -> Value -> Bool
moves field table row req = case (judged req, judged (setKey table (toJSON row) req)) of
  (Just a, Just b) -> a /= b
  _ -> False
 where
  judged r = do
    o <- replyObjWith respondV r
    case KM.lookup "degraded" o of
      Just (Bool True) -> Nothing
      _ -> KM.lookup field o

tree :: Value
tree =
  object
    [ "proto" .= ("10.0.0" :: String)
    , "type" .= ("verdict.request" :: String)
    , "id" .= (1 :: Int)
    , "sim" .= [[1, 2, 0, 90, 100 :: Integer]]
    , "pos" .= [[u, 1, u, 1, 1] | u <- [0 .. 2 :: Integer]]
    , "tier" .= [[u, 0] | u <- [0 .. 2 :: Integer]]
    , "churn" .= [[1, 30, 10 :: Integer]]
    , "cochange" .= [[1, 2, 3 :: Integer]]
    , "continuous" .= [[0, 0, 120], [0, 1, 30 :: Integer]]
    , "discrete" .= ([] :: [Integer])
    , "baseline" .= Null
    , "weights" .= [[5, 3 :: Integer]]
    , "floor" .= Null
    , "contSpans" .= [[0, 0, 0], [0, 10, 109 :: Integer]]
    , "dupSpans" .= ([] :: [[Integer]])
    , "refLines" .= ([] :: [[Integer]])
    , "proseLines" .= [[1, 100], [2, 100 :: Integer]]
    ]

-- | An establishing run whose judged LOC derive S = 600 before the
-- fences: the default fence reads softMax 500, a hard line of 450 reads
-- 450, and the 600-line file's size term is 100 or 150 lines.
established :: Value
established =
  setKey "judgedLoc" (toJSON [600, 600, 600 :: Integer]) $
    setKey "continuous" (toJSON [[0, 0, 600 :: Integer]]) $
      setKey "contSpans" (toJSON [[0, 0, 0 :: Integer]]) tree
