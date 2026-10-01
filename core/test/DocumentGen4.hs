-- | The seeded document requests behind DocumentProps4 (plan v2.32
-- step 4): two hundred well-formed requests per family for check,
-- structure, join, deadcode, mentions, sites and the graph screen —
-- the rows a judgment could have answered, the measuring side's facts
-- and ranks, one in six check / join / deadcode / screen requests the
-- judgment that never happened — drawn by the flow reference's
-- generator, as DocumentGen draws step 3's.
module DocumentGen4 (requests4) where

import Control.Monad (forM, replicateM)
import Data.Aeson (Value)
import DocumentGen (docRequest, num, ranks, seededBy, some)
import ReferenceFlowGen (G, rand)

-- | Two hundred requests for a family, by name.
requests4 :: String -> [Value]
requests4 fam = seededBy 7919 17 gen
 where
  gen = case fam of
    "check" -> check
    "structure" -> structure
    "join" -> join'
    "deadcode" -> deadcode
    "mentions" -> mentions
    "sites" -> sites
    _ -> screen

-- | The judged languages' wire codes (CE.Lang's rows).
judgedCodes :: [Integer]
judgedCodes = [0, 1, 2, 3, 4, 5, 6, 10, 15, 16, 17, 18, 20]

-- | Anchors as wide as the ledger's (past i64) and two small ones.
wide :: [Integer]
wide = [18446744073709551000, 7, 4096]

-- | A judged request's tables and facts, or — the judgment never
-- happened — the same keys empty and zero with the reason 0.
judgedOr :: Bool -> [(String, [[Integer]])] -> [(String, Integer)] -> ([(String, [[Integer]])], [(String, Integer)], Maybe Integer)
judgedOr degraded ts fs
  | degraded = ([(t, []) | (t, _) <- ts], [(k, 0) | (k, _) <- fs], Just 0)
  | otherwise = (ts, fs, Nothing)

