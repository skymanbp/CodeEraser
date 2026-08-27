-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The graph request and its boundary contract — decode plus the
-- machine check that every row is well shaped, every index in range,
-- and every set-shaped table actually a set. Split from CE.Graph at
-- the 300-line dogfood wall when the 4.1.0 symbol table arrived (the
-- deadcode/flags.rs precedent): checking what arrived and judging it
-- are two jobs, and only the first one is allowed to know the
-- request's spelling.
--
-- Every message here is golden-pinned text. A refusal is named by
-- table, row index and reason so a producer learns which row it got
-- wrong, never just that something was wrong.
module CE.Graph.Contract (GraphReq (..), symRows, unresRows, violation) where

import CE.Wire (rowCheck, tableOffence)
import Data.Aeson
import Data.Foldable (asum)

-- | Wire shape (design brief §2): index = node identity, nothing
-- text-shaped crosses. Absent @pos@ = counts only. The optional
-- @unres@ table (2.32.0, H3) is the per-language site ledger
-- [[lang, unresolvedSites, totalSites]] — unlike the old scalar
-- (which stayed an unvalidated honest ledger under the §1 rule),
-- this one IS an input to judgment: each dead row grows a
-- confidence column derived from its node's language.
data GraphReq = GraphReq
  { reqId :: Value
  , reqNodes :: [[Integer]]
  , reqEdges :: [[Integer]]
  , reqPos :: [Integer]
  , reqUnres :: Maybe [[Integer]]
  , -- the optional @symbols@ table (4.1.0) is the EXPORT SURFACE:
    -- deduped [node, visibility] pairs saying which files declare
    -- something and how visibly. It is the producer bit 0 never had.
    reqSymbols :: Maybe [[Integer]]
  }

instance FromJSON GraphReq where
  parseJSON = withObject "GraphReq" $ \o ->
    GraphReq
      <$> o .: "id"
      <*> o .: "nodes"
      <*> o .: "edges"
      <*> o .:? "pos" .!= []
      <*> o .:? "unres"
      <*> o .:? "symbols"

-- | First boundary-contract offender, if any — checked in request
-- order so the message is deterministic. Shape errors surface before
-- ordering errors, so the ascending pass only ever compares
-- well-formed four-tuples (list Ord is lexicographic).
violation :: GraphReq -> Maybe String
violation req =
  asum
    [ asum (zipWith nodeRow [0 :: Int ..] (reqNodes req))
    , tableOffence "edge" id (edgeRow n) es
    , -- ascending pos is also the reply BOUND (M5-close review MED:
      -- pos escaped the declared caps — a repeated-index list made
      -- the reply larger than the request without limit; strictly
      -- ascending indices in [0, n) cannot exceed nodeCap rows)
      tableOffence "pos" id (posRow n) ps
    , -- ascending symbols: the table is a deduped SET of (node,
      -- visibility) pairs, so a repeat is a producer that lost its
      -- set semantics, not a second declaration
      tableOffence "symbol" id (symRow n) ss
    , -- ascending langs: duplicate-free, so the confidence lookup's
      -- first match is the only match
      tableOffence "unres" (take 1) unresRow us
    ]
 where
  n = fromIntegral (length (reqNodes req))
  es = reqEdges req
  ps = reqPos req
  us = unresRows req
  ss = symRows req

-- | The unres table as sent, [] when absent — the cap's and the
-- validator's view; road selection stays on reqUnres's Maybe.
unresRows :: GraphReq -> [[Integer]]
unresRows = maybe [] id . reqUnres

-- | The symbols table as sent, [] when absent — the cap's and the
-- validator's view. Nothing selects a road on it: an empty table and
-- an absent one both name no export surface, so the reply is the
-- same bytes either way (K5).
symRows :: GraphReq -> [[Integer]]
symRows = maybe [] id . reqSymbols

-- | One export-surface row: [node, visibility]. The node index is
-- bounded like an edge endpoint; the visibility word is opaque bits
-- (Cost.exportVisBit picks the one that judges), so only its sign is
-- checked. The openness sits one layer lower than it used to: the
-- producer stores a wider word than it sends and masks the wire's
-- copy to the one bit this core judges on (cli/src/graph/symwire.rs
-- export_surface) — stored word open, wire projection closed — so a
-- bit a later core wants arrives by widening that projection, not
-- by loosening this row.
symRow :: Integer -> Int -> [Integer] -> Maybe String
symRow n = rowCheck "symbol" "malformed row (need [node,visibility])" 2 fields
 where
  fields [node, vis]
    | node < 0 || vis < 0 = Just "negative field"
    | node >= n = Just "node out of range"
  fields _ = Nothing

unresRow :: Int -> [Integer] -> Maybe String
unresRow = rowCheck "unres" "malformed row (need [lang,unresolved,total])" 3 fields
 where
  fields [lang, unres, total]
    | lang < 0 || lang > 6 = Just "lang outside the judged set"
    | unres < 0 || total < 0 = Just "negative count"
    | unres > total = Just "unresolved above total"
  fields _ = Nothing

-- | One node row: [lang, kind, roles]. ONE arity since 5.0.0 — the
-- pre-2.28 legacy flags column retired, so a wrong-width row is
-- simply malformed and the table-level "mixed arity" refusal has
-- nothing left to say. The three columns mean lang, granularity and
-- ROLE FACTS; the pre-5.0.0 three meant lang, granularity and flags,
-- and the arity is reused deliberately — a major forbids any
-- cross-version conversation at the envelope, which is exactly what
-- makes reuse safe (VERSIONING §2).
nodeRow :: Int -> [Integer] -> Maybe String
nodeRow = rowCheck "node" "malformed row (need [lang,kind,roles])" 3 nonNegative

edgeRow :: Integer -> Int -> [Integer] -> Maybe String
edgeRow n = rowCheck "edge" "malformed row (need [src,dst,kind,rung])" 4 fields
 where
  fields row@[src, dst, _, _]
    | Just m <- nonNegative row = Just m
    | src >= n || dst >= n = Just "endpoint out of range"
  fields _ = Nothing

-- | The check nearly every row shares: no field may be negative.
nonNegative :: [Integer] -> Maybe String
nonNegative row = if any (< 0) row then Just "negative field" else Nothing

-- notAscending moved to CE.Wire (its birthplace was here — the
-- tenth ratchet bite promoted it to the shared skeleton).

posRow :: Integer -> Int -> Integer -> Maybe String
posRow n i p
  | p < 0 || p >= n = Just ("pos " <> show i <> ": index out of range")
  | otherwise = Nothing
