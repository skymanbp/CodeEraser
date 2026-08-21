-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | erase.request handler (M9 batch 3): the ninth judgment family —
-- the deterministic two-phase eraser's PREDICATE. Rust measures the
-- candidate facts (graph death, byte equality, word counts, unit
-- coverage, per-language unresolved-site counts) and sends dense
-- integer rows; this family answers which rows are safe to erase and
-- names WHY every other row is advisory. Paths never cross the wire
-- (§5.9.2) — row index is identity and Rust re-labels on return.
-- The predicate is deliberately knobless: safety is not tunable.
module CE.Erase (respond) where

import CE.Erase.Cost (eraseRowCap, judgeRow)
import CE.Wire (RowsReq (..), rowsFamily)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

-- | The shared cascade with this family's bindings (CE.Wire); the
-- request is the shared RowsReq — this family reads rows + knobs.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  rowsFamily
    "erase"
    (\req -> toInteger (length (rowsOf req)) > eraseRowCap)
    violation
    (degraded proto)
    (judged proto)

-- | First boundary-contract offender in request order. Every row is
-- [class, w, x, y, z]; class field ranges are checked per class so a
-- malformed fact refuses by name instead of judging as "unsafe". Any
-- knob row is refused outright — v1 declares no codes, and an
-- unknown knob silently dropped would present a tuned predicate as
-- effective (the pinned-echo lesson, structure/wire.rs).
violation :: RowsReq -> Maybe String
violation req =
  asum
    [ asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
    , asum (zipWith noKnob [0 :: Int ..] (knobsOf req))
    ]
 where
  noKnob i _ = Just ("knob " <> show i <> ": erase/1 declares no knob codes")

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [cls, w, x, y, z]
    | cls < 0 || cls > 2 -> Just (label <> "unknown class")
    | any (< 0) [w, x, y, z] -> Just (label <> "negative fact")
    | cls == 0 && (w < 1 || w > 4) -> Just (label <> "dead verdict outside 1..4")
    | cls == 1 && z > 1 -> Just (label <> "bytesEqual not a boolean")
    | cls == 2 && any (> 1) [w, x, y] -> Just (label <> "coverage/equality/death not booleans")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [class,w,x,y,z])")
 where
  label = "row " <> show i <> ": "

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
-- THIS table, not a core fail bit).
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
