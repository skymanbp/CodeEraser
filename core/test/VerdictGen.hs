-- | The seeded inputs behind VerdictEquivProps (plan v2.32 step 7):
-- two hundred fact sets under perturbed score knobs, two hundred
-- ratchet cases, two hundred LOC distributions, two hundred wire
-- requests, and the exhaustive leg family of the join lattice. The
-- LCG is ReferenceFlowGen's; nothing here judges.
module VerdictGen (WireCase, factCases, joinFamily, locCases, ratchetCases, wireCases) where

import CE.Verdict.Join (Legs (..), Pos (..))
import CE.Verdict.Score (ScoreKnobs (..), scoreBound)
import Data.List (sort)
import ReferenceFlowGen (G, S (..), rand, runG)
import ReferenceScore (Inputs (..))

seeded :: Int -> G a -> [a]
seeded salt g = [runG g (S (n * 9973 + salt) 0 0 0) | n <- [1 .. 200]]

int :: Int -> G Integer
int n = toInteger <$> rand n

listOf :: Int -> G a -> G [a]
listOf n g = rand n >>= \k -> mapM (const g) [1 .. k]

-- | Ascending distinct members of 0..n-1, each kept one time in two.
subset :: Integer -> G [Integer]
subset n = concat <$> mapM (\u -> (\r -> [u | r == 0]) <$> rand 2) [0 .. n - 1]

-- | Fact sets over a universe of up to eight files, under knobs moved
-- off their defaults one time in two, with a soft line or none.
factCases :: [(ScoreKnobs, Maybe Integer, Inputs)]
factCases = seeded 11 $ do
  n <- (+ 1) <$> int 8
  (sim, pos, churn) <- graphTables n
  cont <- classedRows 18 900
  docs <- subset n
  loops <- subset n
  classes <- classLines 500
  k <- knobs
  soft <- (\r s -> if r == 0 then Nothing else Just s) <$> rand 3 <*> ((+ 150) <$> int 400)
  pure (k, soft, Inputs sim pos churn cont docs classes loops)

-- | The graph-side tables over n file nodes: candidate pairs u < v
-- around both similarity bars, positions, churn.
graphTables :: Integer -> G ([[Integer]], [[Integer]], [[Integer]])
graphTables n = do
  pairs <- filter (\p -> p `div` n < p `mod` n) <$> subset (n * n)
  sim <- mapM (\p -> (\kind num -> [p `div` n, p `mod` n, kind, num, 100]) <$> int 3 <*> ((+ 70) <$> int 31)) pairs
  pos <- subset n >>= mapM (\u -> (\i s r -> [u, i, 0, u, s, r]) <$> int 3 <*> ((+ 1) <$> int 3) <*> int 2)
  churn <- subset n >>= mapM (\u -> (\rw ap -> [u, rw, ap]) <$> int 4 <*> int 4)
  pure (sim, pos, churn)

-- | Classed continuous rows [entity, metric, value, class] on
-- ascending distinct (entity, metric) keys.
classedRows :: Integer -> Int -> G [[Integer]]
classedRows keys top = subset keys >>= mapM (\key -> (\v cls -> [key `div` 2, key `mod` 2, v, cls]) <$> int top <*> int 3)

-- | Every line (codes 0..2) of classes 1 and 2.
classLines :: Int -> G [[Integer]]
classLines top = concat <$> mapM (\c -> mapM (\code -> (\v -> [c, code, v]) <$> ((+ 1) <$> int top)) [0, 1, 2]) [1, 2]

knobs :: G ScoreKnobs
knobs = do
  moved <- (== 0) <$> rand 2
  cyc <- (+ 1) <$> int 3
  ceil' <- (+ 5) <$> int 30
  dead <- int 2
  cost <- (+ 5) <$> int 20
  pure (if moved then scoreBound {sCycleFloor = cyc, sCocCeil = ceil', sDeadIndegCeil = dead, sViolCost = cost} else scoreBound)

-- | LOC multisets of zero to twenty values, zeros included.
locCases :: [[Integer]]
locCases = seeded 23 (listOf 21 (int 1200))

-- | Baselines and current facts over six entities and two metrics,
-- with class allowances, a present set or none, and establish runs.
ratchetCases :: [([[Integer]], Maybe [Integer], Maybe ([[Integer]], [Integer]), [[Integer]], [Integer])]
ratchetCases = seeded 37 $ do
  keys <- subset 12
  base <- mapM (\key -> (\v -> [key `div` 2, key `mod` 2, v]) <$> int 700) keys
  cont <- classedRows 12 760
  classes <- concat <$> mapM (\c -> concat <$> mapM (\code -> (\r v -> [[c, code, v] | r == 0]) <$> rand 2 <*> int 30) [3, 4]) [1, 2]
  present <- (\r ps -> if r == 0 then Nothing else Just ps) <$> rand 2 <*> subset 6
  baseDisc <- subset 10
  disc <- subset 10
  established <- (/= 0) <$> rand 5
  pure (classes, present, if established then Just (base, baseDisc) else Nothing, cont, disc)

-- | verdict/1 requests over two to eight file nodes: sim pairs,
-- positions, churn, classed continuous rows with class lines, judged
-- LOC, doc files and a weight table.
-- | sim, pos, churn, continuous, class lines, judged LOC, doc files,
-- weights, the node count.
type WireCase = ([[Integer]], [[Integer]], [[Integer]], [[Integer]], [[Integer]], [Integer], [Integer], [[Integer]], Integer)

wireCases :: [WireCase]
wireCases = seeded 41 $ do
  n <- (+ 2) <$> int 7
  (sim, pos, churn) <- graphTables n
  cont <- classedRows 14 900
  classes <- classLines 600
  locs <- sort <$> listOf 9 (int 900)
  docs <- subset n
  weights <- subset 7 >>= mapM (\c -> (\w -> [c, w]) <$> ((+ 1) <$> int 3))
  pure (sim, pos, churn, cont, classes, locs, docs, weights, n)

-- | Every leg combination the lattice distinguishes: the similarity
-- kind on both sides of its bar, each graph side absent or one of
-- sixteen positions, three churn shapes per side, cochange absent,
-- under or at its floor.
joinFamily :: [Legs]
joinFamily =
  [ Legs (kind, num, 100) a b ca cb coch
  | (kind, num) <- [(0, 85), (0, 84), (1, 90), (2, 80), (2, 79)]
  , a <- sides
  , b <- sides
  , ca <- churns
  , cb <- churns
  , coch <- [Nothing, Just 1, Just 2]
  ]
 where
  sides = Nothing : [Just (Pos i r f s) | i <- [0, 1], r <- [0, 1], f <- [0, 1, 2, 4], s <- [0, 1]]
  churns = [(0, 0), (3, 1), (1, 3)]
