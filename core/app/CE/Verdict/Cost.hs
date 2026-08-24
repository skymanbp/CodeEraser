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
  , sizeHard
  , sizePMax
  , softLineK
  , softMin
  , softMax
  , cocCeil
  , deadIndegCeil
  , violCost
  , violCostNeutral
  , zoneWarnPermille
  , zoneAskPermille
  , defaultWeight
  , scoreScale
  , verdictNodeCap
  , verdictRowCap
  , classCap
  ) where

-- | The rulepack fence (plan v2.13 ①, 3.1.0): a class id on a
-- continuous row or in the classKnobs table is bounded below this.
-- A fence, not a quota — the softKMax stance.
classCap :: Integer
classCap = 64

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

-- | Size-axis soft-line FALLBACK: when the baseline carries no
-- softLine (pre-v0.6 file, or no judgedLoc to derive from), the
-- graded zone opens here (plan §4.1's 300; plan v2.6 §A). Before
-- v0.6 this was the binary violation ceiling.
sizeCeil :: Integer
sizeCeil = 300

-- | The hard line H (plan §4.1's 750), reaching the core for the
-- first time in v0.6: the convex zone denominator (H − S). The
-- deny semantics stay Rust-side (scan fail tier, guard budget);
-- here it only scales the curve.
sizeHard :: Integer
sizeHard = 750

-- | P_max: the axis-0 penalty of a file AT the hard line, in the
-- same violation units the other axes count in — one hard-line
-- file weighs like ten old binary violations. Past H the curve
-- continues LINEARLY at the slope it reached (C¹, monotone, no
-- kink — Soft.zonePenalty): P_max is the at-the-line value, not a
-- cap, but the quadratic never leaves its contracted (S,H] domain
-- (the M9 batch-6 saturation lesson).
sizePMax :: Integer
sizePMax = 10

-- | k: the multiplicative-MAD exponent in the soft-line derivation
-- S = clamp(median·r^k, [softMin, softMax]) (plan v2.6 §B).
-- Calibrated over self + requests + ripgrep (v0.6 P2): k=2 lands S
-- within ±6% of the historical 300 on this repo's judged set.
softLineK :: Integer
softLineK = 2

-- | The §B clamp fence: however the repo's distribution leans, the
-- soft line stays inside [200, 500]. Structural constants, not
-- knobs — the fence is what makes the relative line safe to trust.
softMin :: Integer
softMin = 200

softMax :: Integer
softMax = 500

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

-- | The global strictness dial in the score fold:
-- score = scale - sum(w_i * charge_i * violCost) / (violCostNeutral
-- * wTotal), floored at 0. charge_i is the axis's bounded per-mille
-- density (Score.charge), so at the neutral default the score is
-- the plain weighted mean of the axis charges and can never
-- saturate; a repo declaring viol_cost above neutral asks for
-- harsher scores by choice. wTotal is DERIVED from the effective
-- weights, never a literal (the destFloor convention — a hand-typed
-- total is how a weight silently dies).
violCost :: Integer
violCost = 10

-- | The graded-zone tier cut points, in permille of the (S, H]
-- span (plan v2.7 (1), batch-7 slice 5): below warn = observe,
-- [warn, ask] = warn, past ask = ask. They reach the PreToolUse
-- hook through the committed baseline document (written beside
-- softLine at establish — the hook is daemon-free by design, so a
-- local file read is the honest transport); the map lived as bare
-- Rust literals with no knob and no wire field before. The default
-- STAYS observe-only until `[guard] zone_tiers` opts in — these
-- numbers decide tiers only after the FPR discipline armed them.
zoneWarnPermille :: Integer
zoneWarnPermille = 250

zoneAskPermille :: Integer
zoneAskPermille = 750

-- | The value of violCost at which the dial is a no-op — the fixed
-- denominator that makes the default score exactly the weighted
-- mean of the axis charges. A structural constant, deliberately NOT
-- a knob: two dials on one ratio is one dial and a trap.
violCostNeutral :: Integer
violCostNeutral = 10

-- | Weight of an axis the request's weights table does not name.
-- Equal weights are the decided opening stance (decision ⑦); the
-- wire can override per axis, and wTotal follows.
defaultWeight :: Integer
defaultWeight = 1

-- | The score's opening value (per-mille scale) — the last scoring
-- number that lived as an inline literal in Score.score, outside
-- every perturbation battery's reach (ADR-008 survey gap 1). The
-- floor stays a structural `max 0`, not a knob: a negative score
-- has no meaning at any scale.
scoreScale :: Integer
scoreScale = 1000

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
