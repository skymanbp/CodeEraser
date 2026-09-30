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
-- 7.4.0 = the fourteenth judgment family, `flow/1` (plan v2.31 step
-- 3; ADR-008 seventh instalment, design booklet
-- docs/reference/analysis-track.md §5), additive: a new request type
-- `flow.request` — every unit's statements as a pre-order tree of
-- kinds and flags, its variables with their declaring statement and
-- exemption flags, its accesses in evaluation order — answered by
-- `flow.result`: the findings (unreachable runs, dead stores, unused
-- locals, unused parameters as advice) and the counts (CE.Flow and
-- its modules). Every existing family answers byte for byte as
-- before; the hello's capability list grows by one name. The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.4.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
