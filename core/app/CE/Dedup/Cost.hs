-- | Dedup-family constants (batch-7 slice 1) — one number today,
-- its own module by the CE.Graph.Cost convention: a family's policy
-- constants live where the batteries and the ablation table can
-- reach them, bound to the judgment only at the family boundary.
--
-- Until 2.19.0 this floor lived ONLY in Rust
-- (cli/src/dedup/pairs.rs), guarding a deny path the core could
-- neither see nor ablate; the FPR ledger that admitted that deny
-- tier (methodology 11) is calibrated for exactly this number. The
-- Rust constant survives as the declared mirror the gated path
-- proves equal on every run.
module CE.Dedup.Cost (minDistinct) where

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
