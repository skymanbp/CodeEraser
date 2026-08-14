-- | Verdict-family constants — nothing but constants (the
-- CE.Graph.Cost convention), so the JoinProps/VerdictProps dead-knob
-- batteries and the ablation table have exactly one target. Knobs
-- are consumed as PARAMETERS by CE.Verdict.{Join,Score,Ratchet} and
-- bound to these constants only at the family boundary, which is
-- what lets the batteries perturb each one and watch a census move
-- without touching production code. Similarity thresholds are NOT
-- here: the clone axis reuses CE.Clone.Cost (85/100) and the docdup
-- axis CE.Docdup.Cost (80/100) — one authority per family, the
-- entryMask precedent.
--
-- Integer per the 2026-08-12 blocking decision ③ (the Anchor.hs
-- overflow lesson generalized: guards stay out of bounded
-- arithmetic).
--
-- reasonBits — the ledger of which conditions HELD, travelling with
-- every verdict so a two-leg firing can never hide (design §6.3).
-- One comment per bit:
--
--   bit 0 — deliberately ABSENT (the entryMask bit-0 body style):
--           exported-ness never argues FOR a verdict. It is the
--           public/private judgment axis (RG10), only ever a guard,
--           so this bit stays 0 in every reply.
--   bit 1 — simOver: the pair's own family threshold cleared —
--           kind 0/1 (t1t2/t3) against CE.Clone.Cost's 85/100,
--           kind 2 (docdup) against CE.Docdup.Cost's 80/100,
--           num/den cross-multiplied (no division).
--   bit 2 — graphBoth: BOTH sides answered a graph position row —
--           the Tier F discriminator; without it nothing gates.
--   bit 3 — bothReferenced: indeg >= 1 on both sides (merge axis).
--   bit 4 — sccDistinct: the sides sit in different SCCs; merging
--           inside one cycle is dead-code work, not merge work.
--   bit 5 — deadFlank: one side has indeg 0, reachIn 0 and no entry
--           bit while its partner keeps indeg >= 1 (delete axis —
--           partner-alive is part of THIS bit's definition). On the
--           verdict/1 wire flags are structurally 0 and entry-ness
--           is IMPLIED by reachIn: an entry node is in its own reach
--           set, so reachIn 0 already excludes entries.
--   bit 6 — publicGuard: a structurally delete-ready flank was
--           public, so RG10 blocked the delete. File-granularity
--           wire flags carry no exported bit today (symbol facts
--           are R6), so on the wire this guard is dormant — the
--           lattice and its battery keep it live.
--   bit 7 — cochangeHot: co-change count >= cochangeFloor.
--   bit 8 — rewriteHot: the window's rewrite share clears
--           rewriteNum/rewriteDen (cross-multiplied).
module CE.Verdict.Cost
  ( cochangeFloor
  , rewriteNum
  , rewriteDen
  , tolNum
  , tolDen
  , tolAbs
  , sizeCeil
  , cocCeil
  , deadIndegCeil
  , violCost
  , defaultWeight
  , verdictNodeCap
  , verdictRowCap
  ) where

-- | Co-change count that counts as entangled — the churn report's
-- own table floor (pairs enter it at count >= 2), so the lattice
-- never claims heat the report would not even list.
cochangeFloor :: Integer
cochangeFloor = 2

-- | Rewrite share threshold, numerator/denominator: at least half
-- the window's added lines on the entity landed inside EXISTING
-- units. A rewrite-heavy similar pair is being maintained twice —
-- the churn_hotspot claim; the same ratio is the churn AXIS
-- violation predicate (one authority for "rewrite-heavy").
rewriteNum :: Integer
rewriteNum = 50

rewriteDen :: Integer
rewriteDen = 100

-- | ADR-006 continuous-ratchet tolerance: a ceiling may be exceeded
-- by max(+2%, +10) in one edit — tolNum/tolDen is the 2% leg,
-- tolAbs the +10 leg, the max taken in Ratchet.tolerated. The legs
-- cross at ceiling 500 (Spec.costModel pins one assertion per leg).
tolNum :: Integer
tolNum = 102

tolDen :: Integer
tolDen = 100

tolAbs :: Integer
tolAbs = 10

-- | Size-axis violation ceiling: file lines over this count as a
-- violation (metricCode 0). The dogfood number (plan §4.1) — the
-- same 300 the scan gate warns at.
sizeCeil :: Integer
sizeCeil = 300

-- | Complexity-axis violation ceiling: cognitive complexity over
-- this counts (metricCode 1) — plan §4.1's CoC 15.
cocCeil :: Integer
cocCeil = 15

-- | Deadcode-axis violation shape: indeg <= this AND reachIn 0.
-- 0 = strictly unreferenced. The wire carries no entry flags (they
-- were consumed by the graph family's own verdicts; reachIn already
-- excludes entries), so this counts structurally orphaned files.
deadIndegCeil :: Integer
deadIndegCeil = 0

-- | Per-mille cost of one weighted violation in the score fold:
-- score = 1000 - sum(w_i * p_i * violCost) / wTotal, floored at 0.
-- wTotal is DERIVED from the effective weights, never a literal
-- (the destFloor convention — a hand-typed total is how a weight
-- silently dies).
violCost :: Integer
violCost = 10

-- | Weight of an axis the request's weights table does not name.
-- Equal weights are the decided opening stance (decision ⑦); the
-- wire can override per axis, and wTotal follows.
defaultWeight :: Integer
defaultWeight = 1

-- | Real oversize protection for verdict requests (the envelope
-- byte precheck is relaxed for the trusted same-machine child).
-- Nodes are file-tier entities: the graph family's cap magnitude
-- carries over; rows are the sum of every fact table's length.
-- Over cap => degraded verdict_too_large, never a truncated
-- judgment.
verdictNodeCap :: Integer
verdictNodeCap = 131072

verdictRowCap :: Integer
verdictRowCap = 524288
