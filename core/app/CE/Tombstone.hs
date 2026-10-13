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
module CE.Tombstone (kindNames, respond) where

import CE.Tombstone.Cost (budgetCode, isSite, kindProse, overBudget, tombstoneRowCap)
import CE.Wire (Respond, RowsReq (..), knobTable, knobbedRows, resultLine, rowCheck, tooLarge)
import qualified CE.Wire.Families as F
import Data.Aeson ((.=))
import qualified Data.ByteString.Char8 as B8

-- | The row kinds by code, as a site names its kind (`file:line kind`;
-- the measuring side's `Kind::name`, read by the guard and audit
-- sentences, plan v2.32 step 5); the codes are CE.Tombstone.Cost's.
kindNames :: [String]
kindNames = ["bracketed", "bare", "prose"]

-- | The declared budget, if any (later rows win, as applyRows would).
budgetOf :: RowsReq -> Maybe Integer
budgetOf req = case [v | [c, v] <- knobsOf req, c == budgetCode] of
  [] -> Nothing
  vs -> Just (last vs)

-- | The shared knobbed-table cascade with this family's bindings
-- (CE.Wire knobbedRows): the rows-and-knobs cap, the row contract
-- through rowCheck (width 3, then the value checks), the knob
-- contract — one code today — and the two replies.
respond :: Respond
respond = knobbedRows F.tombstone tombstoneRowCap rowShape knobsOffence degraded judged
 where
  rowShape = rowCheck "row" "malformed row (need [kind,marks,erasedNames])" 3 rowChecks

rowChecks :: [Integer] -> Maybe String
rowChecks row = case row of
  [kind, marks, names]
    | kind < 0 || kind > kindProse -> Just "kind outside 0..2"
    | marks < 0 || names < 0 -> Just "negative count"
  _ -> Nothing

-- | The knob table (CE.Wire.knobTable): one code, the budget.
knobsOffence :: [[Integer]] -> Maybe String
knobsOffence = knobTable "knob" "malformed knob (need [code,value])" budgetCode "unknown knob code" judge
 where
  judge _ v
    | v < 0 = Just "negative knob value"
    | otherwise = Nothing

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
reply proto req sites (labelled, prose) over isDegraded =
  resultLine
    proto
    F.tombstone
    (rowsId req)
    ["sites" .= sites, "over" .= over, "knobs" .= [[budgetCode, b] | Just b <- [budgetOf req]]]
    ["rows" .= length (rowsOf req), "label" .= labelled, "prose" .= prose]
    (tooLarge F.tombstone isDegraded)
