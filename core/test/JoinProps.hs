-- | The join lattice's fixed-seed property battery (M5-3h; the wire
-- hookup landed with 3i, and the sim leg speaks the wire vocabulary:
-- (simKind, num, den) against the owning family's threshold).
-- Non-vacuity first (the F16 stance): the population must produce
-- every verdict class, else the battery asserts over nothing —
-- deterministic fixtures guarantee the seats and every knob probe
-- owns a boundary fixture that must flip. The RG10 counterfactual
-- is 同形: the SAME row with only the dead flank's exported bit
-- flipped must abandon delete_candidate and raise the guard bit.
module JoinProps (battery) where

import CE.Verdict.Join
import Data.Bits (testBit, xor, (.&.), (.|.))

battery :: IO Bool
battery = do
  a <- check "non-vacuous: all four verdict classes appear" allClasses
  b <- check "RG10 counterfactual: a public dead flank never deletes" rg10
  c <- check "legsMask honest: gated => 3 legs; graph-absent never gates" legsHonest
  d <- check "dead knobs all live (clone/dup thresholds, cochange, rewrite, entryMask)" deadKnobs
  e <- check "priority: merge outranks hotspot when both hold" priority
  f <- check "reason bit 0 never fires (deliberately absent)" bitZeroSilent
  g <- check "reason ledger pinned on the delete and guard fixtures" ledgerPinned
  pure (and [a, b, c, d, e, f, g])

check :: String -> Bool -> IO Bool
check name ok = do
  putStrLn ((if ok then "ok   " else "FAIL ") <> name)
  pure ok

verdictOf :: Legs -> Integer
verdictOf l = let (v, _, _) = judge bound l in v

reasonsOf :: Legs -> Integer
reasonsOf l = let (_, _, r) = judge bound l in r

-- deterministic anchors: one per class, plus every knob's boundary
alive :: Integer -> Integer -> Maybe Pos
alive n scc = Just (Pos {pIndeg = n, pReach = 1, pFlags = 0, pScc = scc})

deadPos :: Integer -> Maybe Pos
deadPos flags = Just (Pos {pIndeg = 0, pReach = 0, pFlags = flags, pScc = 0})

-- | sim sits EXACTLY on the clone threshold (85/100) — the
-- kCloneNum/kCloneDen probes' boundary.
mergeRow :: Legs
mergeRow = Legs (1, kCloneNum bound, kCloneDen bound) (alive 1 0) (alive 2 1) (0, 0) (0, 0) Nothing

deleteRow :: Legs
deleteRow = Legs (1, 90, 100) (deadPos 0) (alive 1 1) (0, 0) (0, 0) Nothing

-- | Same shape as deleteRow with the dead flank public (bit 0).
publicGuardRow :: Legs
publicGuardRow = Legs (1, 90, 100) (deadPos 1) (alive 1 1) (0, 0) (0, 0) Nothing

-- | Dead flank carries the test entry bit (2): entry under the bound
-- mask, dead once the entryMask probe strips that bit.
entryFlankRow :: Legs
entryFlankRow = Legs (1, 90, 100) (deadPos 2) (alive 1 1) (0, 0) (0, 0) Nothing

