-- | Brute-force tree edit distance straight from the DEFINITION: the
-- minimum cost over all valid Tai mappings, enumerated explicitly —
-- no keyroots, no forest tables, nothing shared with the production
-- Zhang-Shasha (CE.Clone.Ted) except the wire's postorder (lab, lld)
-- form. Also owns the exhaustive small-tree family generator the
-- property battery walks (the Reference.hs set-phrased-independence
-- posture, tree edition).
module ReferenceTed (refTed, family, labelInterOf) where

-- | A tree as its wire arrays: postorder labels and leftmost-leaf
-- descendant indices.
type T = ([Int], [Int])

-- | Minimum Tai-mapping cost: cost(M) = n1 + n2 − 2|M| + relabels.
-- Backtracking over nodes of the first tree in order, each mapped to
-- an unused consistent node of the second or skipped; a lower bound
-- prunes branches that cannot beat the incumbent.
refTed :: T -> T -> Integer
refTed (lab1, lld1) (lab2, lld2) = go 0 [] best0
 where
  (n1, n2) = (length lab1, length lab2)
  best0 = fromIntegral (n1 + n2) -- the empty mapping
  go i m best
    | i == n1 = min best (cost m)
    | bound i m >= best = best
    | otherwise = foldl try (go (i + 1) m best) [0 .. n2 - 1]
   where
    try acc j
      | j `elem` map snd m = acc
      | all (consistent (i, j)) m = go (i + 1) ((i, j) : m) acc
      | otherwise = acc
  -- even mapping every remaining first-tree node cannot go below this
  bound i m = cost m - 2 * fromIntegral (min (n1 - i) (n2 - length m))
  cost m =
    fromIntegral (n1 + n2 - 2 * length m)
      + fromIntegral (length [() | (i, j) <- m, lab1 !! i /= lab2 !! j])
  -- Tai validity: order preserved both ways, ancestry preserved in
  -- BOTH directions (postorder: a sits in b's subtree ⇔
  -- lld b ≤ a ≤ b). The one-directional first draft admitted invalid
  -- mappings and under-counted — the equivalence battery caught it.
  consistent (i1, j1) (i2, j2) =
    (i1 < i2) == (j1 < j2)
      && inSub lld1 i1 i2 == inSub lld2 j1 j2
      && inSub lld1 i2 i1 == inSub lld2 j2 j1
  inSub lld a b = lld !! b <= a && a <= b

-- | I = Σ_label min(c1,c2) — the reference's own tally (the product
-- half lives in CE.Clone.Prefilter; the property battery compares
-- bounds computed from THIS side against brute-force ted).
labelInterOf :: T -> T -> Int
labelInterOf (lab1, _) (lab2, _) =
  sum [min (count l lab1) (count l lab2) | l <- labels]
 where
  labels = foldr (\l acc -> if l `elem` acc then acc else l : acc) [] (lab1 <> lab2)
  count l = length . filter (== l)

-- | Every tree with 1..maxN nodes and labels drawn from {0,1}, as
-- wire arrays — the exhaustive family. Shapes come from the rose
-- recursion (a tree = root + ordered forest), labelings are the full
-- function space.
family :: Int -> [T]
family maxN =
  [ (labs, lldOf shape)
  | n <- [1 .. maxN]
  , shape <- shapes n
  , labs <- mapM (const [0, 1]) [1 .. n]
  ]

data Rose = Rose [Rose]

shapes :: Int -> [Rose]
shapes n = [Rose f | f <- forests (n - 1)]

forests :: Int -> [[Rose]]
forests 0 = [[]]
forests k = [t : f | s <- [1 .. k], t <- shapes s, f <- forests (k - s)]

-- | Postorder lld array of a shape: children lay out left to right,
-- root last; a node's leftmost leaf is its first child's (its own
-- index for leaves).
lldOf :: Rose -> [Int]
lldOf t = snd (walk 0 t)
 where
  walk at (Rose cs) =
    let (end, rows) = foldl child (at, []) cs
        low = if null cs then end else at
     in (end + 1, concat (reverse rows) <> [low])
  child (at, acc) c = let (end, rows) = walk at c in (end, rows : acc)
