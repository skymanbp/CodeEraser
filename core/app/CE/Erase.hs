-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | erase.request handler (M9 batch 3): the ninth judgment family —
-- the deterministic two-phase eraser's PREDICATE. Rust measures the
-- candidate facts (graph death, byte equality, word counts, unit
-- coverage, per-language unresolved-site counts) and sends dense
-- integer rows; this family answers which rows are safe to erase and
-- names WHY every other row is advisory. Since 7.2.0 (plan v2.30 step
-- 7b) the request may also carry the rows' TARGETS — one
-- [pathId, start, end] per row — and the reply then says which row
-- stands for each target (`kept`, CE.Erase.Cost.keptRows); a request
-- without the table keeps its bytes. Paths never cross the wire
-- (§5.9.2) — row index is identity and Rust re-labels on return.
-- The predicate is deliberately knobless: safety is not tunable.
module CE.Erase (respond) where

import CE.Erase.Cost (classOf, eraseRowCap, judgeRow, keptRows)
import CE.Wire (RowsReq (..), knoblessRows, rowCheck)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

-- | The shared knobless cascade with this family's bindings
-- (CE.Wire.knoblessRows): per-class row shapes refused by name,
-- any knob refused outright — v1 declares no codes, and an
-- unknown knob silently dropped would present a tuned predicate
-- as effective (the pinned-echo lesson, structure/wire.rs) — and
-- the target table's own contract past the rows.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = knoblessRows "erase" eraseRowCap rowShape targetsBattery (degraded proto) (judged proto)

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [cls, w, x, y, z]
    | cls == 0 -> Just (label <> "retired class 0 (superseded by 3 at 2.32.0, retired 4.0.0)")
    | cls < 0 || cls > 3 -> Just (label <> "unknown class")
    | any (< 0) [w, x, y, z] -> Just (label <> "negative fact")
    | cls == 3 && (w < 1 || w > 4) -> Just (label <> "dead verdict outside 1..4")
    | cls == 1 && z > 1 -> Just (label <> "bytesEqual not a boolean")
    | cls == 2 && any (> 1) [w, x] -> Just (label <> "coverage/equality not booleans")
    | cls == 2 && y > 4 -> Just (label <> "dead verdict outside 0..4")
    | cls == 3 && x > 2 -> Just (label <> "confidence outside 0..2")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [class,w,x,y,z])")
 where
  label = "row " <> show i <> ": "

-- | The target table's contract (7.2.0): aligned 1:1 with the fact
-- rows; each [pathId, start, end] a dense non-negative path id
-- (names never cross) with 1-based inclusive lines, or 0/0 for the
-- whole file — the one shape each class can name (a verbatim segment
-- is a span, a twin or a dead file is a whole file); and the table
-- in key order, so the closure's earliest-row tie is a stated fact.
-- Runs after the fact rows, so every class read here is 1..3.
targetsBattery :: RowsReq -> Maybe String
targetsBattery req = case targetsOf req of
  Nothing -> Nothing
  Just ts ->
    asum
      [ counts ts
      , asum (zipWith3 targetShape [0 :: Int ..] ts (rowsOf req))
      , asum (zipWith keyOrder [1 :: Int ..] (zip ts (drop 1 ts)))
      ]
 where
  counts ts
    | length ts /= length (rowsOf req) =
        Just ("targets: " <> show (length ts) <> " rows for " <> show (length (rowsOf req)) <> " fact rows")
    | otherwise = Nothing
  keyOrder i (prev, cur)
    | take 3 prev > take 3 cur = Just ("target " <> show i <> ": out of key order")
    | otherwise = Nothing

targetShape :: Int -> [Integer] -> [Integer] -> Maybe String
targetShape i t row = rowCheck "target" "malformed row (need [pathId,start,end])" 3 checks i t
 where
  cls = classOf row
  checks target = case target of
    [p, s, e]
      | any (< 0) [p, s, e] -> Just "negative field"
      | (s == 0) /= (e == 0) -> Just "half-open span"
      | s > e -> Just "span end before start"
      | cls == 1 && s == 0 -> Just "verbatim_doc row names a whole file"
      | cls /= 1 && s > 0 -> Just "whole-file class names a span"
    _ -> Nothing

judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req verdicts False
 where
  verdicts = map judgeRow (rowsOf req)

-- | Over-cap: a complete degraded reply that FAILS with an empty
-- verdict table — a plan the core refused to judge licenses nothing.
degraded :: String -> RowsReq -> B8.ByteString
degraded proto req = reply proto req [] True

-- | The erase.result object: one [eraseable, reason] pair per row in
-- request order; fail mirrors degraded (the plan itself never gates —
-- the self-repo zero-row gate is the CLI's --check, judged against
-- THIS table, not a core fail bit). `kept` rides exactly when the
-- target table rode and the rows were judged (the cocBumped
-- precedent): one 0/1 per row, the closure's answer.
reply :: String -> RowsReq -> [(Bool, Integer)] -> Bool -> B8.ByteString
reply proto req verdicts isDegraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("erase.result" :: String)
    , "id" .= rowsId req
    , "rows" .= [[if e then 1 else 0 :: Integer, r] | (e, r) <- verdicts]
    , "counts"
        .= object
          [ "rows" .= length (rowsOf req)
          , "eraseable" .= length (filter fst verdicts)
          , "advisory" .= length (filter (not . fst) verdicts)
          ]
    , "fail" .= isDegraded
    , "degraded" .= isDegraded
    ]
      <> ["reason" .= ("erase_too_large" :: String) | isDegraded]
      <> [ "kept" .= [if k then 1 else 0 :: Integer | k <- keptRows ts (rowsOf req) verdicts]
         | not isDegraded
         , Just ts <- [targetsOf req]
         ]
