-- | The tombstone family's constants (plan v2.27 step 4; ADR-008 fifth
-- instalment): the residue conjunction and its floors, repatriated
-- from cli/src/tombstone where stage one judged them in Rust. Rust
-- measures three integers per candidate surface — its kind, the
-- retrospective marks in it, the erased names it binds; WHICH rows
-- are sites and whether a changeset is over its budget is judgment
-- and lives here. Every rule below is a consequence of these numbers.
module CE.Tombstone.Cost (
  tombstoneRowCap,
  minName,
  minMarks,
  kindProse,
  budgetCode,
  isSite,
  overBudget,
) where

-- | Row ceiling: one row per candidate surface of one changeset. A
-- whole-history replay commit carries a few dozen; 65536 is far above
-- any honest changeset. Over-cap answers a complete degraded reply.
tombstoneRowCap :: Integer
tombstoneRowCap = 65536

-- | A site binds at least this many erased names — the label rule
-- whole, and half of the prose conjunction.
minName :: Integer
minName = 1

-- | A prose site carries at least this many retrospective marks in the
-- SAME sentence — the other half (spec §三 M4: a mark alone is a
-- sentence about something else, a name alone is a mention). The
-- conjunction is what the replay rounds measured the precision of
-- (docs/FPR-TOMBSTONE.md); loosening either floor re-opens them.
minMarks :: Integer
minMarks = 1

-- | Row kinds: 0 = bracketed label, 1 = bare label, 2 = prose
-- sentence. Only the prose kind reads the marks column.
kindProse :: Integer
kindProse = 2

-- | The one knob code: `[tombstone] budget` — sites one changeset may
-- carry. Absent = the condition is never evaluated.
budgetCode :: Integer
budgetCode = 0

-- | One row's verdict.
isSite :: [Integer] -> Bool
isSite [kind, marks, names]
  | kind == kindProse = marks >= minMarks && names >= minName
  | otherwise = names >= minName
isSite _ = False -- unreachable behind famOffence; refuse, never convict

-- | The changeset's condition: strictly more sites than the declared
-- budget; no budget, no condition (feed-only — `[dedup] budget`'s
-- precedent for an absent knob).
overBudget :: Maybe Integer -> Int -> Bool
overBudget budget sites = maybe False (\b -> toInteger sites > b) budget
