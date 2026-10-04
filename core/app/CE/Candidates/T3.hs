-- | The T3 candidate pass over integer facts (design vol.2 §4.2–§4.3;
-- plan v2.33 W3): the same-key source S2, the two admissible bounds
-- over the generators' union, then the exhaustive source S5. Until W3
-- the measuring side generated S2, applied both bounds, and the clone
-- judgment applied them again; S2 and the bounds now run here once (the
-- measuring side still generates S1, S3 and S4 from its index), and the
-- clone judgment's prefilter (CE.Clone) stays the judge's own guard.
--
-- S2 pairs every two units that share a key in different files, within
-- one language (a cross-language pair is counted and dropped). A pair
-- several sources found is one union pair carrying every source's bit.
-- S5 walks each language's units in ascending node order and pairs each
-- unit with the later ones while the size bound still admits them — the
-- window closes exactly where the size bound would cut — then cuts by
-- the label bound like everywhere else; a pair the union already kept
-- is counted, never duplicated.
module CE.Candidates.T3 (Answer (..), Tally (..), judgeT3) where

import CE.Candidates.Cost (exhaustiveSource, keySource)
import CE.Candidates.Units (Units, columns, interUpTo, nodesOf)
import qualified CE.Candidates.Units as U
import CE.Clone.Prefilter (Bound (..), boundOf, reachFloor, sizeBelow)
import Data.Array.Base (numElements, unsafeAt)
import Data.Array.Unboxed (UArray, listArray)
import Data.Bits ((.|.))
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (sortOn)

