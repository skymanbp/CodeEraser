-- | The extent law (plan v2.35 score amendment): the check score is
-- the share of the lines no finding is about,
--
--     score = floor (scale * (1 - k * E / L)),  not below 0,
--
-- L = the lines of every file some family reads, E = the lines the
-- findings are about, each line once, exact Rational. This module is
-- the fold over one file's facts (a 'Unit', assembled by
-- CE.Verdict.ExtentFacts from the request); it knows no wire.
--
-- Axis codes are the reply's: 0 size, 1 complexity, 2 clone, 3 docdup,
-- 4 dead, 5 churn, 6 cycle. Four axes name lines (clone, docdup, dead,
-- cycle: the located lines N). Three are excesses of a measured
-- quantity over its allowance and have no place of their own: a
-- function's cognitive complexity over its ceiling (the share 1 - C/c
-- of its lines), a file's lines beyond its soft line, a rewrite-heavy
-- file's churn excess. Such an excess counts only as far as it exceeds
-- the lines already named where it lives:
--
--     in(R) = |N inside R, outside every child of R| + sum over the
--             children g of R of T_g,    T_g = max(x_g, in(g))
--     T     = in(the whole file)
--     E_f   = min(lines, max(T, lines - S, churn excess))
--
-- (a file the size family does not size - a prose-only file, read by
-- the docdup family alone - has no `lines - S` term: S is absent)
--
-- over ALL the file's functions as a forest by span containment
-- ('forest' reads spans only, never a c; x_g = the function's lines
-- after its cut * max(0, 1 - C/c), 0 at and below the ceiling). A c
-- therefore moves exactly one x_g, upward, and nothing else: raising
-- any c never lowers E. The union of named
-- lines is per file by sort and merge.
module CE.Verdict.Extent
  ( Unit
  , unit
  , extentOf
  , layered
  , extentScore
  , tenths
  , ownShares
  , totalLines
  ) where

import Data.List (nub, sort, sortOn)
import Data.Ratio ((%))

-- | One file's facts, every number already resolved by its family;
-- made by 'unit' only, so its forest is built once and shared by
-- every evaluation (each weight layer, each own share).
data Unit = Unit
  { uLines :: Integer
  -- ^ the file's line count
  , uSoft :: Maybe Integer
  -- ^ its soft line S (its class's where declared, the tree's otherwise);
  -- Nothing = the size family does not size the file: no size term
  , uNamed :: [(Integer, Integer, Integer)]
  -- ^ located lines as (axis code, lo, hi), any order, possibly overlapping
  , uForest :: [Node]
  -- ^ the file's function forest ('forest' over every function row)
  , uChurn :: Rational
  -- ^ the churn excess in lines, 0 when the file is not rewrite-heavy
  }

-- | A unit from its line count, soft line, located lines, every
-- function as (lo, hi, its share over the ceiling: 1 - C/c above it,
-- 0 at and below) and churn excess.
unit :: Integer -> Maybe Integer -> [(Integer, Integer, Integer)] -> [(Integer, Integer, Rational)] -> Rational -> Unit
unit n s named funcs = Unit n s named (forest n funcs)

axisCodes :: [Integer]
axisCodes = [0 .. 6]

-- | E_f with only the axes `on` admits.
unitExtent :: (Integer -> Bool) -> Unit -> Rational
unitExtent on u = min (n % 1) (maximum [t, size, churn])
 where
  n = uLines u
  clip lo hi = (max 1 lo, min n hi)
  named = merged [clip lo hi | (a, lo, hi) <- uNamed u, on a, uncurry (<=) (clip lo hi)]
  t = inside named (1, n) (if on 1 then uForest u else [])
  size = if on 0 then maybe 0 (\s -> max 0 (n - s) % 1) (uSoft u) else 0
  churn = if on 5 then uChurn u else 0

-- | Disjoint, ascending intervals covering exactly the input's lines.
merged :: [(Integer, Integer)] -> [(Integer, Integer)]
merged = reverse . foldl join [] . sortOn fst
 where
  join ((lo, hi) : done) (lo', hi')
    | lo' <= hi + 1 = (lo, max hi hi') : done
  join done r = r : done

-- | How many lines of the merged intervals fall in [lo, hi].
covered :: [(Integer, Integer)] -> (Integer, Integer) -> Integer
covered spans (lo, hi) = sum [max 0 (min hi b - max lo a + 1) | (a, b) <- spans]

