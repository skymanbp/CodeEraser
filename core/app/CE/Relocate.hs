-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | relocate.request handler (v2.35 guard lane, GD4): a STAGE of the
-- duplicate-write class, not a family of its own. The PreToolUse
-- probe refuses a write whose content matches a verified run of the
-- tree; a write that MOVES code (its source removed first) still
-- matched the moved function's other surviving twins, though the
-- tree's count of that run did not rise. Rust sends one
-- [occHead, occNow] row per match; this stage answers which rows are
-- relocations and whether every one is. No knob: a knob that loosened
-- the rule would be a licence (CE.Wire knoblessRows' stance).
module CE.Relocate (respond) where

import CE.Relocate.Cost (relocateRowCap, relocated)
import qualified CE.Wire as W
import qualified CE.Wire.Families as F
import Data.Aeson ((.=))
import qualified Data.ByteString.Char8 as B8

respond :: W.Respond
respond proto = W.knoblessRows F.relocate relocateRowCap rowShape (const Nothing) (reply proto True) (reply proto False)
 where
  rowShape = W.rowCheck "row" "malformed row (need [occHead,occNow])" 2 rowChecks

-- | The write's own copy is always in the count (it is the target's
-- applied text), so `occNow = 0` names a measuring side that lost it —
-- a fragment tokenised apart from the applied file — and is refused
-- rather than credited.
rowChecks :: [Integer] -> Maybe String
rowChecks row = case row of
  [h, n]
    | h < 0 || n < 0 -> Just "negative count"
    | n == 0 -> Just "occNow 0: the write's own copy is missing from the count"
  _ -> Nothing

-- | The relocate.result object: the relocation row indices in request
-- order and whether every row is one (an empty or refused request is
-- never `all`). Over the cap: degraded, nothing credited.
reply :: String -> Bool -> W.RowsReq -> B8.ByteString
reply proto isDegraded req =
  W.resultLine
    proto
    F.relocate
    (W.rowsId req)
    ["relocated" .= moved, "all" .= (not (null rows) && length moved == length rows)]
    ["rows" .= length (W.rowsOf req)]
    (W.tooLarge F.relocate isDegraded)
 where
  rows = if isDegraded then [] else W.rowsOf req
  moved = [i | (i, r) <- zip [0 :: Integer ..] rows, relocated r]
