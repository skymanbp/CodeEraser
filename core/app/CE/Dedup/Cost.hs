-- | Dedup-family constants (batch-7 slice 1), its own module by the
-- CE.Graph.Cost convention: a family's policy constants live where the
-- batteries and the ablation table can reach them, bound to the
-- judgment only at the family boundary.
--
-- Until 2.19.0 the diversity floor lived ONLY in Rust
-- (cli/src/dedup/pairs.rs), guarding a deny path the core could
-- neither see nor ablate; the FPR ledger that admitted that deny
-- tier (methodology 11) is calibrated for exactly this number. Since
-- plan v2.33 W3 the measuring side holds no copy: it reads the floor
-- and the hot-group cap from the definition package (CE.Limits).
module CE.Dedup.Cost (dedupKgram, dedupWindow, hotCap, minDistinct) where

-- | The winnowing operating point the report states (plan v2.32 step
-- 5): k-gram 25 and window 26, so the guarantee t = window + kgram − 1
-- = 50 tokens (cli/src/dedup/mod.rs `Params::default`, which measures
-- with them).
dedupKgram, dedupWindow :: Integer
dedupKgram = 25
dedupWindow = 26

-- | The diversity floor: a T1/T2 block is admitted only when its
-- token stream carries at least this many DISTINCT token kinds.
-- Calibration (M2, contracts/fixtures/crosscheck/
-- DEDUP-CALIBRATION.md): across the fixture corpus + cobra +
-- requests, arbitrated data-row false positives (status-code rows,
-- locale key sections, pygments style dicts) measured distinct <= 6
-- while arbitrated true clones measured >= 7 — the floor buys
-- precision, not purity (one 16-outlier FP survives).
minDistinct :: Integer
minDistinct = 7

-- | The hot-group cap (attack review D4): a group of more members than
-- this that share one hash is paired as an adjacent chain (n − 1 pairs)
-- instead of every two (C(n, 2)), and the chaining is counted. One cap
-- for every hash-group walk: the T1/T2 extension's fingerprint groups
-- (cli/src/dedup/pairs.rs reads it from the package), the T3 sources S3
-- and S4 and the docdup coarse filter (CE.Candidates.Groups).
hotCap :: Integer
hotCap = 64
