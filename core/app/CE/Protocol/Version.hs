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
-- 6.1.0 = the RG10 firewall reaches the faces that act (plan v2.14
-- K15, K round step 5, 2026-08-25). CE.Graph.Dead splits dead along
-- indegree x reachability precisely so "a library's exported-but-
-- unreferenced API can never collapse into plain dead" -- the
-- firewall is a verdict CODE, not a policy. 4.1.0 gave flag bit 0 a
-- producer and the two public codes started firing; the two faces
-- DOWNSTREAM of that verdict were still reading past the code. The
-- erase plan judged class-3 rows on confidence alone, so a public
-- API became an eraseable row; the join lattice synthesized
-- pFlags = 0, so publicGuard could not forbid a `delete` on an
-- exported flank. Additively: `verdict/1` accepts `symbols`, the
-- same [node, visibility] table graph/1 has carried since 4.1.0,
-- re-keyed to the tier universe -- the RAW word, because which bit
-- means exported is judgment (Graph.Cost.exportVisBit) and stays
-- here; and erase reason code 6 `public_surface` joins a frozen
-- domain that only ever grows. Absent table, empty domain: a legacy
-- request answers byte for byte (K15).
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
proto = "6.1.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
