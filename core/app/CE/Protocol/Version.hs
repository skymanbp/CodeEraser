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
-- 6.4.0 = the fence close-out batch (plan v2.18 step #14, piece
-- (b)), additive on three families. verdict/1: class knob code 4
-- `cognitive_ratchet_tolerance` replaces code 3 for CoC alone
-- (O37); class ids 1..=64 are admitted, one predicate at four
-- readers (O38); the `present` provenance table answers with
-- `ratchet.dropped` and the sixth fail name `rows_dropped` (O40);
-- thresholds code 7 `cycleFloor` with the `cycleSelfLoops` table
-- required exactly at floor 1 (O59); the degraded reply echoes the
-- request digest like the judged one (O43). scan/1: `knobsFence`
-- rides and the reply names `failed` (O33). graph/1: `sccFloor`
-- declared and echoed (O59). A request without the new keys is
-- judged byte for byte as before (K16).
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
proto = "6.4.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
