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
-- 7.2.0 = the judged-language set rides the wire (plan v2.30 step 1),
-- additive: `scan.request` and `graph.request` accept `judgedMask`,
-- the producer's own bitmask of judged language codes, and the two
-- validators that used to bound a naming row's or an unres row's
-- code with the constant 6 now test the bit (CE.Wire.judgedLang). A
-- request without the key is judged against the legacy seven, byte
-- for byte; a judged reply echoes the mask exactly when it rode. A
-- language turning judged on the Rust side is thereby a request
-- fact, never a core release. No capability name changes. Step 2 of
-- the same plan (C / C++) adds, inside this unreleased minor, roleBits
-- row 8: a C-family compilation unit — a `.c` file nothing includes —
-- lands on the executable bit (CE.Graph.Cost).
-- The per-version ledger lives in contracts/VERSIONING.md and nowhere
-- else; only THIS version's entry stays beside the constant. The
-- reason the mirrors were retired is written once, at the client's
-- constant (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.2.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