check :: G Value
check = do
  nf <- rand 6
  let fs = [0 .. toInteger nf - 1]
  degraded <- (== 0) <$> rand 6
  scale <- some 2 [[1000]]
  floor' <- some 2 [[900]]
  reason <- some 4 [[1]]
  axes <- mapM (\a -> (\p -> [a, p]) <$> num 300) [0 .. 6]
  cands <- some 2 [[u, v] | u <- fs, v <- fs, u < v] >>= mapM (\uv -> (\c r l k -> uv <> [c, r, l, k]) <$> num 4 <*> num 512 <*> num 8 <*> num 4)
  (added, removed) <- (,) <$> some 2 (map pure wide) <*> some 2 (map pure wide)
  over <- some 2 [[x, a, 400, 350] | (x, a) <- zip wide [0 ..]]
  drawn <- some 2 [[x, 0, 7] | x <- wide]
  failed <- some 3 [[c] | c <- [0 .. 6]]
  rode <- num 2
  dropped <- if rode == 1 then some 2 [[x, 0, 12] | x <- wide] else pure []
  counts <- (:) <$> num 1001 <*> replicateM 5 (num 40)
  roast <- num 2
  let tables = [("scale", scale), ("floor", floor'), ("reason", reason), ("axes", axes), ("candidates", cands), ("joinSeverity", [[1, 2], [2, 3], [3, 1]])]
        <> [("added", added), ("removed", removed), ("over", over), ("toleranceDrawn", drawn), ("failed", failed), ("dropped", dropped)]
      facts = zip (words "score fail simPairs members collapsed skippedSelf") counts <> [("droppedRode", rode), ("roast", roast)]
      (ts, fs', why) = judgedOr degraded tables (map bit facts)
      bit (k, v) = if k == "fail" then (k, v `mod` 2) else (k, v)
  pure (docRequest "check" [("files", toInteger nf), ("why", 1)] ts fs' why)

structure :: G Value
structure = do
  nd <- (+ 1) <$> rand 6
  ns <- rand 4
  let (ds, ss) = ([0 .. toInteger nd - 1], [0 .. toInteger ns - 1])
  tree <- forM ds (\d -> (\p dep sub f -> [d, if d == 0 then 0 else p, dep, sub, f]) <$> num (max 1 (fromInteger d)) <*> num 5 <*> num 4 <*> num 9)
  findings <- some 2 [[d, a] | d <- ds, a <- [0 .. 6]]
  deviations <- some 3 [[d, k] | d <- ds, k <- [0, 1]]
  split <- num 2
  cands <- if split == 1 then some 2 [[f, u] | f <- ss, u <- [0, 1]] >>= mapM (\fu -> (\b c l -> fu <> [b, c, l + 1]) <$> num 3000 <*> num 2000 <*> num 400) else pure []
  exempt <- if split == 1 then some 2 ss >>= mapM (\f -> (\b c -> [f, b, c]) <$> num 3000 <*> num 2000) else pure []
  (days, divergence) <- (,) <$> some 2 [[14]] <*> some 2 [[37]]
  entropy <- mapM (\k -> (\v -> [k, v]) <$> num 1000) [0, 1]
  axes <- mapM (\a -> (\p -> [a, p]) <$> num 200) [0 .. 7]
  facts <- sequence [num 1001, pure 1000, num 6, num 2, pure split]
  pure $
    docRequest
      "structure"
      [("dirs", toInteger nd), ("seamFiles", toInteger ns)]
      (zip (words "days divergence entropy axes tree findings deviations splitCandidates sizeExempt") [days, divergence, entropy, axes, tree, findings, deviations, cands, exempt])
      (zip (words "score scale declaredDirs deep split") facts)
      Nothing

join' :: G Value
join' = do
  np <- rand 6
  let ps = [0 .. toInteger np - 1]
  places <- ranks np
  pairs <- some 2 [[a, b] | a <- ps, b <- ps, a /= b]
  files <- mapM (\ab -> (\bl t n ch -> ab <> [bl, t, n] <> ch) <$> num 4 <*> num 300 <*> num 3 <*> replicateM 4 (num 50)) pairs
  pos <- some 2 ps >>= mapM (\p -> (p :) <$> sequence [num 5, num 5, subtract 1 <$> num 4, num 3, num 6])
  cochange <- some 2 pairs >>= mapM (\ab -> (\n -> ab <> [n]) <$> num 9)
  cands <- some 2 pairs >>= mapM (\ab -> (\c k -> ab <> [c, 5, 3, k]) <$> num 4 <*> num 4)
  nu <- if np == 0 then pure 0 else rand 4
  units <- forM [0 .. toInteger nu - 1] (\k -> (\a an b bn fam ms ch -> [k, a, an, b, bn, fam] <> ms <> ch) <$> num np <*> num 3 <*> num np <*> num 3 <*> num 2 <*> replicateM 3 (num 90) <*> replicateM 4 (num 20))
  (gr, vr) <- (,) <$> some 5 [[0]] <*> some 5 [[1]]
  degraded <- (== 0) <$> rand 6
  facts <- sequence [(+ 1) <$> num 30, num 100]
  let tables = [("rankPaths", zipWith (\p r -> [p, r]) ps places)] <> judgedTables
      judgedTables = [("files", files), ("pos", pos), ("cochange", cochange), ("candidates", cands), ("joinSeverity", [[1, 2], [2, 3], [3, 1]]), ("units", units), ("graphReason", gr), ("verdictReason", vr)]
      (ts, fs, why) = judgedOr degraded tables (zip ["days", "commits"] facts)
  pure (docRequest "join" [("paths", toInteger np), ("units", if degraded then 0 else toInteger nu), ("why", 1)] ts fs why)

-- | The deadcode request over `nodes` nodes, its dead rows drawn from
-- `deadOn`: the ranges, tables and facts, and whether the judgment
-- happened.
deadParts :: Integer -> [Integer] -> G ([(String, Integer)], [(String, [[Integer]])], [(String, Integer)], Maybe Integer)
deadParts nodes deadOn = do
  let ns = [0 .. nodes - 1]
  dead <- some 2 deadOn >>= mapM (\n -> (\v t w -> [n, v + 1] <> [t | w /= 0]) <$> num 4 <*> num 3 <*> rand 3)
  reported <- some 3 ns >>= mapM (\n -> (\v -> [n, v + 1]) <$> num 4)
  asked <- num 2
  dropped <- if asked == 1 then (\x -> if x == 0 then 1 else 0) <$> rand 4 else pure 0
  cut <- if asked == 1 then num 2 else pure 0
  na <- if asked == 1 && dropped == 0 then rand 5 else pure 0
  advisory <- forM [0 .. toInteger na - 1] (\k -> (\n l c -> [k, n, l + 1, c]) <$> num (fromInteger nodes) <*> num 300 <*> num 4)
  (kept, reason) <- (,) <$> some 2 [[11]] <*> some 6 [[0]]
  unresolved <- num 20
  degraded <- (== 0) <$> rand 6
  (files, check') <- (,) <$> num (fromInteger nodes + 1) <*> num 2
  let tables = [("kept", kept), ("reason", reason), ("dead", dead), ("reported", reported), ("unmentioned", advisory)]
      facts = [("unresolvedSites", unresolved), ("asked", asked), ("dropped", dropped), ("cut", cut), ("files", files), ("check", check')]
      (ts, fs, why) = judgedOr degraded tables facts
  pure ([("nodes", nodes), ("advisory", if degraded then 0 else toInteger na), ("why", 1)], ts, fs, why)

deadcode :: G Value
deadcode = do
  nn <- (+ 1) <$> rand 8
  (rs, ts, fs, why) <- deadParts (toInteger nn) [0 .. toInteger nn - 1]
  pure (docRequest "deadcode" rs ts fs why)

-- | The graph screen: nodes of three kinds, every file node its own
-- file row, a section naming a file row or none; the deadcode request
-- with its dead rows on file nodes, and the canvas's tables.
screen :: G Value
screen = do
  nn <- (+ 1) <$> rand 8
  kinds <- replicateM nn (num 3)
  let ns = [0 .. toInteger nn - 1]
      fileNodes = [n | (n, 0) <- zip ns kinds]
      nf = toInteger (length fileNodes)
  graph <- forM (zip3 ns kinds (scanl (\i k -> if k == 0 then i + 1 else i) 0 kinds)) $ \(n, k, i) -> case k of
    0 -> pure [n, 0, i]
    1 -> pure [n, 1, -1]
    _ -> (\x -> [n, 2, x - 1]) <$> num (fromInteger nf + 1)
  edges <- some 3 [[s, d] | s <- ns, d <- ns]
  pos <- some 2 ns >>= mapM (\p -> (p :) <$> sequence [num 5, num 5, num 4, num 3, num 6])
  cycles <- some 2 [[c, n] | c <- [0, 1], n <- ns]
  (rs, ts, fs, why) <- deadParts (toInteger nn) fileNodes
  let canvasTables = [("graph", graph), ("edges", edges), ("pos", pos), ("cycles", cycles)]
      blank = [(t, []) | (t, _) <- canvasTables]
  pure (docRequest "graphscreen" rs (ts <> (if why == Nothing then canvasTables else blank)) fs why)

mentions :: G Value
mentions = do
  facts <- replicateM 19 (num 400)
  rescanned <- num 2
  langs <- some 2 judgedCodes
  rates <- mapM (\c -> (c :) <$> replicateM 8 (num 60)) langs
  let names =
        words
          "mention_rev universe sources rows capped dist_js_dedup_runs skipped.oversize skipped.binary \
          \skipped.signed skipped.walk_errors run.refreshed run.removed run.rescanned run.clipped run.starved \
          \outside.oversize outside.binary outside.nested outside.ignored"
      valued k v = if k == "run.rescanned" then rescanned else v
  pure (docRequest "mentions" [] [("rates", rates)] (zipWith (\k v -> (k, valued k v)) names facts) Nothing)

sites :: G Value
sites = do
  nf <- rand 5
  let fs = [0 .. toInteger nf - 1]
  langs <- forM fs (\f -> (\i -> [f, judgedCodes !! i]) <$> rand (length judgedCodes))
  places <- ranks nf
  per <- forM fs (\f -> rand 4 >>= \n -> replicateM n ((\k l o -> (f, k, l + 1, o)) <$> num 23 <*> num 200 <*> num 2))
  let rows' = [[i, f, k, l, nth, o] | (i, (nth, (f, k, l, o))) <- zip [0 ..] (concatMap (zip [0 ..]) per)]
  pure $
    docRequest
      "sites"
      [("files", toInteger nf), ("sites", toInteger (length rows'))]
      [("rankFiles", zipWith (\f r -> [f, r]) fs places), ("langs", langs), ("sites", rows')]
      []
      Nothing
