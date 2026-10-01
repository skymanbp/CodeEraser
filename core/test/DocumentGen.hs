-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The seeded document requests behind DocumentProps (plan v2.32
-- step 3): two hundred well-formed requests per family — the tables a
-- judgment could have answered, with their ranges, facts and ranks —
-- drawn by the flow reference's generator (ReferenceFlowGen's `G`), so
-- the battery reads every document the core assembles from inputs it
-- did not hand-pick.
module DocumentGen (docRequest, num, ranks, requests, some) where

import Control.Monad (filterM, forM, replicateM)
import Data.Aeson (Value, object, (.=))
import Data.Aeson.Key (fromString)
import Data.List (sortOn)
import ReferenceFlowGen (G, S (..), rand, runG)

-- | A request line's object: the envelope at the anchor proto, the
-- family and its four parts.
docRequest :: String -> [(String, Integer)] -> [(String, [[Integer]])] -> [(String, Integer)] -> Maybe Integer -> Value
docRequest fam ranges tables facts why =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("document.request" :: String)
    , "id" .= (1 :: Int)
    , "family" .= fam
    , "ranges" .= object [fromString k .= v | (k, v) <- ranges]
    , "rows" .= object [fromString k .= v | (k, v) <- tables]
    , "facts" .= object [fromString k .= v | (k, v) <- facts]
    , "degraded" .= why
    ]

-- | Two hundred requests for a family, by name.
requests :: String -> [Value]
requests fam = [runG (gen fam) (S (n * 104729 + 31) 0 0 0) | n <- [1 .. 200]]
 where
  gen f = case f of
    "arch" -> arch
    "flow" -> flow
    "merge" -> merge
    _ -> query f

num :: Int -> G Integer
num n = toInteger <$> rand n

-- | Keep each item with probability 1 / n.
some :: Int -> [a] -> G [a]
some n = filterM (const ((== 0) <$> rand n))

-- | A random place for each of `n` slots: a permutation of 0..n-1.
ranks :: Int -> G [Integer]
ranks n = do
  keys <- replicateM n (rand 1000)
  let order = map snd (sortOn fst (zip keys [0 :: Integer ..]))
  pure (map snd (sortOn fst (zip order [0 ..])))

arch :: G Value
arch = do
  nf <- (+ 1) <$> rand 6
  nd <- (+ 1) <$> rand 4
  let (fs, ds) = ([0 .. toInteger nf - 1], [0 .. toInteger nd - 1])
  files <- forM fs (\f -> (\d n -> [f, d, n]) <$> num nd <*> num 20)
  dirs <- forM ds (\d -> if d == 0 then pure [0, -1] else (\p -> [d, p]) <$> num (fromInteger d))
  edges <- some 3 [[f, g] | f <- fs, g <- fs, f /= g] >>= weigh
  pkg <- some 4 [[f, d] | f <- fs, d <- ds] >>= weigh
  focus <- some 3 [[f] | f <- fs]
  layers <- forM ds (\d -> (\l -> [d, l]) <$> num 4)
  cuts <- some 3 [[a, b] | a <- ds, b <- ds, a /= b] >>= mapM (\r -> (\w e -> r <> [w + 1, e]) <$> num 3 <*> num 2)
  clusters <- forM fs (\f -> (\c -> [f, c]) <$> num 3)
  misplaced <- some 3 fs >>= mapM (\f -> (\d -> [f, d]) <$> num nd)
  impact <- some 2 fs >>= mapM (\f -> (\d -> [f, d]) <$> num 3)
  metrics <- forM ds (\d -> (\i o s -> [d, i, o, s - 1]) <$> num 5 <*> num 5 <*> num 1001)
  places <- ranks (nf + nd)
  let (rf, rd) = splitAt nf places
  pure $
    docRequest
      "arch"
      [("files", toInteger nf), ("dirs", toInteger nd), ("why", 1)]
      [ ("files", files), ("dirs", dirs), ("edges", edges), ("pkgEdges", pkg), ("focus", focus)
      , ("layers", layers), ("cuts", cuts), ("clusters", clusters), ("misplaced", misplaced)
      , ("impact", impact), ("metrics", metrics), ("rankFiles", zipWith pair fs rf), ("rankDirs", zipWith pair ds rd)
      ]
      []
      Nothing
 where
  weigh = mapM (\r -> (\w -> r <> [w + 1]) <$> num 3)
  pair a b = [a, b]