-- | Same-SCC referenced pair (merge cannot fire), docdup-kind sim
-- EXACTLY on 80/100 (the kDupNum/kDupDen probes' boundary), rewrite
-- share EXACTLY on 50/100 (the rewrite probes'), cochange argument
-- the cochangeFloor probe's.
hotspotRow :: Maybe Integer -> Legs
hotspotRow c = Legs (2, kDupNum bound, kDupDen bound) (alive 1 0) (alive 1 0) (25, 25) (25, 25) c

-- | Satisfies merge AND hotspot conditions at once.
bothRow :: Legs
bothRow = Legs (1, 90, 100) (alive 1 0) (alive 1 1) (25, 25) (25, 25) (Just 3)

-- | Unit tier: one graph leg honestly missing.
tierURow :: Legs
tierURow = Legs (1, 90, 100) Nothing (alive 1 0) (5, 5) (5, 5) (Just 3)

fixtures :: [Legs]
fixtures =
  [ mergeRow
  , deleteRow
  , publicGuardRow
  , entryFlankRow
  , hotspotRow (Just (kCochangeFloor bound))
  , hotspotRow (Just (kCochangeFloor bound + 1))
  , bothRow
  , tierURow
  ]

pop :: [Legs]
pop = fixtures <> map rowAt [1 .. 400]

-- | Seeded rows (no RNG in a deterministic contract): sim drawn
-- boundary-heavy across both families, graph legs sometimes absent,
-- flags drawn from {0, public, test-entry, dyn-entry}.
rowAt :: Int -> Legs
rowAt i =
  Legs
    { lSim =
        [(1, 84, 100), (1, 85, 100), (2, 80, 100), (2, 79, 100), (0, 100, 100)]
          !! (s 1 `mod` 5)
    , lGraphA = posAt 2
    , lGraphB = posAt 3
    , lChurnA = (toInteger (s 4 `mod` 40), toInteger (s 5 `mod` 40))
    , lChurnB = (toInteger (s 6 `mod` 40), toInteger (s 7 `mod` 40))
    , lCochange = case s 8 `mod` 4 of
        0 -> Nothing
        n -> Just (toInteger n)
    }
 where
  s :: Int -> Int
  s n = fromInteger (lcg (toInteger i * 104729 + toInteger n) `mod` 1000)
  posAt n = case s n `mod` 5 of
    0 -> Nothing
    _ ->
      Just
        ( Pos
            { pIndeg = toInteger (s (n * 11) `mod` 3)
            , pReach = toInteger (s (n * 13) `mod` 2)
            , pFlags = [0, 1, 2, 4] !! (s (n * 17) `mod` 4)
            , pScc = toInteger (s (n * 19) `mod` 3)
            }
        )

lcg :: Integer -> Integer
lcg s = (s * 6364136223846793005 + 1442695040888963407) `mod` 18446744073709551616

census :: Knobs -> [Int]
census k = [length [() | l <- pop, let (v, _, _) = judge k l, v == c] | c <- [0 .. 3]]

allClasses :: Bool
allClasses = all (> 0) (census bound)

-- | Every delete in the population, republished with its dead flank
-- public, must leave class 2 and raise bit 6 — and there must BE
-- deletes to flip (deleteRow guarantees the seat).
rg10 :: Bool
rg10 = not (null dels) && all blocked dels
 where
  dels = [l | l <- pop, verdictOf l == 2]
  blocked l = let l' = publicize l in verdictOf l' /= 2 && testBit (reasonsOf l') 6
  -- the test's own flank selector — independent of the lattice's
  publicize l = case (lGraphA l, lGraphB l) of
    (Just a, Just b)
      | deadish a b -> l {lGraphA = Just a {pFlags = pFlags a .|. 1}}
      | deadish b a -> l {lGraphB = Just b {pFlags = pFlags b .|. 1}}
    _ -> l
  deadish x y =
    pIndeg x == 0 && pReach x == 0 && (pFlags x .&. kEntryMask bound) == 0 && pIndeg y >= 1

legsHonest :: Bool
legsHonest = all ok pop
 where
  ok l =
    let (v, m, _) = judge bound l
        absent = case (lGraphA l, lGraphB l) of
          (Just _, Just _) -> False
          _ -> True
     in (v == 0 || m == legSim + legGraph + legChurn)
          && (not absent || (v == 0 && m == legSim + legChurn))

deadKnobs :: Bool
deadKnobs =
  all
    (/= base)
    [ census bound {kCloneNum = kCloneNum bound + 1}
    , census bound {kCloneDen = kCloneDen bound - 1}
    , census bound {kDupNum = kDupNum bound + 1}
    , census bound {kDupDen = kDupDen bound - 1}
    , census bound {kCochangeFloor = kCochangeFloor bound + 1}
    , census bound {kRewriteNum = kRewriteNum bound + 1}
    , census bound {kRewriteDen = kRewriteDen bound - 1}
    , census bound {kEntryMask = kEntryMask bound `xor` 2}
    ]
 where
  base = census bound

priority :: Bool
priority = verdictOf bothRow == 1

bitZeroSilent :: Bool
bitZeroSilent = all (\l -> not (testBit (reasonsOf l) 0)) pop

-- | Exact reason integers pin the bit layout as a contract: delete =
-- bits {1,2,4,5} = 54; guard = bits {1,2,4,5,6} = 118 (deadFlank
-- stays set — publicness blocks the verdict, not the observation).
ledgerPinned :: Bool
ledgerPinned = reasonsOf deleteRow == 54 && reasonsOf publicGuardRow == 118
