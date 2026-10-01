-- | The seeded document requests of step 5's ten report families (plan
-- v2.32 step 5): two hundred well-formed requests per family — scan,
-- dedup, clone and its unit listing, docdup, the erase plan and its
-- trail, churn, trend and similar — the rows a judgment could have
-- answered with the measuring side's tables and facts, every code
-- inside its table and every switch both ways (`--check` with and
-- without a held veto, the trail present or absent, a similar run the
-- core did not judge one time in six); drawn by the flow reference's
-- generator, as DocumentGen, DocumentGen4 and DocumentGen5 draw theirs.
module DocumentGen6 (requests6) where

import Control.Monad (forM, replicateM)
import Data.Aeson (Value)
import Data.Maybe (fromMaybe)
import DocumentGen (docRequest, num, seededBy, some)
import ReferenceFlowGen (G, rand)

requests6 :: String -> [Value]
requests6 fam = seededBy 6007 41 (fromMaybe similar (lookup fam (zip names gens)))
 where
  names = words "scan dedup clone clone-units docdup erase erase-trail churn trend"
  gens = [scan, dedup, clone, cloneUnits, docdup, erase, trail, churn, trend]

-- | Fact rows named by the words, each drawn below the bound.
drawn :: String -> Int -> G [(String, Integer)]
drawn names bound = mapM (\k -> (,) k <$> num bound) (words names)

