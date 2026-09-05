-- | The protocol version this server speaks, what today's number
-- means, and the per-message major check — split from "CE.Protocol"
-- when a growing ledger pushed the envelope module past the E01 size
-- line (the CE.Structure.Stale precedent). Splitting bought room; it
-- did not make an ever-growing mirror right, and the room ran out
-- again at 6.1.0. So the standing rule is a SUBSTITUTION, not an
-- append: each version replaces the last one's entry here, and the
-- history lives at its address.
module CE.Protocol.Version (majorMatches, proto) where

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 6.7.0 = the similar family (ADR-008 sixth instalment, plan v2.29),
-- additive: one new family, similar/1. The measuring side ranks a
-- query's candidates off its own inverted tables and sends the query
-- bag as [termHash, weight] pairs plus one [nHit, pHit, cHit, dHit,
-- sHit, lHit, shapeEqual, bm25Num, bm25Den] row per candidate; this
-- side answers the order they stand in (exact rationals) and which of
-- them play the query's role (a shared name AND callee, or two shared
-- names with the shape equal). The conjunction and its floors exist in
-- one place (CE.Similar.Cost); the measuring side re-labels indices
-- and never judges. An advisor: no condition bit, no knob. Every other
-- family is answered byte for byte as before.
-- The per-version ledger lives in contracts/VERSIONING.md and nowhere
-- else; only THIS version's entry stays beside the constant. The
-- reason the mirrors were retired is written once, at the client's
-- constant (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "6.7.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
