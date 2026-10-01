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
-- 7.8.0 = the report documents, `document/1` (plan v2.32 step 3;
-- design booklet docs/reference/authority-track.md §5), additive: a
-- `document.request` names a family (arch, query, rules, flow,
-- merge) and sends the integer tables, ranges and facts its document
-- is assembled from; `document.result` answers the document with
-- every repository string a reference `{"$": [class, integers…]}`
-- (CE.Document). The definition package gains `document`, the
-- catalogue of each family's schema id and empty document, so
-- `tablesDigest` moves. Every judgment family answers byte for byte
-- as before; the hello's capability list grows by one name.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.8.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
