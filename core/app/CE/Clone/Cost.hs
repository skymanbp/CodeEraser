-- | The T3 knobs — every constant the clone judgment reads, nothing
-- else (the CE.Graph.Cost posture: ablation targets live in ONE
-- module, so a dead knob has nowhere to hide).
module CE.Clone.Cost (tsedNum, tsedDen, unitNodeCap, pairCap) where

-- | The clone threshold as an integer ratio (0.85). Defined in-repo
-- and documented here (2026-08-13 decision ②): plan §4.1 gives the
-- number without a checkable source, so this repo owns the
-- definition — clone ⇔ (max − ted)·tsedDen ≥ tsedNum·max, integer
-- cross-multiplication, no floats anywhere in core (FourClass.Cost:
-- floats tie-break differently across platforms). The Rust candidate
-- prunes use the SAME 85/100 (candidates.rs) — that identity is what
-- makes them admissible.
tsedNum :: Integer
tsedNum = 85

tsedDen :: Integer
tsedDen = 100

-- | Per-tree node ceiling. Sizing anchor (design §4.4, F5):
-- Zhang-Shasha is O(n1·n2·min(d,l)1·min(d,l)2); at 256 nodes with a
-- typical min(depth, leaves) ≈ 16 one pair costs ≈ 256·256·16·16 ≈
-- 1.7×10⁷ strict map updates — bounded work per pair, decided before
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
