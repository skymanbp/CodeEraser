-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Two hundred seeded structure requests behind StructureEquivProps
-- (plan v2.32 step 7): a tree of one to nine directories with chained
-- depths, a name-pattern distribution as shape facts (or none),
-- conventions, file references — consistent with a directed dir-edge
-- table when one rides — a declared layout, raw staleness facts, a
-- redundancy rollup, the seam tables of up to three files and a
-- seeded knob table; each optional table rides one time in two. The
-- LCG is ReferenceFlowGen's; nothing here judges.
module StructureGen (cases, encode') where

import Data.Aeson (Value, object, (.=))
import Data.List (nub, sort, sortOn)
import ReferenceFlowGen (G, S (..), rand, runG)
import ReferenceStructure (SIn (..))

cases :: [SIn]
cases = [runG one (S (n * 5501 + 13) 0 0 0) | n <- [1 .. 200 :: Int]]

int :: Int -> G Integer
int n = toInteger <$> rand n

chance :: Int -> G Bool
chance k = (== 0) <$> rand k

-- | Optional: Just one time in two.
maybeOf :: G a -> G (Maybe a)
maybeOf g = chance 2 >>= \yes -> if yes then Just <$> g else pure Nothing

-- | Each candidate kept one time in k.
keep :: Int -> [a] -> G [a]
keep k xs = concat <$> mapM (\x -> (\yes -> [x | yes]) <$> chance k) xs

one :: G SIn
one = do
  n <- (+ 1) <$> int 9
  parents <- mapM (\i -> int (fromInteger i)) [1 .. n - 1]
  files <- mapM (const (int 13)) [0 .. n - 1]
  road <- rand 3
  shapes <- if road == 0 then pure Nothing else Just <$> distribution n 128
  convs <- keep 2 [0 .. n - 1] >>= mapM (\d -> (\b -> [d, b + 1]) <$> int 3)
  edged <- chance 2
  (refs, edges) <- if edged then crossing n else (\r -> (r, Nothing)) <$> loose n
  decl <- chance 2 >>= \yes -> if yes then keep 2 [0 .. n - 1] >>= mapM (\d -> (\w -> [d, w + 1]) <$> int 5) else pure []
  docs <- maybeOf (sortOn (take 1) <$> (rand 6 >>= \c -> mapM (const ((\d t -> [d, t]) <$> int (fromInteger n) <*> int 4)) [1 .. c]))
  staleEdges <- maybe (pure []) (\ds -> if null ds then pure [] else rand 7 >>= \c -> mapM (const ((\i t -> [i, t + 1]) <$> int (length ds) <*> int 4)) [1 .. c]) docs
  red <- maybeOf (keep 2 [0 .. n - 1] >>= mapM (\d -> (\a b -> [d, a, b]) <$> int 3 <*> int 3))
  seams <- maybeOf (rand 3 >>= \c -> mapM seamFile [0 .. toInteger c])
  ks <- keep 4 [0 .. 20] >>= mapM (\c -> (\v -> [c, v]) <$> knobValue c)
  let nodes = [[i, p, depthOf i, toInteger (length (filter (== i) parents)), f] | (i, p, f) <- zip3 [0 ..] (0 : parents) files]
      depthOf i = if i == 0 then 0 else 1 + depthOf ((0 : parents) !! fromInteger i)
      bundle = maybe ([], [], [], []) (\fs -> (concatMap (\(_, u, _, _, _) -> u) fs, concatMap (\(_, _, r, _, _) -> r) fs, concatMap (\(_, _, _, cl, _) -> cl) fs, concatMap (\(_, _, _, _, ch) -> ch) fs)) seams
  pure (SIn nodes shapes convs refs decl docs staleEdges red edges (map (\(t, _, _, _, _) -> t) <$> seams) bundle ks)

-- | [dir, key, count] rows ascending, a few keys per directory.
distribution :: Integer -> Int -> G [[Integer]]
distribution n keys = concat <$> mapM (\d -> rand 4 >>= \c -> mapM (const (int keys)) [1 .. c] >>= mapM (\key -> (\cnt -> [d, key, cnt + 1]) <$> int 5) . nub . sort) [0 .. n - 1]

-- | fileRefs without a dir-edge table: per-file splits, free.
loose :: Integer -> G [[Integer]]
loose n = nubOn . sort . concat <$> mapM (\d -> rand 4 >>= \c -> mapM (const ((\i o cnt -> [d, i, o, cnt + 1]) <$> int 5 <*> int 9 <*> int 3)) [1 .. c]) [0 .. n - 1]
 where
  nubOn rows = [r | (i, r) <- zip [0 :: Int ..] rows, take 3 r `notElem` map (take 3) (take i rows)]

-- | A dir-edge table and the one fileRefs row per directory that
-- describes the same graph: inside twice the internal edges, outside
-- the incident crossing mass.
crossing :: Integer -> G ([[Integer]], Maybe [[Integer]])
crossing n = do
  edges <- keep 3 [(a, b) | a <- [0 .. n - 1], b <- [0 .. n - 1], a /= b] >>= mapM (\(a, b) -> (\c -> [a, b, c + 1]) <$> int 3)
  intra <- mapM (const (int 4)) [0 .. n - 1]
  pure ([[d, 2 * e, sum [c | [a, b, c] <- edges, a == d || b == d], 1] | (d, e) <- zip [0 ..] intra], Just edges)

-- | One file's seam tables: [file, total], then its units, references,
-- clone blocks and co-change pairs.
seamFile :: Integer -> G ([Integer], [[Integer]], [[Integer]], [[Integer]], [[Integer]])
seamFile f = do
  k <- (+ 1) <$> rand 5
  spans <- mapM (const ((,) <$> int 6 <*> ((+ 1) <$> int 260))) [1 .. k]
  extra <- int 400
  let ends = scanl1 (+) [g + l | (g, l) <- spans]
      starts = [e - l + 1 | (e, (_, l)) <- zip ends spans]
      total = last ends + extra
      units = [[f, u, s, e] | (u, s, e) <- zip3 [0 ..] starts ends]
      ids = [0 .. toInteger k - 1]
  refs <- keep 3 [(a, b) | a <- ids, b <- ids, a /= b]
  churn <- keep 4 [(a, b) | a <- ids, b <- ids, a < b]
  clones <- rand 3 >>= \c -> mapM (const ((\s l -> [f, s + 1, min total (s + 1 + l)]) <$> int (fromInteger total) <*> int 300)) [1 .. c]
  pure ([f, total], units, [[f, a, b] | (a, b) <- refs], nub (sort clones), [[f, a, b] | (a, b) <- churn])

-- | A knob value in a range where the code's axis or price moves.
knobValue :: Integer -> G Integer
knobValue c = (+ 1) <$> int (maybe 9 id (lookup c [(3, 900), (7, 20), (8, 1500), (12, 400), (13, 800), (15, 600), (16, 600), (17, 600), (18, 600), (19, 900)]))

-- | The structure.request a case spells; absent optional tables do
-- not ride.
encode' :: SIn -> Value
encode' i =
  object
    ( ["proto" .= ("7.0.0" :: String), "type" .= ("structure.request" :: String), "id" .= (1 :: Int), "nodes" .= sNodes i]
        <> ["patternShapes" .= p | Just p <- [sShapes i]]
        <> ["conventions" .= sConventions i, "fileRefs" .= sRefs i, "declared" .= sDeclared i, "staleEdgeRows" .= sStaleEdges i, "knobs" .= sKnobs i]
        <> ["staleDocRows" .= d | Just d <- [sStaleDocs i]]
        <> ["redundancy" .= r | Just r <- [sRedundancy i]]
        <> ["dirEdges" .= e | Just e <- [sDirEdges i]]
        <> ["seamFiles" .= f | Just f <- [sSeamFiles i]]
        <> zipWith (.=) ["seamUnits", "seamRefs", "seamClones", "seamChurn"] (let (u, r, c, h) = sSeams i in [u, r, c, h])
    )
