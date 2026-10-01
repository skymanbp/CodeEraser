-- | The erase-family predicate (M9 batch 3, docs/reference/erase.md):
-- which candidate rows are DETERMINISTIC-SAFE to erase. The bytes are
-- Rust's measurement (byte equality, word counts, coverage, liveness
-- lookups arrive as integer facts); WHICH rows are safe is judgment
-- and lives here (ADR-008). Classes are frozen positions:
--   0 (retired)   was dead_file on the local-count road; superseded
--                  at 2.32.0 by class 3 and RETIRED at 4.0.0 once
--                  the grace window closed. The position stays
--                  frozen and is refused by name — renumbering the
--                  survivors would move three other frozen codes
--   1 verbatim_doc docdup pair; safe only as FULL-segment byte-equal
--   2 t1_twin      whole-unit T1 twin whose copy's file is dead —
--                  fact 3 is the copy's dead VERDICT code since 7.0.0
--                  (0 not dead / 1..4 the graph family's four-way
--                  code), so the RG10 bar below reaches twins by the
--                  same publicDeadVerdicts table instead of by the
--                  planner's row-closure order; a 6.x boolean 1 reads
--                  as unref_private and judges exactly as it did (its
--                  language-trust fact stays the local count: a twin
--                  row is not a graph dead row, so no core confidence
--                  exists for it — the H3 scope line)
--   3 dead_file    the confidence road (2.32.0, H3): fact 1 is the
--                  graph family's OWN per-row confidence (0
--                  unvouched / 1 vacuous / 2 vouched) — refused
--                  only at 0, so the trust judgment lives with the
--                  family that owns the site ledger
-- Reason codes are frozen positions too — the advisory face names
-- WHY a row is refused, never just that it was:
--   0 eraseable / 1 language_unresolved / 2 not_full_segment
--   3 bytes_differ / 4 copy_not_dead / 5 unit_not_covered
--   6 public_surface (6.1.0) — the RG10 firewall reaching this face
-- v1 has NO knobs: the safety predicate is not tunable — a knob that
-- loosens "safe" would be a licence to guess (contract §boundaries).
module CE.Erase.Cost (
  advisoryFirst,
  classNames,
  classOf,
  eraseRowCap,
  judgeRow,
  keptRows,
  licence,
  publicDeadVerdicts,
  reasonNames,
) where

import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | The dead verdicts an erase plan may never act on (6.1.0): 2
-- unref_public and 4 unreach_public. CE.Graph.Dead splits dead along
-- indegree × reachability precisely so "a library's exported-but-
-- unreferenced API can never collapse into plain dead" — the RG10
-- firewall is a verdict CODE, not a policy. Until 4.1.0 gave flag
-- bit 0 a producer the two codes could not fire, and this face read
-- past the code to the confidence alone; the moment they could fire,
-- reading past them turned the firewall into a deletion proposal.
-- Data, not a guard, so a battery can permute it.
publicDeadVerdicts :: [Integer]
publicDeadVerdicts = [2, 4]

-- | The class and reason names by their frozen positions (above) —
-- the one spelling the erase plan, its console and the trail print
-- (plan v2.32 step 5).
classNames, reasonNames :: [String]
classNames = ["(retired)", "verbatim_doc", "t1_twin", "dead_file"]
reasonNames = words "eraseable language_unresolved not_full_segment bytes_differ copy_not_dead unit_not_covered public_surface"

-- | Row ceiling: candidates are bounded by dead files + verbatim
-- pairs + whole-unit twins; 4096 is far above any honest plan.
-- Over-cap answers a complete degraded reply that FAILS.
eraseRowCap :: Integer
eraseRowCap = 4096

-- | One row's verdict: (eraseable, reason). The first failing fact
-- in a fixed order names the refusal; a malformed class never
-- reaches here (famOffence refused it by name).
judgeRow :: [Integer] -> (Bool, Integer)
judgeRow [1, verbatim, wordsA, wordsB, bytesEqual]
  | verbatim < wordsA || verbatim < wordsB = (False, 2)
  | bytesEqual /= 1 = (False, 3)
  | otherwise = (True, 0)
