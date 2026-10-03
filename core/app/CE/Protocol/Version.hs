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
-- 8.0.0 = the retired request key (plan v2.32 step 6; design booklet
-- docs/reference/authority-track.md §3, §13 item 18): `judgedMask`
-- leaves scan/1, graph/1 and verdict — the core reads the judged set
-- off its own language table (CE.Wire.Mask), a request that still
-- carries the key is refused by name, and no reply echoes it. Major
-- because a key that was accepted is now refused; every document and
-- console line answers byte for byte as before.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "8.0.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
