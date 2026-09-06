-- | The cost model — all of it. Two per-line and two per-site
-- integer constants (floats tie-break differently across platforms;
-- the output contract is byte determinism). Every downstream rule is
-- a consequence of these four numbers, so the ablation and the plan
-- §7.4 sensitivity test have exactly one target and a dead knob
-- cannot hide.
module CE.FourClass.Cost
  ( movedCost
  , plainCost
  , siteCostWithin
  , siteCostCross
  , siteOpens
  , destFloor
  , anchorFloor
  , declCredit
  , declFloor
  ) where

-- | Cost of explaining a line as moved.
movedCost :: Int
movedCost = 1

-- | Cost of leaving a line novel / deleted.
plainCost :: Int
plainCost = 3

-- | Opening a relocation site inside one file pair is free — which
-- derives L1's unfloored within-file rule (any single matching line).
siteCostWithin :: Int
siteCostWithin = 0

-- | Opening a site across two file pairs costs 2 — from which the
-- two-line cross-file evidence floor is a theorem, not a threshold:
-- a single cross line is 1*1 + 2 = 3 = 1*3, a tie, and ties do not
-- open. That tie IS the coincidence rejection.
siteCostCross :: Int
siteCostCross = 2

-- | A site opens iff explaining its lines as moved beats leaving
-- them plain: n*movedCost + s < n*plainCost. Ties resolve to not
-- opening (conservative). The multiply rides Integer — decision ③
-- (Codex C4, Anchor.overWork) backported at this module's ONLY
-- multiply site: n is bounded by the 32 MiB line cap (< 3.4M
-- entries) so machine Int cannot overflow today, but the judgment
-- must not depend on that arithmetic being redone at every reading.
siteOpens :: Int -> Int -> Bool
siteOpens s n =
  toInteger n * toInteger movedCost + toInteger s
    < toInteger n * toInteger plainCost

-- | Minimal block length that opens a cross-pair site (derived).
destFloor :: Int
destFloor = go 1
 where
  go n = if siteOpens siteCostCross n then n else go (n + 1)

-- | A cross-pair site must also carry ONE evidence line wide enough
-- to hold provenance identity by itself: >= this many alphanumeric
-- chars. Not derived — decided (2026-08-11) from the dual-corpus
-- shadow ablation: the invented station's widest anchor measured 16,
-- the thinnest real anchor 19, and every threshold in 17..19 kills
-- all measured coincidences while keeping every measured real site
-- (contracts/eval/commit-ablation*.json; the aggregate form fails —
-- 7+16=23 would re-admit the invention).
anchorFloor :: Int
anchorFloor = 19

-- | What a DECLARATION key is worth toward opening a cross site. A
-- key that vanished from one pair and appeared in exactly one other
-- is the provenance identity a cross site must buy — it is the very
-- removed-definition scan the ground-truth reviewers ran by hand —
-- so it pays the site cost outright. Set this to 0 and declFloor
-- collapses onto destFloor: the knob is as ablatable as the other
-- four and cannot hide.
declCredit :: Int
declCredit = siteCostCross

-- | Minimal shared leftover content that opens a DECLARATION-anchored
-- cross site (derived, exactly as destFloor is): the least n whose
-- site still profits once the key has paid. Evaluates to 1 — one
-- shared line, which ALONE is the coincidence tie at the line level
-- (1*1 + 2 = 3 = 1*3), opens once the key carries the identity.
-- n = 0 stays shut: a declaration that kept its name but not one
-- line of its body is ADAPTED, and this stage says nothing about it
-- (docs/EVAL-SET.md, the adapted class).
declFloor :: Int
declFloor = go 0
 where
  go n = if siteOpens (siteCostCross - declCredit) n then n else go (n + 1)
