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
-- 6.2.0 = the symbol-level advisory reaches the wire (plan v2.17 L
-- round piece (6), 2026-08-27). graph/1 accepts two ADDITIVE tables
-- that travel together or not at all: `unmentioned` = [[node, vis,
-- conv]], the declarations no other file of the corpus spells (the
-- producer's negative mention instrument, hashes only, cli/src/
-- mention), and `mounts` = [[node, private, total, bits]], every
-- node's mount facts. The reply gains `exportUnmentioned` =
-- [[node, vis, conv, code]] -- 0 public / 1 private / 2 restricted /
-- 3 reexported, the folds of CE.Graph.Advisory -- when the request
-- carried the table, or `unmentionedDropped` when the table exceeded
-- its soft cap: an advisory that can never touch `dead`, `fail` or
-- `degraded`, by construction. Absent tables: a legacy request
-- answers byte for byte (K16); `symbols` still crosses masked to
-- bit 0 (K34); `reqSymbols` now counts toward verdict/1's row cap
-- (K47, the C15 discipline).
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
proto = "6.2.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
