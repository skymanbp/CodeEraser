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
-- 8.5.0 = plan v2.33 wave W3's four stages (design booklet
-- docs/reference/algorithm-track.md §11 items 29-38), four additive
-- minors on 8.1.0 landed together: 8.2.0 `candidates/1` (the T3
-- candidate pass), clone/1's `decide` and the definition package's
-- `limits`; 8.3.0 `rank/1` (the advisor's ranking); 8.4.0
-- `docpairs/1` (the docdup coarse filter) and docdup/1's `seqs`;
-- 8.5.0 `moves/1` (fourclass L1). `tablesDigest` moves with `limits`;
-- every older family answers byte for byte as before.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "8.5.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
