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
-- 7.0.0 = the judgment-correctness batch (plan v2.29 step 8), a
-- MAJOR because one request field changed shape: the fourclass pair's
-- `dup` (duplicated key hashes) is replaced by `dupSpans`
-- ([hash, start, end] per after-side occurrence), and the stacking
-- rule now asks whether the novel mass landed INSIDE a duplicated unit
-- instead of merely beside one (O47). Riding the same number: the
-- verdict score's clone and docdup axes charge the FILES a verified
-- pair touches over the code-file and doc-file universes instead of
-- pairs over files (O22 -- scores are not comparable with 6.x), and
-- the erase class-2 row's third fact is the copy's four-way dead
-- verdict code rather than a boolean, so the RG10 firewall reaches
-- twins by the same publicDeadVerdicts bar (O51). Every other family
-- is answered byte for byte as before.
-- The per-version ledger lives in contracts/VERSIONING.md and nowhere
-- else; only THIS version's entry stays beside the constant. The
-- reason the mirrors were retired is written once, at the client's
-- constant (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "7.0.0"

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