-- | Every count the pass sheds, published (the report's ledger and the
-- frozen docs' summary rows).
data Tally = Tally
  { union :: !Int
  , crossLanguage :: !Int
  , prunedSize :: !Int
  , prunedLabel :: !Int
  , survivors :: !Int
  , s5Windowed :: !Int
  , s5PrunedLabel :: !Int
  , s5Already :: !Int
  , s5New :: !Int
  }

-- | The kept pairs `[a, b, sources]` ascending, the tally, and S2's
-- pairs per language `[lang, n]` ascending.
data Answer = Answer {kept :: [[Integer]], tally :: Tally, keyPairs :: [[Integer]]}

-- | The bounds' running tallies and the kept pairs by pair key.
data Pruned = Pruned !Int !Int !(IM.IntMap Int)

-- | S2's walk: the sent pairs it also found, how many pairs it alone
-- found (pruned as they come), its cross-language drops, its pairs per
-- language.
data Same = Same !IS.IntSet !Int !Int !(IM.IntMap Int) !Pruned

-- | The S5 accumulator: windowed, label-cut, and the keys of the window
-- pairs the label bound kept.
data S5 = S5 !Int !Int !IS.IntSet

judgeT3 :: Bool -> [[Integer]] -> [[Integer]] -> Answer
judgeT3 ex rows sent = Answer (map row (IM.toAscList (IM.union keptAll s5Rows))) t (map perLang (IM.toAscList byLang))
 where
  us = columns rows
  n = U.count us
  pairKey a b = a * n + b
  -- the intersection walk stops once it provably cannot reach the
  -- threshold: what it returns is below the floor exactly when I is
  bound a b =
    let (na, nb) = (nodesOf us a, nodesOf us b)
     in boundOf na nb (interUpTo us (reachFloor (max na nb)) a b)
  prune (Pruned s l kp) k bits = case bound (k `div` n) (k `mod` n) of
    SizeBound -> Pruned (s + 1) l kp
    LabelBound -> Pruned s (l + 1) kp
    Within -> Pruned s l (IM.insert k bits kp)
  sentMap = IM.fromList [(pairKey (fromInteger a) (fromInteger b), fromInteger s) | [a, b, s] <- sent]
  Same hits fresh cross byLang afterSame =
    foldl' (sameKey us pairKey sentMap prune) (Same IS.empty 0 0 IM.empty (Pruned 0 0 IM.empty)) (groups (U.key us) n)
  sameBit k bits = if IS.member k hits then bits .|. fromInteger keySource else bits
  Pruned nSize nLabel keptAll = IM.foldlWithKey' (\acc k bits -> prune acc k (sameBit k bits)) afterSame sentMap
  -- a kept union pair passed both bounds, so inside the window it is
  -- never label-cut: the window's survivors split into the ones the
  -- union already kept and S5's new pairs without asking `have` per pair
  S5 windowed cut within
    | ex = foldl' (bucketStep us bound pairKey) (S5 0 0 IS.empty) (buckets us)
    | otherwise = S5 0 0 IS.empty
  have = IM.keysSet keptAll
  freshKeys = IS.difference within have
  already = IS.size within - IS.size freshKeys
  s5Rows = IM.fromSet (const (fromInteger exhaustiveSource)) freshKeys
  row (k, bits) = map toInteger [k `div` n, k `mod` n, bits]
  perLang (l, c) = map toInteger [l, c]
  t =
    Tally
      { union = IM.size sentMap + fresh
      , crossLanguage = cross
      , prunedSize = nSize
      , prunedLabel = nLabel
      , survivors = IM.size keptAll
      , s5Windowed = windowed
      , s5PrunedLabel = cut
      , s5Already = already
      , s5New = IS.size freshKeys
      }

-- | Unit ids grouped by a column's value, each group ascending.
groups :: UArray Int Int -> Int -> [[Int]]
groups col n = IM.elems (IM.fromListWith (++) [(col `unsafeAt` i, [i]) | i <- [n - 1, n - 2 .. 0]])

-- | One key group of the S2 walk: every two members in different files.
sameKey :: Units -> (Int -> Int -> Int) -> IM.IntMap Int -> (Pruned -> Int -> Int -> Pruned) -> Same -> [Int] -> Same
sameKey us pairKey sentMap prune acc0 ids = foldl' visit acc0 [(a, b) | (a : rest) <- suffixes ids, b <- rest]
 where
  visit acc@(Same hits fresh cross byLang pr) (a, b)
    | U.file us `unsafeAt` a == U.file us `unsafeAt` b = acc
    | la /= U.lang us `unsafeAt` b = Same hits fresh (cross + 1) byLang pr
    | IM.member k sentMap = Same (IS.insert k hits) fresh cross lang' pr
    | otherwise = Same hits (fresh + 1) cross lang' (prune pr k (fromInteger keySource))
   where
    la = U.lang us `unsafeAt` a
    k = pairKey a b
    lang' = IM.insertWith (+) la 1 byLang
  suffixes xs = case xs of
    [] -> []
    (_ : rest) -> xs : suffixes rest

-- | Each language's unit ids in ascending (nodes, id) order.
buckets :: Units -> [UArray Int Int]
buckets us = [listArray (0, length ids - 1) (sortOn (\i -> (nodesOf us i, i)) ids) | ids <- groups (U.lang us) (U.count us)]

-- | One language bucket of the S5 window walk.
bucketStep :: Units -> (Int -> Int -> Bound) -> (Int -> Int -> Int) -> S5 -> UArray Int Int -> S5
bucketStep us bound pairKey acc0 bk = outer 0 acc0
 where
  len = numElements bk
  outer p !acc
    | p >= len = acc
    | otherwise = outer (p + 1) (window (bk `unsafeAt` p) (p + 1) acc)
  window a q !acc
    | q >= len = acc
    | sizeBelow (nodesOf us a) (nodesOf us b) = acc
    | otherwise = window a (q + 1) (visit acc)
   where
    b = bk `unsafeAt` q
    lo = min a b
    hi = max a b
    visit (S5 w c kept)
      | bound lo hi == LabelBound = S5 (w + 1) (c + 1) kept
      | otherwise = S5 (w + 1) c (IS.insert (pairKey lo hi) kept)
