-- | The audit-family verdict constants (M9 batch 7 slice 7): the
-- session/commit duplication rule, repatriated from cli/src/audit.rs
-- where it lived as a Rust match with no owner. The bits are Rust's
-- measurement (set membership of each block side in the session's
-- changed files); WHICH blocks convict and HOW MANY are tolerated is
-- judgment and lives here (ADR-008).
module CE.Audit.Cost (
  auditBlockCap,
  dupTolerance,
  judgeRow,
) where

-- | Row ceiling: one row per clone block in the repo's dedup index —
-- the self repo carries ~200, and 65536 is far above any honest
-- index. Over-cap answers a complete degraded reply that FAILS.
auditBlockCap :: Integer
auditBlockCap = 65536

-- | Zero tolerance, as a NAMED number instead of an emptiness test:
-- a session that leaves ANY duplicate block touching its changed
-- files may not end, a commit that stages one may not land. The
-- whole-repo budget (dedup_budget, CE.Verdict) rides a ratchet
-- because history is paid down over time; the per-session rule has
-- no history to amortise — every new touch is new debt.
dupTolerance :: Integer
dupTolerance = 0

-- | One block's verdict: convicted iff EITHER side sits in the
-- session's changed set. The disjunction is deliberately asymmetric
-- — a pre-existing pair whose other side the session merely brushed
-- convicts, because touching one side of a clone was the moment to
-- collapse it (the v1 attribution stance, audit.rs's own comment;
-- the exact new/old split is the four-class family's job, not this
-- rule's).
judgeRow :: [Integer] -> Bool
judgeRow [a, b] = a /= 0 || b /= 0
judgeRow _ = False -- unreachable behind famOffence; refuse, never convict
