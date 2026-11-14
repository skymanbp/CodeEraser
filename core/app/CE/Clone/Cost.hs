-- | The T3 knobs — every constant the clone judgment reads, nothing
-- else (the CE.Graph.Cost posture: ablation targets live in ONE
-- module, so a dead knob has nowhere to hide). Every binding here is
-- exported, so the module states no export list.
module CE.Clone.Cost where

-- | The clone threshold as an integer ratio (0.85). Defined in-repo
-- and documented here (2026-08-13 decision ②): plan §4.1 gives the
-- number without a checkable source, so this repo owns the
-- definition — clone ⇔ (max − ted)·tsedDen ≥ tsedNum·max, integer
-- cross-multiplication, no floats anywhere in core (FourClass.Cost:
-- floats tie-break differently across platforms). The candidate
-- prunes (CE.Candidates.T3, through CE.Clone.Prefilter) read this same
-- 85/100 — that identity is what makes them admissible.
tsedNum :: Integer
tsedNum = 85

tsedDen :: Integer
tsedDen = 100

-- | Admission floor for a T3 unit: below this many AST nodes a
-- "clone" is a signature, not an implementation (batch-7 slice 10;
-- the ceilings unitNodeCap/pairCap made the trip at M5-3a, this
-- floor never did). The measuring side reads it from the definition
-- package (`limits`, `min_unit_nodes`) and sends only units at or
-- above it, so a sub-floor unit is never paired.
minUnitNodes :: Integer
minUnitNodes = 24

-- | Per-tree node ceiling. Sizing anchor (design §4.4, F5):
-- Zhang-Shasha is O(n1·n2·min(d,l)1·min(d,l)2); at 256 nodes with a
-- typical min(depth, leaves) ≈ 16 one pair costs ≈ 256·256·16·16 ≈
-- 1.7×10⁷ cell updates — bounded work per pair, decided before
-- any corpus measurement.
unitNodeCap :: Integer
unitNodeCap = 256

-- | Per-request pair ceiling, applied AFTER the two admissible
-- prunes. Over-cap answers a complete degraded reply, never a
-- truncated one; the 3e measured exit (self + ripgrep cold path
-- under budget, zod's 21,740 survivors as the pressure case) is the
-- number's backing — tightening it is a published trigger, not a
-- silent edit.
pairCap :: Integer
pairCap = 4096

-- | Per-request work ceiling (plan v2.35 C3): the sum of n1·n2 over a
-- request's pairs, beside pairCap. The node counts are facts the
-- measuring side holds, so it plans chunks under both ceilings; a
-- request over it answers like one over pairCap (complete, degraded).
-- One pair always fits: unitNodeCap² = 65,536 < workCap. Calibration
-- (counting on no bound, every pair a clone): the Zhang-Shasha CPU
-- seconds per unit of n1·n2 over the recorded clone.requests of zod,
-- jsoup, gson and fmt (26 requests, an in-process probe on this build,
-- 2026-11-11, AMD Ryzen 7 7700, the machine itself at 100 % CPU)
-- run 2.6e-7 to 7.3e-7 s; at the worst, 2 s is 2.7·10⁶ and the nearest
-- power of two is 2^21: 1.5 s of distance per request at the worst
-- slope, far inside the core link's 600 CPU-s budget (corelink/watch.rs).
workCap :: Integer
workCap = 2097152

-- | The clone verdict, stated by the threshold's OWNER (ADR-008 P1:
-- the reported set is the core's decision, riding each reply as a
-- per-row verdict bit).
cloneDecides :: Integer -> Integer -> Integer -> Bool
cloneDecides = cloneDecidesWith (tsedNum, tsedDen)

-- | The threshold-parameterized form (the Docdup dupDecidesWith
-- posture): ONE formula, so the battery's dead-knob and boundary
-- probes exercise the production comparison, never a
-- re-implementation.
cloneDecidesWith :: (Integer, Integer) -> Integer -> Integer -> Integer -> Bool
cloneDecidesWith (num, den) t n1 n2 = (mx - t) * den >= num * mx
 where
  mx = max n1 n2
