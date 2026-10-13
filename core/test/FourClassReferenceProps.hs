-- | The four-class family's generated battery for the two stages the
-- exhaustive family (Reference) does not send: stacking over the
-- duplicated spans (7.0.0, O47) and the declaration stage (7.1.0,
-- O48). Three hundred seeded batches of two to four pairs — removed
-- and added runs over a small hash alphabet (no pair's added hash among
-- its own removed ones, the within-first contract), duplicated spans
-- now and then over a long addition, declaration rows on every pair or
-- none — through the shipped `classify` and through this module's own
-- reading of booklet 09: stacking fires when at least twenty novel
-- lines lie inside a duplicated span and the deletions times ten stay
-- under the novel lines; a key that vanished from one pair and appears
-- in exactly one OTHER pair is an edge when the two spans' leftover
-- content shares at least one distinct hash; two destinations refuse
-- the key. The constants are this module's data, read off the booklet;
-- the novel and deleted lines are Reference's set-form phases.
module FourClassReferenceProps (battery) where

import CE.FourClass.Provenance (classify)
import CE.FourClass.Wire (Pair (..), Request (..), Result (..))
import Data.Aeson (Value (Null))
import qualified Data.Set as S
import Data.Word (Word64)
import Reference (refBlocks, refMarks)
import ReferenceFlowGen (G, S (..), rand, runG)
import WireHarness (generated, runChecks)

-- | Booklet 09's numbers: the stacking floor and ratio, the
-- declaration floor (one shared line opens an edge, zero does not).
novelFloor, removalRatio, sharedFloor :: Int
novelFloor = 20
removalRatio = 10
sharedFloor = 1

battery :: IO Bool
battery = runChecks (generated names agrees reached shapes batches)
 where
  names = ("three hundred generated batches agree with booklet 09 on stacking and declaration edges", "the generated batches reach every stacking and declaration shape")

batches :: [[Pair]]
batches = [runG batch (S (n * 6151 + 29) 0 0 0) | n <- [1 .. 300 :: Int]]

pick :: [a] -> G a
pick xs = (xs !!) <$> rand (length xs)

batch :: G [Pair]
batch = do
  n <- (2 +) <$> rand 3
  declared <- (/= 0) <$> rand 4
  mapM (pairAt declared) [0 .. n - 1]

pairAt :: Bool -> Int -> G Pair
pairAt declared i = do
  long <- (== 0) <$> rand 3
  remN <- if long then rand 4 else rand 9
  remHs <- mapM (const (pick [1 .. 12 :: Word64])) [1 .. remN]
  addN <- if long then (20 +) <$> rand 12 else rand 9
  addHs <- mapM (const (pick [h | h <- [1 .. 14], h `notElem` remHs])) [1 .. addN]
  spanOn <- (/= 0) <$> rand 3
  s <- (1 +) <$> rand 3
  e <- (s +) <$> pick [12, 18, 19, 20, 24, 30]
  dRem <- declRows (max 1 remN)
  dAdd <- declRows (max 1 addN)
  pure
    Pair
      { pIdx = i
      , pRem = [lineRun 1 remHs | remN > 0]
      , pAdd = [lineRun 1 addHs | addN > 0]
      , pDupSpans = [(77, s, e) | spanOn, long]
      , pDeclRem = if declared then Just dRem else Nothing
      , pDeclAdd = if declared then Just dAdd else Nothing
      }
 where
  lineRun from hs = [(l, h, 30) | (l, h) <- zip [from ..] hs]

-- | Zero to two declaration rows of distinct keys over 1..top.
declRows :: Int -> G [(Word64, Int, Int, Int)]
declRows top = do
  k <- rand 3
  keys <- mapM (const ((,) <$> pick [100, 101, 102] <*> pick [1, 2])) [1 .. k]
  rows <- mapM (\(h, kind) -> (\a b -> (h, kind, min a b, max a b)) <$> ((1 +) <$> rand top) <*> ((1 +) <$> rand top)) keys
  pure [r | (j, r@(h, kind, _, _)) <- zip [0 :: Int ..] rows, (h, kind) `notElem` [(h', k') | (h', k', _, _) <- take j rows]]

