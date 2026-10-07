-- | The protocol version this server speaks, what today's number
-- means, and the per-message major check — split from "CE.Protocol"
-- when a growing ledger pushed the envelope module past the E01 size
-- line (the CE.Structure.Stale precedent). Splitting bought room; it
-- did not make an ever-growing mirror right, and the room ran out
-- again at 6.1.0. So the standing rule is a SUBSTITUTION, not an
-- append: each version replaces the last one's entry here, and the
-- history lives at its address.
module CE.Protocol.Version (majorMatches, maxLineBytes, proto) where

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 9.0.0 = plan v2.33 W2-text stage A (design booklet
-- docs/reference/algorithm-track.md §3, §6 row W2a′): `resolve/1` carries
-- text — paths, specifiers and configuration files as strings — and the
-- core reads go.mod, the root pyproject.toml's keys, the compile
-- databases, their response files and flag files; the request's segment
-- table, vocabulary, affix rows and directory table are retired, and the
-- definition package's `resolve` key now names the config basenames the
-- request carries. A retired key is a major; every other family answers
-- byte for byte as before.
-- The per-version
-- ledger lives in contracts/VERSIONING.md and nowhere else; only
-- THIS version's entry stays beside the constant. The reason the
-- mirrors were retired is written once, at the client's constant
-- (cli/src/corelink.rs::PROTO) -- it is not repeated here.

proto :: String
proto = "9.0.0"

-- | The request line's byte ceiling, checked by "CE.Protocol" before any
-- JSON parse, so a hostile oversized line is never decoded. Relaxed from
-- 1 MiB at M5-2a (2026-08-12 decision): the only client is the trusted
-- same-machine daemon, and a graph request legitimately carries ~1 MB per
-- 100k LOC — the real per-family guards are CE.Graph.Cost's node/edge
-- caps. Here, beside the version, because the definition package names it
-- too (CE.Limits `line_bytes`: the bags/1 batches are planned by it, plan
-- v2.33 W2-text Z4) and "CE.Protocol" imports the package.
maxLineBytes :: Int
maxLineBytes = 33554432

-- | The per-message major check (§1): a request without a proto, or
-- with a foreign major, is never answered as if it negotiated.
majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto
