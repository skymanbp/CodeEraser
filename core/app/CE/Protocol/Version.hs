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
-- 6.6.0 = the tombstone family (ADR-008 fifth instalment, plan
-- v2.27), additive: one new family, tombstone/1. The measuring side
-- sends one [kind, marks, erasedNames] row per candidate surface a
-- changeset wrote, plus an optional budget knob; this side judges
-- which rows are sites (a label binding an erased name; a prose
-- sentence with a mark AND a name) and whether the changeset is over
-- its budget, and answers the site indices, their label / prose
-- split and `over`. The conjunction and its floors exist in one place
-- (CE.Tombstone.Cost); the measuring side renders indices back into
-- places and never judges. Every other family is answered byte for
-- byte as before.
-- The per-version change ledger lives in contracts/VERSIONING.md and
-- nowhere else. It used to be mirrored here in English and a third
-- time in cli/src/corelink.rs; the three copies drifted (four entries
-- sat in one English mirror and not the other) and a mirror that
-- gains an entry every minor grows without bound inside a size-gated
-- file. What stays here is THIS version's entry and nothing
-- else, because a reader standing at the constant needs to know what
-- today's number means -- what every past number meant is a ledger
-- question, and the ledger has an address. Four entries had stacked
-- up here by 6.1.0 and pushed the file past its own ratchet: the
-- ledger that documents a size gate is not exempt from it.

proto :: String
proto = "6.6.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
