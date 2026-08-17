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
  , structScale
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

-- | Per-mille cost of one violation in the structure score fold
-- (the Verdict violCost anchor).
structViolCost :: Integer
structViolCost = 10

-- | The structure score's opening value (per-mille scale).
structScale :: Integer
structScale = 1000

-- | Node ceiling (the verdictRowCap magnitude anchor): over-cap
-- answers a complete degraded reply that FAILS (the P1 posture).
structNodeCap :: Integer
structNodeCap = 524288
