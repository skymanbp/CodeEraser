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
-- 7.10.0 = the console form in `document/1` (plan v2.32 step 5;
-- design booklet docs/reference/authority-track.md §6), additive: a
-- `document.request` may name a language (`lang`, 0 en / 1 zh), the
-- reply carries the family's console `lines` ([stream, text, ref…],
-- every number and product word written, a `{}` hole per reference)
-- and its veto (`exit: {fail}`); the guard and the audit sentences
-- are two more families (`guard`, `audit`, document `{}`, outside the
-- catalogue, so `tablesDigest` stands). Every judgment family answers
-- byte for byte as before; the capability list is unchanged.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.10.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