-- | One of three roads: the measuring side's faults, the core's
-- errors, or a judged program (one in five of those degraded).
query :: String -> G Value
query fam = do
  road <- rand 3
  tokens <- num 40
  why <- (+ 1) <$> num 3
  counts <- replicateM 10 (num 9)
  flags <- replicateM 3 (num 2)
  goals <- (+ 1) <$> rand 3
  shapes <- forM [0 .. toInteger goals - 1] (\g -> (,,) g <$> num 2 <*> num 4)
  let heads = [[g, n, c] | (g, n, c) <- shapes]
  sorts <- forM shapes (\(g, _, c) -> (,,) g <$> num 2 <*> replicateM (fromInteger c) (subtract 1 <$> num 7))
  let sorted = [g : k : ss | (g, k, ss) <- sorts]
  preds <- forM [1000, 1001] (\c -> (c :) <$> replicateM 2 (subtract 1 <$> num 7))
  answers <- forM sorts (\(g, _, ss) -> (g :) <$> mapM (const value) ss)
  proof <- forM sorts (\(g, _, _) -> (\p as -> [g, 0, 0, -1, -1, p] <> as) <$> pickPred <*> replicateM 2 value)
  faults <- replicateM 2 ((\a b -> [a, b]) <$> num (fromInteger why) <*> num (fromInteger why))
  errors <- replicateM 2 ((\t c -> [t, c + 1]) <$> num (fromInteger tokens + 1) <*> num 9)
  degraded <- (== 0) <$> rand 5
  let judged = road == 2 && not degraded
      core = if judged then [("goals", sorted), ("preds", preds), ("answers", answers), ("proof", proof)] else []
  pure $
    docRequest
      fam
      [("at", tokens + 1), ("why", why)]
      ( [("faults", if road == 0 then faults else []), ("heads", if road == 0 then [] else heads), ("errors", if road == 1 then errors else [])]
          <> [(t, []) | t <- ["goals", "preds", "answers", "proof"], judged == False]
          <> core
      )
      (zip (words "tokens clauses prelude askWhy rulesFile query") (tokens : 3 : 2 : flags) <> zip countKeys counts)
      (if road == 2 && degraded then Just 0 else Nothing)
 where
  countKeys = words "rules queries asserts strata facts derived answers violations proofNodes proofTruncated"
  value = do
    k <- rand 3
    v <- num 1000
    pure (case k of 0 -> negate v; 1 -> 18446744073709551000 + v; _ -> v)
  pickPred = do
    k <- rand 3
    pure (case k of 0 -> -1; 1 -> 9; _ -> 1000)

flow :: G Value
flow = do
  nf <- rand 5
  why <- (+ 1) <$> num 3
  let fs = [0 .. toInteger nf - 1]
  langs <- forM fs (\f -> (\i -> [f, [0, 1, 2, 3, 4, 5, 6, 15, 16, 17, 18, 20] !! i]) <$> rand 12)
  places <- ranks nf
  shown <- some 2 [[k] | k <- [0 .. 3]]
  unlowered <- some 2 fs >>= mapM (\f -> (\n w -> [f, n, w]) <$> num 4 <*> num (fromInteger why))
  degraded <- (== 0) <$> rand 5
  findings <- if degraded then pure [] else concat <$> forM fs (\f -> rand 4 >>= \n -> replicateM n (finding f))
  refused <- if degraded then pure [] else some 2 fs >>= mapM (\f -> (\n w -> [f, n + 4, w]) <$> num 4 <*> num (fromInteger why))
  facts <- replicateM 5 (num 30)
  pure $
    docRequest
      "flow"
      [("files", toInteger nf), ("why", why)]
      [ ("langs", langs), ("rankFiles", zipWith (\f r -> [f, r]) fs places), ("shown", shown)
      , ("unlowered", unlowered), ("findings", findings), ("refused", refused)
      ]
      (zip (words "units stmts vars uses dynamicUnits") (if degraded then take 4 facts <> [0] else facts))
      (if degraded then Just (why - 1) else Nothing)
 where
  finding f = (\n k l d v -> [f, n, k, l, l + d, v - 1]) <$> num 3 <*> num 4 <*> num 5 <*> num 3 <*> num 4

merge :: G Value
merge = do
  ng <- rand 4
  sizes <- replicateM ng ((+ 2) <$> rand 2)
  let members = [[k, g, m] | (k, (g, m)) <- zip [0 ..] [(toInteger g, toInteger m) | (g, n) <- zip [0 :: Int ..] sizes, m <- [0 .. n - 1]]]
  memberRows <- forM members (\r -> (\a l u -> r <> [a + 1, a + 1 + l, a + 1, a + 1 + l, u]) <$> num 50 <*> num 9 <*> num 2)
  groups <- forM (zip [0 ..] sizes) (\(g, n) -> suggestion g n)
  holes <- concat <$> forM (zip [0 ..] sizes) (\(g, n) -> concat <$> forM [0 .. 2] (\h -> (\p -> [[g, h, p, toInteger m, toInteger m - 1, toInteger m] | m <- [0 .. n - 1]]) <$> num 2))
  facts <- replicateM 6 (num 30)
  pure $
    docRequest
      "merge"
      [("members", toInteger (length members)), ("why", 1)]
      [("groups", groups), ("members", memberRows), ("holes", holes)]
      (zip (words "nodes merged_duplicates not_isomorphic no_slot_table unbuilt over_cap") facts)
      Nothing
 where
  suggestion g n = do
    reason <- num 6
    fam <- num 2
    frag <- num 2
    params <- num 3
    kept <- num n
    savings <- subtract 2 <$> num 9
    pure [g, params, kept, savings, if reason == 0 then 1 else 0, reason, fam, frag]
