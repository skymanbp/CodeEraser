-- | Three-signal join verdict constants — nothing but constants (the
-- CE.Graph.Cost convention), so the JoinProps dead-knob battery and
-- the 3i ablation table have exactly one target. The knobs are
-- consumed as PARAMETERS by CE.Verdict.Join and bound to these
-- constants only at the family boundary (Join.bound), which is what
-- lets the battery perturb each one and watch the verdict census
-- move without touching production code.
--
-- Integer per the 2026-08-12 blocking decision ③ (the Anchor.hs
-- overflow lesson generalized: guards stay out of bounded arithmetic
-- even while today's only use is a comparison).
--
-- reasonBits — the ledger of which conditions HELD, travelling with
-- every verdict so a two-leg firing can never hide (design §6.3).
-- One comment per bit; the verdict code is the lattice's conclusion,
-- the bits are its working:
--
--   bit 0 — deliberately ABSENT (the entryMask bit-0 body style):
--           exported-ness never argues FOR a verdict. It is the
--           public/private judgment axis (RG10), only ever a guard,
--           so this bit stays 0 in every reply.
--   bit 1 — simOver: verified similarity tokens >= simFloor.
--   bit 2 — graphBoth: BOTH sides answered a graph position row —
--           the Tier F discriminator; without it nothing gates.
--   bit 3 — bothReferenced: indeg >= 1 on both sides (merge axis).
--   bit 4 — sccDistinct: the sides sit in different SCCs; merging
--           inside one cycle is dead-code work, not merge work.
--   bit 5 — deadFlank: one side has indeg 0, reachIn 0 and no entry
--           bit while its partner keeps indeg >= 1 (delete axis —
--           partner-alive is part of THIS bit's definition, the
--           "伙伴 survive 更高" clause).
--   bit 6 — publicGuard: a structurally delete-ready flank was
--           public, so RG10 blocked the delete. Fires beside
--           report_only, or beside a delete carried by the OTHER
--           flank — the ledger records both.
--   bit 7 — cochangeHot: co-change count >= cochangeFloor.
--   bit 8 — rewriteHot: the window's rewrite share clears
--           rewriteNum/rewriteDen (cross-multiplied, no division —
--           the Docdup 80/100 boundary style).
module CE.Verdict.Cost (simFloor, cochangeFloor, rewriteNum, rewriteDen) where

-- | Similarity tokens that make a pair worth judging — mirrors the
-- winnowing guarantee t = 50 (the report threshold; blocks below t
-- are opportunistic detections, not guaranteed ones, so they do not
-- gate).
simFloor :: Integer
simFloor = 50

-- | Co-change count that counts as entangled — the churn report's
-- own table floor (pairs enter it at count >= 2), so the lattice
-- never claims heat the report would not even list.
cochangeFloor :: Integer
cochangeFloor = 2

-- | Rewrite share threshold, numerator/denominator: at least half
-- the window's added lines on the pair landed inside EXISTING units.
-- A rewrite-heavy similar pair is being maintained twice — the
-- churn_hotspot claim.
rewriteNum :: Integer
rewriteNum = 50

rewriteDen :: Integer
rewriteDen = 100
