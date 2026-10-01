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
-- 7.5.0 = the fifteenth judgment family, `merge/1` (plan v2.31 step
-- 6; ADR-008 seventh instalment, design booklet
-- docs/reference/analysis-track.md §6), additive: a new request type
-- `merge.request` — the clone groups with their family, the members
-- with their lines and file in-degree, one tree per member in
-- clone/1's postorder encoding with a leaf-hash and a position-class
-- column — answered by `merge.result`: per group the parameter count,
-- the member kept, the lines saved, feasibility and its reason, per
-- hole each member's first and last root, and the counts (CE.Merge
-- and its modules); clone/1's tree gains the optional `leaf` column,
-- which its judgment never reads. Every existing family answers byte
-- for byte as before; the hello's capability list grows by one name.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.5.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
