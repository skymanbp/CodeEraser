-- | The protocol version this server speaks, its change ledger, and
-- the per-message major check — split from "CE.Protocol" when the
-- growing ledger pushed the envelope module past the E01 size line
-- (the CE.Structure.Stale precedent): the ledger lives with the
-- constant it documents, and every future minor's entry lands here.
module CE.Protocol.Version (majorMatches, proto) where

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 6.0.0 = the fingerprint widens (plan v2.14 (2), K round step 4b,
-- 2026-08-25): 5.1.0's `classDigest` becomes `knobsDigest` and covers
-- the WHOLE parsed ce.toml, not the class table alone. An adversarial
-- review found the first scope wrong within the hour and the finding
-- reproduced first-party: `[score] viol_cost = 0` takes a repo from
-- 939/1000 FAIL to 1000/1000 pass while the axes still report the
-- violation, and `[score] tol_abs = 100000` turns a 280-line growth
-- from "1 over -> FAIL" into "0 over, 1 tolerance drawn -> pass".
-- Both move the gates a glob edit moves; neither touched the class
-- table; neither asked anyone to name a floor. Choosing which tables
-- to fence is how that happens, so nothing is chosen: the scalar is
-- fnv1a over the serialized config, which covers the knob nobody has
-- added yet and moves for none of the comments or key order in the
-- file, because it fingerprints the PARSE. A repo whose config equals
-- the shipped default still sends nothing. The fail condition renames
-- with the key (`knobs_digest`), and the rename of a request field is
-- a schema change, hence the major (VERSIONING section 2) -- the same
-- rule 5.0.0 was measured against.
-- 5.1.0 = the rulepack fence (plan v2.14 ②, K round step 4,
-- 2026-08-25): verdict.request gains the scalar `knobsDigest`, a
-- fingerprint of the normalized [[rules.class]] declaration (names,
-- globs in declaration order, knobs); ce-baseline.json records the
-- digest its ceilings were established under; and the fail table
-- gains the named condition `knobs_digest`, which holds on plain
-- Maybe inequality -- both absent agrees, a changed rulepack
-- disagrees, and so does declaring one against a pre-fence baseline
-- or removing one the baseline recorded. Only establish writes a
-- digest, so agreeing to a new rulepack is the same named act as
-- agreeing to a new floor. classKnobs gains code 3, a class's OWN
-- ratchet allowance in lines: declared, it replaces both global legs,
-- so 0 means a class may not grow by one line and the global
-- max(+2%, +10) cannot rescue it. It is the only class knob whose
-- zero is meaningful, which is why the table's value bound is judged
-- per code. A repo declaring no class sends no digest and no class
-- knobs and answers byte for byte as before.
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
proto = "6.0.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