-- | The judged language codes a file may carry (CE.Lang's rows).
langCodes :: [Integer]
langCodes = [0, 1, 3, 5, 6, 8, 10, 15, 18, 21]

scan :: G Value
scan = do
  nf <- rand 4
  perFile <- replicateM nf (rand 3)
  let fs = [0 .. toInteger nf - 1]
      owners = concat [replicate k f | (f, k) <- zip fs perFile]
      nu = toInteger (length owners)
      nrows = toInteger nf + 6 * nu
  files <- forM fs (\f -> (\t c l -> [f, t, c, langCodes !! fromInteger l]) <$> num 900 <*> num 50 <*> num (length langCodes))
  fns <- forM (zip [0 ..] owners) (\(u, f) -> (\s n p ok cc coc nest -> [u, f, s + 1, s + n + 1, n + 1, p, ok, cc, coc, nest]) <$> num 200 <*> num 90 <*> num 8 <*> num 2 <*> num 20 <*> num 25 <*> num 6)
  levels <- some 3 [0 .. nrows - 1] >>= mapM (\r -> (\l c -> [r, l + 1, c]) <$> num 2 <*> num 2)
  overrides <- some 3 [[1, c, 10 + c, 20 * c] | c <- [0 .. 6]]
  failed <- some 2 [[0], [1], [2]]
  let grades = [[0, 300, 750], [1, 50, 75], [2, 5, 0], [3, 15, 0], [4, 15, 0], [5, 4, 0], [6, 0, 0]]
  pure (docRequest "scan" [("files", toInteger nf), ("fns", nu), ("rows", nrows)] [("files", files), ("fns", fns), ("levels", levels), ("grades", grades), ("overrides", overrides), ("failed", failed)] [] Nothing)

dedup :: G Value
dedup = do
  np <- (+ 1) <$> rand 4
  ng <- rand 3
  let ps = [0 .. toInteger np - 1]
  blocks <- some 2 [(a, b) | a <- ps, b <- ps, a <= b] >>= mapM (\(a, b) -> (\s l t d -> [a, s + 1, s + l + 1, b, s + 5, s + l + 5, t + 50, d + 7]) <$> num 300 <*> num 40 <*> num 200 <*> num 30)
  groups <- forM [0 .. toInteger ng - 1] (\g -> (\b t -> [g, b + 1, t + 50]) <$> num 4 <*> num 200)
  members <- fmap concat (forM [0 .. toInteger ng - 1] (\g -> some 2 ps >>= mapM (\f -> (\s -> [g, f, s + 1, s + 9]) <$> num 300)))
  facts <- drawn "files refreshed removed hot_chained stale_skipped low_diversity_suppressed" 40
  check <- num 2
  fails <- if check == 1 then num 2 else pure 0
  budget <- num 6
  pure (docRequest "dedup" [("paths", toInteger np), ("groups", toInteger ng)] [("blocks", blocks), ("groups", groups), ("members", members)] (facts <> [("min_tokens", 50), ("min_distinct", 7), ("check", check), ("budget", budget), ("fail", fails)]) Nothing)

-- | The clone report's counters.
cloneFacts :: String
cloneFacts = "over_cap_units forest_units survivors s5_windowed s5_pruned_label s5_already s5_new pairs_dropped_over_cap pairs_dropped_forest sent requests prefiltered judged cached"

clone :: G Value
clone = do
  nu <- (+ 2) <$> rand 5
  let us = [0 .. toInteger nu - 1]
  pairs <- some 2 [[a, b] | a <- us, b <- us, a < b] >>= mapM (\ab -> (\t n1 n2 v -> ab <> [t, n1 + 24, n2 + 24, v]) <$> num 30 <*> num 80 <*> num 80 <*> num 2)
  facts <- drawn cloneFacts 60
  pure (docRequest "clone" [("units", toInteger nu)] [("pairs", pairs)] facts Nothing)

cloneUnits :: G Value
cloneUnits = do
  np <- (+ 1) <$> rand 3
  nu <- rand 6
  units <- forM [0 .. toInteger nu - 1] (\u -> (\f n k -> [u, f, n, k + 24]) <$> num np <*> num 3 <*> num 200)
  pure (docRequest "clone-units" [("paths", toInteger np), ("units", toInteger nu)] [("units", units)] [] Nothing)

docdup :: G Value
docdup = do
  np <- (+ 1) <$> rand 3
  ns <- (+ 2) <$> rand 5
  let ss = [0 .. toInteger ns - 1]
  segs <- forM ss (\s -> (\f a l k -> [s, f, a + 1, a + l + 1, k]) <$> num np <*> num 200 <*> num 30 <*> num 5)
  pairs <- some 2 [[a, b] | a <- ss, b <- ss, a < b] >>= mapM (\ab -> (\i u r v -> ab <> [i, i + u + 1, r, v]) <$> num 60 <*> num 20 <*> num 80 <*> num 2)
  facts <- drawn "over_cap_segments lsh_pairs seed_pairs hot_bands hot_shingles sent requests judged jaccard_dups exempt_license exempt_allow" 40
  check <- num 2
  pure (docRequest "docdup" [("paths", toInteger np), ("segs", toInteger ns)] [("segs", segs), ("pairs", pairs)] (("check", check) : facts) Nothing)

erase :: G Value
erase = do
  np <- (+ 1) <$> rand 3
  nc <- rand 7
  cands <- replicateM nc $ do
    cls <- (+ 1) <$> num 3
    f <- num np
    spanned <- num 2
    s <- num 100
    sites <- num 9
    ok <- num 2
    reason <- if ok == 1 then pure 0 else (+ 1) <$> num 6
    kept <- (\x -> if x == 0 then 0 else 1) <$> rand 4
    pure [cls, f, spanned, s + 1, s + 12, sites, 18446744073709551000 - s, ok, reason, kept]
  out <- some 2 [[0, 3]]
  check <- num 2
  apply <- num 2
  applied <- if apply == 1 then num 3 else pure 0
  pure (docRequest "erase" [("paths", toInteger np), ("cands", toInteger nc)] [("cands", cands), ("outOfClass", out)] [("check", check), ("apply", apply), ("applied", applied)] Nothing)

trail :: G Value
trail = do
  present <- num 2
  nr <- if present == 1 then rand 4 else pure 0
  nb <- if present == 1 then rand 3 else pure 0
  np <- (+ 1) <$> rand 3
  records <- forM [0 .. toInteger nr - 1] (\i -> (\ts c f sp s -> [i, 1700000000000 + ts * 86399999, c, f, sp, s + 1, s + 7]) <$> num 9000 <*> num 4 <*> num np <*> num 2 <*> num 90)
  unread <- forM [0 .. toInteger nb - 1] (\k -> (\l -> [k, l + 1]) <$> num 40)
  pure (docRequest "erase-trail" [("paths", toInteger np), ("records", toInteger nr), ("unread", toInteger nb)] [("records", records), ("unreadable", unread)] [("present", present)] Nothing)

churn :: G Value
churn = do
  np <- (+ 2) <$> rand 4
  nsub <- rand 3
  let ps = [0 .. toInteger np - 1]
  wide <- num 4
  pairs <- if wide == 0 then replicateM 23 ((\a b n -> [a, b, n + 2]) <$> num np <*> num np <*> num 9) else some 2 [[a, b, 3] | a <- ps, b <- ps, a < b]
  facts <- drawn "commits appended rewrote surviving skipped" 300
  days <- (+ 1) <$> num 30
  pure (docRequest "churn" [("paths", toInteger np), ("submodules", toInteger nsub)] [("cochange", pairs)] (("days", days) : facts) Nothing)

trend :: G Value
trend = do
  npt <- rand 6
  nfail <- rand 3
  points <- forM [0 .. toInteger npt - 1] (\i -> (\ts sc ax -> [i, 1700000000 + ts, sc + 500, 1000] <> concat [[a, sc `mod` 300] | a <- [0 .. ax]]) <$> num 99999 <*> num 500 <*> num 7)
  judged <- num 2
  slope <- if judged == 1 && npt > 0 then (\s -> [[s - 2500]]) <$> num 5000 else pure []
  verdict <- if null slope then pure [] else (\v -> [[v]]) <$> num 3
  cliff <- if npt > 1 then some 2 [[toInteger npt - 1, 1350]] else pure []
  run <- if npt > 1 then some 2 [[0, toInteger npt]] else pure []
  floor' <- num 3
  fails <- num 2
  pure (docRequest "trend" [("points", toInteger npt), ("failed", toInteger nfail)] [("points", points), ("slope", slope), ("verdict", verdict), ("cliff", cliff), ("declineRun", run), ("knobs", [[0, 3], [1, floor' * 250]])] [("window", toInteger (npt + nfail) + 2), ("pending", toInteger nfail), ("fail", fails)] Nothing)

similar :: G Value
similar = do
  ns <- (+ 1) <$> rand 6
  degraded <- (== 0) <$> rand 6
  nc <- rand 7
  widen <- num 2
  cands <- replicateM nc $ do
    seat <- num ns
    hits <- replicateM 6 (num 5)
    rest <- (\nth sc sh w r -> [nth, sc, sh, w * widen, if degraded then 2 else r]) <$> num 3 <*> num 40 <*> num 2 <*> num 2 <*> num 3
    pure ([seat] <> take 2 rest <> hits <> drop 2 rest)
  facts <- (\t -> [("terms", t), ("widen", widen), ("similarRev", 1)]) <$> num 30
  pure (docRequest "similar" [("seats", toInteger ns), ("why", 1)] [("candidates", cands)] facts (if degraded then Just 0 else Nothing))
