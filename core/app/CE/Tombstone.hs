-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | tombstone.request handler (plan v2.27 step 4): the eleventh
-- judgment family — the tombstone-residue VERDICT behind the
-- PreToolUse guard, the Stop audit and `ce precommit`. Rust measures
-- every candidate surface a changeset wrote and sends one
-- [kind, marks, erasedNames] row per surface (a row with both counts
-- zero is not sent: absence is zero); this family answers which rows
-- are sites and whether the changeset is over its declared budget.
-- Names and paths never cross the wire (§5.9.2) — row index is
-- identity and Rust re-labels on return. One knob, code 0 = the
-- budget; absent = feed-only, the condition is never evaluated.
module CE.Tombstone (respond) where

import CE.Tombstone.Cost (budgetCode, isSite, kindProse, overBudget, tombstoneRowCap)
import CE.Wire (RowsReq (..), knobbedRows, rowCheck)
import Data.Aeson (Value, encode, object, (.=))
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | The declared budget, if any (later rows win, as applyRows would).
budgetOf :: RowsReq -> Maybe Integer
budgetOf req = case [v | [c, v] <- knobsOf req, c == budgetCode] of
  [] -> Nothing
  vs -> Just (last vs)

-- | The shared knobbed-table cascade with this family's bindings
-- (CE.Wire knobbedRows): the rows-and-knobs cap, the row contract
-- through rowCheck (width 3, then the value checks), the knob
-- contract — one code today — and the two replies.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond = knobbedRows "tombstone" tombstoneRowCap rowShape knobShape degraded judged
 where
  rowShape = rowCheck "row" "malformed row (need [kind,marks,erasedNames])" 3 rowChecks

rowChecks :: [Integer] -> Maybe String
rowChecks row = case row of
  [kind, marks, names]
    | kind < 0 || kind > kindProse -> Just "kind outside 0..2"
    | marks < 0 || names < 0 -> Just "negative count"
  _ -> Nothing

knobShape :: Int -> [Integer] -> Maybe String
knobShape i row = case row of
  [code, v]
    | code /= budgetCode -> Just (label <> "unknown knob code")
    | v < 0 -> Just (label <> "negative knob value")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed knob (need [code,value])")
 where
  label = "knob " <> show i <> ": "

judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req (map fst sites) (label, prose) over False
 where
  sites = [(i, row) | (i, row) <- zip [0 :: Integer ..] (rowsOf req), isSite row]
  prose = length [() | (_, k : _) <- sites, k == kindProse]
  label = length sites - prose
  over = overBudget (budgetOf req) (length sites)

-- | Over-cap: a complete degraded reply with an empty site table and
-- the condition unevaluated — a changeset the core refused to judge
-- is neither convicted nor cleared; the CLI names the degradation.
degraded :: String -> RowsReq -> B8.ByteString
degraded proto req = reply proto req [] (0, 0) False True

-- | The tombstone.result object: the site row indices in request
-- order, their label / prose split, the budget condition, the
-- effective knob table echoed (empty = no budget declared).
reply :: String -> RowsReq -> [Integer] -> (Int, Int) -> Bool -> Bool -> B8.ByteString
reply proto req sites (label, prose) over isDegraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("tombstone.result" :: String)
    , "id" .= rowsId req
    , "sites" .= sites
    , "counts"
        .= object
          [ "rows" .= length (rowsOf req)
          , "label" .= label
          , "prose" .= prose
          ]
    , "over" .= over
    , "knobs" .= [[budgetCode, b] | Just b <- [budgetOf req]]
    , "degraded" .= isDegraded
    ]
      <> ["reason" .= ("tombstone_too_large" :: String) | isDegraded]
