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
-- 7.1.0 = the structure family's directed dir-edge table (plan v2.29
-- step 10 batch C3, O54), additive: `structure.request` accepts
-- `dirEdges` ([fromDir, toDir, count], crossing edges only), and the
-- reply gains axis 7 -- modularity -- exactly when that table rides,
-- with knob codes 19/20 (the per-mille floor on a directory's
-- normalized Newman contribution, and the incident-edge mass below
-- which it is not judged). The intra mass is fileRefs' `inside` sum
-- halved and never rides twice; the two tables are held to one graph
-- by a boundary law. O48 adds declaration relocation to fourclass in
-- the same unreleased minor: paired declRem/declAdd tables receive
-- unitEdges (or unitEdgesDropped over the cap). Line judgments and
-- replies without declaration tables are unchanged.
-- The per-version ledger lives in contracts/VERSIONING.md and nowhere
-- else; only THIS version's entry stays beside the constant. The
-- reason the mirrors were retired is written once, at the client's
-- constant (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.1.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