judgeRow [2, unitCovered, bytesEqual, copyDead, langUnresolved]
  | unitCovered /= 1 = (False, 5)
  | bytesEqual /= 1 = (False, 3)
  | copyDead == 0 = (False, 4)
  | copyDead `elem` publicDeadVerdicts = (False, 6)
  | langUnresolved /= 0 = (False, 1)
  | otherwise = (True, 0)
-- a public surface is refused BEFORE the trust fact is weighed: no
-- amount of confidence makes an exported API eraseable, so naming
-- the categorical bar tells the reader more than naming the strength
judgeRow [3, verdict, conf, _, _]
  | verdict `elem` publicDeadVerdicts = (False, 6)
  | conf == 0 = (False, 1)
  | otherwise = (True, 0)
judgeRow _ = (False, 4) -- unreachable behind famOffence; refuse, never erase

-- | A fact row's class — its first field; an empty row (unreachable
-- behind famOffence) reads as the retired class 0, which no rule
-- below licenses.
classOf :: [Integer] -> Integer
classOf row = case row of
  (cls : _) -> cls
  [] -> 0

-- | How much a class's ERASEABLE row tells the reader (7.0.0, O51;
-- judged here since 7.2.0, plan v2.30 step 7b): a twin names the live
-- unit it duplicates, a dead file names only its death, a verbatim
-- segment never shares a target with either. The richer row stands
-- for the target — before 7.0.0 the dead-file row won by class-name
-- order and the twin class could never reach apply.
licence :: Integer -> Integer
licence cls = case cls of
  2 -> 2
  3 -> 1
  _ -> 0

-- | Which ADVISORY row stands when no row of a target is eraseable:
-- the dead file's — its reason names the file's own verdict, the
-- categorical bar (public_surface) or the trust fact
-- (language_unresolved) — over the twin's, whose reason is about the
-- copy. The reader sees the refusal that is about the file.
advisoryFirst :: Integer -> Integer
advisoryFirst cls = case cls of
  3 -> 2
  2 -> 1
  _ -> 0

-- | The target closure (7.2.0, plan v2.30 step 7b): which judged row
-- STANDS for its target. A target is one (path, span) — the whole
-- file, or one line span in it — and several rows may name it: a
-- dead file that is also a byte-identical twin of a live unit names
-- one target twice, once per class. The planner closed the set for
-- itself until this step (erase/mod.rs `close_targets`); which
-- verdict the reader acts on is judgment, so the rule lives here.
--
-- Three rules, in order: (1) an eraseable whole-file row owns its
-- path, and every span row on that path is closed out — an apply
-- that deleted the file and then spliced lines out of it would
-- refuse on the hash it can no longer read; (2) within one target
-- the eraseable row with the richest `licence` stands; (3) with no
-- eraseable row the `advisoryFirst` row stands. Ties inside one
-- class break to the EARLIEST row — which is why the contract asks
-- for key order: two producers sending the same rows in the same
-- order close the same way. Targets, fact rows and verdicts are
-- parallel lists in request order; the answer is one bit per row.
keptRows :: [[Integer]] -> [[Integer]] -> [(Bool, Integer)] -> [Bool]
keptRows targets rows verdicts = [i `S.member` winners | i <- [0 .. length rows - 1]]
 where
  judged = zip3 [0 :: Int ..] (zip targets rows) (map fst verdicts)
  owned = S.fromList [p | (_, ([p, 0, 0], _), True) <- judged]
  subsumed [p, s, _] = s > 0 && p `S.member` owned
  subsumed _ = False
  standing cls eraseable
    | eraseable = (1 :: Integer, licence cls)
    | otherwise = (0, advisoryFirst cls)
  open =
    [ (take 3 t, (standing (classOf r) e, i))
    | (i, (t, r), e) <- judged
    , not (subsumed t)
    ]
  -- fromListWith applies `better new old`, so a strictly better
  -- standing replaces and an equal one keeps the earlier row
  better new old = if fst new > fst old then new else old
  winners = S.fromList (map snd (M.elems (M.fromListWith better open)))