-- | A function after its cut: (lo, hi, its share over the ceiling, children).
data Node = Node Integer Integer Rational [Node]

-- | in(R): the named lines of R outside its children, plus each child's
-- T_g = max(x_g, in(g)).
inside :: [(Integer, Integer)] -> (Integer, Integer) -> [Node] -> Rational
inside named (lo, hi) kids =
  (covered named (lo, hi) - sum [covered named (a, b) | Node a b _ _ <- kids]) % 1
    + sum [max ((b - a + 1) % 1 * x) (inside named (a, b) sub) | Node a b x sub <- kids]

-- | The functions of a file of n lines as a forest by span containment,
-- one stack pass in the order (lo ascending, hi descending), each span
-- clipped to [1, n]: an open node the next function does not lie in
-- closes, and if it owns that function's first line the function begins
-- after it; a function begins no earlier than the node it lies in; a
-- function left without a line is absent.
forest :: Integer -> [(Integer, Integer, Rational)] -> [Node]
forest n = finish . foldl step ([], []) . sortOn (\(lo, hi, _) -> (lo, negate hi))
 where
  step (stack, roots) (lo0, hi0, x) =
    let hi = min n hi0
        (lo1, stack', roots') = pops (max 1 lo0) hi stack roots
        lo2 = case stack' of
          ((plo, _, _, _) : _) -> max lo1 plo
          [] -> lo1
     in if lo2 > hi then (stack', roots') else ((lo2, hi, x, []) : stack', roots')
  pops lo hi stack@((_, fhi, _, _) : _) roots
    | lo > fhi || hi > fhi = let (s', r') = close stack roots in pops (if lo <= fhi then fhi + 1 else lo) hi s' r'
  pops lo _ stack roots = (lo, stack, roots)
  -- the top frame becomes a node, attached to the frame below or to the roots
  close ((flo, fhi, fx, kids) : rest) roots =
    let node = Node flo fhi fx (reverse kids)
     in case rest of
          ((plo, phi, px, pk) : below) -> ((plo, phi, px, node : pk) : below, roots)
          [] -> ([], node : roots)
  close [] roots = ([], roots)
  finish (stack, roots) = if null stack then reverse roots else finish (close stack roots)

extentOf :: (Integer -> Bool) -> [Unit] -> Rational
extentOf on = sum . map (unitExtent on)

totalLines :: [Unit] -> Integer
totalLines = sum . map uLines

-- | The weighted extent: with r_i = w_i / max w and the distinct
-- positive r in falling order r_1 > r_2 > ... (r after the last = 0),
-- E = sum over k of (r_k - r_k+1) * E(the axes with r_i >= r_k).
-- Equal weights read E(every axis); a weight 0 removes its axis. The
-- weight table is the effective one (code, w) for every code; the
-- boundary refuses an all-zero table, so max w > 0.
layered :: [(Integer, Integer)] -> [Unit] -> Rational
layered weights units = sum [(r - r') * extentOf (\a -> ratio a >= r) units | (r, r') <- zip levels (drop 1 levels <> [0])]
 where
  top = maximum (1 : map snd weights)
  ratio a = maybe 0 (% top) (lookup a weights)
  levels = reverse (sort (nub [w % top | (_, w) <- weights, w > 0]))

-- | floor (scale * (1 - k * E / L)), not below 0; L = 0 reads the scale.
extentScore :: Integer -> Rational -> Rational -> Integer -> Integer
extentScore scale k e len
  | len <= 0 = scale
  | otherwise = max 0 (floor ((scale % 1) * (1 - k * e / (len % 1))))

-- | E times ten, rounded half up: the one figure of E that leaves the
-- fold (the reply's `extent.tenths`, printed by every face). The exact
-- E stays here: its reduced denominator is the least common multiple
-- of data-dependent denominators (a function's c, a file's rewritten
-- lines) and has no bound in 64 bits. An integer, at most ten times
-- the lines, since E <= L.
tenths :: Rational -> Integer
tenths e = floor (10 * e + 1 % 2)

-- | Each axis's own share: floor (scale * E(axis alone) / L), unweighted,
-- without k; 0 on an empty tree.
ownShares :: Integer -> [Unit] -> [(Integer, Integer)]
ownShares scale units = [(a, share a) | a <- axisCodes]
 where
  len = totalLines units
  share a
    | len <= 0 = 0
    | otherwise = floor ((scale % 1) * extentOf (== a) units / (len % 1))
