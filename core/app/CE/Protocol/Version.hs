-- | The protocol version this server speaks, its change ledger, and
-- the per-message major check — split from "CE.Protocol" when the
-- growing ledger pushed the envelope module past the E01 size line
-- (the CE.Structure.Stale precedent): the ledger lives with the
-- constant it documents, and every future minor's entry lands here.
module CE.Protocol.Version (majorMatches, proto) where

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 5.0.0 = the legacy-flags subtraction (plan v2.14, K round step 3d,
-- 2026-08-25): the graph node row loses the pre-2.28 flags column and
-- becomes [lang, kind, roles] -- one arity, no legacy road. The
-- column was computed and sent for seven minors after 2.28.0 made the
-- roles column authoritative and the core stopped reading it; it
-- survived 4.0.0 only because flags bit 0 was the public/private
-- verdict axis and, with no producer for visibility, dropping the
-- column would have made unref_public and unreach_public
-- inexpressible even to a fixture. 4.1.0's symbols table gave that
-- axis its producer, so the column has nothing left to hold.
-- A schema subtraction is a MAJOR by VERSIONING section 2 (removing a
-- field or changing a row shape), which is why this is 5.0.0 and not
-- the minor the plan first sketched. The three columns now mean lang,
-- granularity and role facts where the old three meant lang,
-- granularity and flags: the arity is reused on purpose, because a
-- major refuses every cross-version conversation at the envelope, and
-- that refusal is what makes reuse safe. The table-level "node rows:
-- mixed arity" refusal retires with it -- with one legal arity a
-- wrong-width row is simply malformed, and says so by row index.
--
-- The per-version change ledger lives in contracts/VERSIONING.md and
-- nowhere else. It used to be mirrored here in English and a third
-- time in cli/src/corelink.rs; the three copies drifted (four entries
-- sat in one English mirror and not the other) and a mirror that
-- gains an entry every minor grows without bound inside a size-gated
-- file. What stays here is the CURRENT major's entry, because a
-- reader standing at the constant needs to know what today's number
-- means -- what every past number meant is a ledger question, and the
-- ledger has an address.

proto :: String
proto = "5.0.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
