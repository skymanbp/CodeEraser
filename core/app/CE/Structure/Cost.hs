-- | The structure-family knobs (M6 S2) — nothing but constants and
-- the knob table (the CE.Verdict.Cost convention: batteries and the
-- ablation table have exactly one target; ce.toml is the source and
-- these are the DEFAULTS on the 27b9bc2 road). One predicate, one
-- knob, per the design booklet §3; the S0 axis carries two
-- predicates (depth, fanout) and therefore two knobs.
module CE.Structure.Cost
  ( depthCeil
  , fanoutCeil
  , namingMin
  , namingCeil
  , mixRefFloor
  , misplaceMin
  , bigDirFloor
  , structViolCost
  , structViolCostNeutral
  , structScale
  , dupMin
  , deadMin
  , staleMin
  , modFloor
  , modMassFloor
  , seamSoft
  , seamHard
  , seamPMax
  , roiRefMilli
  , roiPhiMilli
  , roiCloneMilli
  , roiChurnMilli
  , structNodeCap
  ) where

-- | S0: directories deeper than this are path-geometry violations.
-- Deep trees hide files from every reader; 8 levels covers the
-- deepest layout in the scanned ecosystems' conventions.
depthCeil :: Integer
depthCeil = 8

-- | S0: immediate children (files + subdirs) over this count — a
-- directory nobody can hold in one glance.
fanoutCeil :: Integer
fanoutCeil = 30

-- | S1: sibling sets smaller than this are not judged for naming
-- consistency (tiny sets have no distribution to speak of — the
-- F16 non-vacuity stance applied at the knob level).
namingMin :: Integer
namingMin = 5

-- | S1: normalized naming Tsallis-2 (‰) above this = an
-- inconsistent sibling set. 600‰ tolerates one odd name in a
-- convention-following set and flags a genuine style mix.
namingCeil :: Integer
namingCeil = 600

-- | S2: a directory participates in the mixing judgment only when
-- its total reference traffic (intra + inter) reaches this floor —
-- below it there is no geometry to judge.
mixRefFloor :: Integer
mixRefFloor = 5

-- | S3: a file is misplacement-judged only when its outside
-- references reach this floor (and outside > 2×inside — the ratio
-- is part of the predicate's definition, not a separate knob in
-- v1).
misplaceMin :: Integer
misplaceMin = 3

-- | S4: directories with at least this many files owe their
-- readers a README.
bigDirFloor :: Integer
bigDirFloor = 8

-- | Cost multiplier per unit of axis charge in the structure score
-- fold (the Verdict violCost anchor) — the family's strictness
-- dial since the 2.26.0 density fold.
structViolCost :: Integer
structViolCost = 10

-- | The dial's neutral point (the CE.Verdict.Cost.violCostNeutral
-- convention, batch 9 P9): at violCost == neutral the fold reduces
-- to the plain mean of the bounded axis charges and the score
-- cannot saturate; dialing above neutral is an explicit request
-- for harsher scores. A structural constant of the fold, not a
-- knob.
structViolCostNeutral :: Integer
structViolCostNeutral = 10

-- | The structure score's opening value (per-mille scale).
structScale :: Integer
structScale = 1000

-- | S6: a directory joins the redundancy finding when its clone
-- blocks reach this floor — one block is already this tool's whole
-- argument, so the default floor is 1 (knob code 9, landing order:
-- the S3c staleness knob takes 11).
dupMin :: Integer
dupMin = 1

-- | S6: dead (unreachable) units per directory at or above this
-- count flag the directory — orphan code is never furniture
-- (knob code 10).
deadMin :: Integer
deadMin = 1

-- | S5: stale documents per directory at or above this count flag
-- the directory — a doc whose referenced code moved on after its
-- last edit is entropy in prose form (knob code 11).
staleMin :: Integer
staleMin = 1

-- | S7 (knob 19, O54): the per-mille floor on a directory's
-- NORMALIZED modularity contribution — how much of the cohesion its
-- own incident mass could buy it actually keeps (1000 = every edge
-- internal, 0 = exactly the null model's expectation, below 0 = worse
-- than a random graph with these degrees). 1 is the least the knob
-- grammar can express (every structure knob is >= 1) AND the line the
-- measure itself draws, so the default flags only directories that do
-- no better than chance. The 1/3 locality line already belongs to S2
-- mixing — `outs > ins` on touch counts IS `inside share < 1/3` — and
-- putting S7 at the null model instead keeps one phenomenon on one
-- axis (structure-axes.md's coverage map). What S7 sees and S2 cannot
-- is the monolith: a directory holding most of the tree's edges
-- internally passes S2 loudly and still fails the null model, because
-- most of the degree mass is its own.
modFloor :: Integer
modFloor = 1

-- | S7 (knob 20, O54): the smallest incident-edge mass at which a
-- directory is judged at all. Below four edges the sign of the
-- contribution is decided by one reference, and a directory nobody has
-- wired up yet is not a failed module. Four is also the smallest mass
-- at which a directory can hold as many internal edges as crossing
-- ones and still cross at all (2 + 2; at three, parity forbids
-- e == cross). Under the floor a directory is ABSENT from the axis —
-- never counted clean, never flagged (the F16 non-vacuity stance at
-- the knob level, mixRefFloor's precedent).
modMassFloor :: Integer
modMassFloor = 4

-- | Split-ROI (plan v2.6 §C, knob codes 12..14): the advisory's own
-- copy of the zone triple — S/H/P_max — because the structure
-- family must price a seam without a verdict/1 request in flight.
-- Rust sends the SAME numbers it sends verdict/1 — the committed
-- softLine, ce.toml's file_lines_fail, and (6.1.0) score's
-- size_penalty_max when one is declared — so the two families cannot
-- disagree about the curve (knob code 12 / 13 / 14). Until 6.1.0
-- only 12 and 13 rode: a repo declaring size_penalty_max got the
-- declared P_max in its score and the default 10 in its advisory,
-- and both halves looked right on their own.
seamSoft :: Integer
seamSoft = 300

seamHard :: Integer
seamHard = 750

seamPMax :: Integer
seamPMax = 10

-- | §C cost prices, in MILLI penalty-units (knob codes 15 / 16):
-- each internal reference a seam severs costs roiRefMilli, and
-- every new file costs the flat roiPhiMilli (the S0-fanout /
-- mental-load overhead the booklet names φ). Defaults sized so a
-- mid-zone file with a clean seam clears ROI 1 and one with 10+
-- crossing references does not.
roiRefMilli :: Integer
roiRefMilli = 250

roiPhiMilli :: Integer
roiPhiMilli = 500

-- | v1.1 prices (plan v2.7 ②, knob codes 17 / 18): a seam CUTTING
-- THROUGH a T1/T2 clone block splits one coherent duplicate span
-- over two files — dearer than one severed reference; a seam
-- crossing a unit co-change pair (two units the churn window edits
-- together) makes every future such edit a two-file edit. Values
-- from the v0.7 external-corpora calibration (booklet §v1.1).
roiCloneMilli :: Integer
roiCloneMilli = 500

roiChurnMilli :: Integer
roiChurnMilli = 150

-- | Node ceiling (the verdictRowCap magnitude anchor): over-cap
-- answers a complete degraded reply that FAILS (the P1 posture).
structNodeCap :: Integer
structNodeCap = 524288