-- | One pair's stacking signals: novel lines, deleted lines, novel
-- lines inside a duplicated span — the leftovers Reference's phases
-- did not claim.
data Signals = Signals {novel, deleted, inDup :: Int}

signals :: [Pair] -> [(Pair, Signals)]
signals ps = [(p, sig p) | p <- ps]
 where
  (outs, ins) = refMarks ps (refBlocks ps)
  left side marks p = [l | (l, _, _) <- concat (side p), (pIdx p, l) `S.notMember` marks]
  sig p =
    let fresh = left pAdd ins p
     in Signals (length fresh) (length (left pRem outs p)) (length [l | l <- fresh, or [s <= l && l <= e | (_, s, e) <- pDupSpans p]])

-- | What became of one vanished key: an edge to the one pair that
-- claims it, a claim by its own pair, a claim whose spans share less
-- than the floor, two or more claimants, or none.
data Fate = Edge Int | Own | Thin | Contested | Unclaimed deriving (Eq)

fates :: [Pair] -> [(Pair, Word64, Fate)]
fates ps = [(p, h, fate p h k s e) | p <- ps, (h, k, s, e) <- rows pDeclRem p]
 where
  rows side = maybe [] id . side
  fate p h k s e = case [(q, s', e') | q <- ps, (h', k', s', e') <- rows pDeclAdd q, (h', k') == (h, k)] of
    [] -> Unclaimed
    [(q, s', e')]
      | pIdx q == pIdx p -> Own
      | S.size (S.intersection (within pRem p s e) (within pAdd q s' e')) >= sharedFloor -> Edge (pIdx q)
      | otherwise -> Thin
    _ -> Contested
  within side p s e = S.fromList [h | (l, h, _) <- concat (side p), l >= s, l <= e]

stacks :: Signals -> Bool
stacks g = inDup g >= novelFloor && light g

light :: Signals -> Bool
light g = deleted g * removalRatio < novel g

-- | The reference's stacking rows and declaration edges.
expected :: [Pair] -> ([(Int, String)], Maybe ([(Int, Int, Word64)], Bool))
expected ps = ([(pIdx p, "stacking") | (p, g) <- signals ps, stacks g], table)
 where
  table
    | all ((== Nothing) . pDeclRem) ps = Nothing
    | otherwise = Just (S.toAscList (S.fromList [(pIdx p, q, h) | (p, h, Edge q) <- fates ps]), False)

agrees :: [Pair] -> Maybe String
agrees ps
  | got == want = Nothing
  | otherwise = Just (take 400 (show (got, want) <> " on " <> show [(pIdx p, pRem p, pAdd p, pDupSpans p, pDeclRem p, pDeclAdd p) | p <- ps]))
 where
  r = classify (Request Null ps)
  got = (resSuspicions r, resUnitEdges r)
  want = expected ps

reached :: [Pair] -> S.Set String
reached ps = S.fromList (concatMap stackShape (signals ps) <> declShapes)
 where
  stackShape (p, g) =
    ["stacking fires" | stacks g]
      <> ["stacking held by the removal ratio" | inDup g >= novelFloor, not (light g)]
      <> ["novel mass outside the duplicated spans is silent" | novel g >= novelFloor, inDup g < novelFloor, not (null (pDupSpans p)), light g]
  declShapes
    | all ((== Nothing) . pDeclRem) ps = ["a batch without declaration tables"]
    | otherwise = [name f | (_, _, f) <- fates ps]
  name f = case f of
    Edge _ -> "a declaration edge"
    Own -> "a key back in its own pair"
    Thin -> "a key whose spans share nothing"
    Contested -> "two destinations refuse a key"
    Unclaimed -> "a key nobody claims"

shapes :: [String]
shapes =
  [ "stacking fires"
  , "stacking held by the removal ratio"
  , "novel mass outside the duplicated spans is silent"
  , "a batch without declaration tables"
  , "a declaration edge"
  , "a key back in its own pair"
  , "a key whose spans share nothing"
  , "two destinations refuse a key"
  , "a key nobody claims"
  ]
