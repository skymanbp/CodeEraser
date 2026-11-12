-- | The verdict family's PURE battery (design §7.4, M5-3i; the
-- wire-driven half — ablation, idempotence, refusals, knob-table
-- probes — lives in VerdictWireProps since the ADR-008 P4 split at the
-- 300-line law): the ratchet's tolerance inclusion and the soft
-- line's derivation, then the extent law's battery (VerdictExtentProps) and its
-- contract refusals (VerdictExtentRefusals). The density law's
-- fixture and its knob probes retired with that law (10.0.0); the
-- zone curve went with the split advisory, its one reader (SplitZoneProps).
module VerdictProps (battery) where

import qualified CE.Verdict.Ratchet as R
import CE.Verdict.Soft (softLine)
import qualified Data.Set as S
import VerdictExtentProps (extentLegs)
import VerdictExtentRefusals (refusalLegs)
import WireHarness (runChecks)

battery :: IO Bool
battery = runChecks (softZone <> extentLegs <> refusalLegs)
 where
  softZone =
    [ ("tolerance-over sets shrink as tolAbs grows (inclusion)", overMono)
    , ("the soft line derives by order statistics and clamps both ends", softDerive)
    ]

-- | Monotonicity as SET INCLUSION (never count comparison): growing
-- the absolute tolerance leg can only shrink the over set, checked
-- on (entity, metric) identities across seeded fact tables.
overMono :: Bool
overMono = all one [1 .. 60 :: Integer]
 where
  one i =
    let cont = [[u, 0, 100 + ((i * (u + 1) * 7) `mod` 40)] | u <- [0 .. 9]]
        base = R.Baseline [[u, 0, 100] | u <- [0 .. 9]] [] Nothing Nothing
        -- no class declares an allowance here, so every row keeps
        -- the global legs (5.1.0 signature, same judgment)
        noClassTol _ _ = Nothing
        ids k = S.fromList [(u, c) | [u, c, _, _] <- R.rOver (R.ratchet k noClassTol Nothing (Just base) cont [])]
        wide = R.ratchetBound {R.rTolAbs = R.rTolAbs R.ratchetBound + 15}
     in ids wide `S.isSubsetOf` ids R.ratchetBound

-- | Hand-computed derivations (plan v2.6 §B, the log-free form):
-- {240,300,375} → m=300, r=5/4, k=2 → floor(300·25/16)=468;
-- {100,300} even count → m²=100·300, r⁴=3·3 → ⌊(9·10⁸·81)^¼⌋=519
-- → clamp 500 (the arithmetic middles gave 612); all-100 → r=1 →
-- 100 → clamp 200; k=1 halves the spread's bite; zeros are dropped
-- and an empty multiset derives NOTHING. The upper fence is min
-- softMax H at establish (9.0.0): under H = 400 the same {100,300}
-- reads 400, and an H below softMin wins over the lower fence too.
softDerive :: Bool
softDerive =
  and
    [ softLine 2 200 500 [240, 300, 375] == Just 468
    , softLine 2 200 500 [100, 300] == Just 500
    , softLine 2 200 (min 500 400) [100, 300] == Just 400
    , softLine 2 200 (min 500 150) [240, 300, 375] == Just 150
    , softLine 2 200 500 [100, 100, 100] == Just 200
    , softLine 1 200 500 [240, 300, 375] == Just 375
    , softLine 2 200 500 [0, 0] == Nothing
    , softLine 2 200 500 [] == Nothing
    ]
