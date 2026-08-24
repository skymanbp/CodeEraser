-- | The erase-family predicate (M9 batch 3, docs/reference/erase.md):
-- which candidate rows are DETERMINISTIC-SAFE to erase. The bytes are
-- Rust's measurement (byte equality, word counts, coverage, liveness
-- lookups arrive as integer facts); WHICH rows are safe is judgment
-- and lives here (ADR-008). Classes are frozen positions:
--   0 dead_file    graph file-tier dead; refused while its language
--                  still carries unresolved sites (a verdict that
--                  assumed no in-corpus lands is not a deletion
--                  licence). SUPERSEDED at 2.32.0 by class 3 — kept
--                  judging unchanged for the grace window (the
--                  staleDocs discipline), retire in a later minor
--   1 verbatim_doc docdup pair; safe only as FULL-segment byte-equal
--   2 t1_twin      whole-unit T1 twin whose copy's file is dead
--                  (its language-trust fact stays the local count:
--                  a twin row is not a graph dead row, so no core
--                  confidence exists for it — the H3 scope line)
--   3 dead_file    the confidence road (2.32.0, H3): fact 1 is the
--                  graph family's OWN per-row confidence (0
--                  unvouched / 1 vacuous / 2 vouched) — refused
--                  only at 0, so the trust judgment lives with the
--                  family that owns the site ledger
-- Reason codes are frozen positions too — the advisory face names
-- WHY a row is refused, never just that it was:
--   0 eraseable / 1 language_unresolved / 2 not_full_segment
--   3 bytes_differ / 4 copy_not_dead / 5 unit_not_covered
-- v1 has NO knobs: the safety predicate is not tunable — a knob that
-- loosens "safe" would be a licence to guess (contract §boundaries).
module CE.Erase.Cost (
  eraseRowCap,
  judgeRow,
) where

-- | Row ceiling: candidates are bounded by dead files + verbatim
-- pairs + whole-unit twins; 4096 is far above any honest plan.
-- Over-cap answers a complete degraded reply that FAILS.
eraseRowCap :: Integer
eraseRowCap = 4096

-- | One row's verdict: (eraseable, reason). The first failing fact
-- in a fixed order names the refusal; a malformed class never
-- reaches here (famOffence refused it by name).
judgeRow :: [Integer] -> (Bool, Integer)
judgeRow [0, _verdict, langUnresolved, _, _]
  | langUnresolved /= 0 = (False, 1)
  | otherwise = (True, 0)
judgeRow [1, verbatim, wordsA, wordsB, bytesEqual]
  | verbatim < wordsA || verbatim < wordsB = (False, 2)
  | bytesEqual /= 1 = (False, 3)
  | otherwise = (True, 0)
judgeRow [2, unitCovered, bytesEqual, copyFileDead, langUnresolved]
  | unitCovered /= 1 = (False, 5)
  | bytesEqual /= 1 = (False, 3)
  | copyFileDead /= 1 = (False, 4)
  | langUnresolved /= 0 = (False, 1)
  | otherwise = (True, 0)
judgeRow [3, _verdict, conf, _, _]
  | conf == 0 = (False, 1)
  | otherwise = (True, 0)
judgeRow _ = (False, 4) -- unreachable behind famOffence; refuse, never erase
