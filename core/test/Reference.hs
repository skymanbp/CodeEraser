-- | Exhaustive small-instance equivalence: an independently written
-- reference — bidirectional-maximality block enumeration plus the
-- set-form definitions of extension and attribution — must agree
-- with the shipped pipeline on EVERY instance of a bounded family
-- (tens of thousands of cases). This is the Haskell analogue of the
-- Rust side's Myers-vs-brute-force-LCS minimality proof, and the
-- reason §7.3's property testing stays deterministic here: a seeded
-- generator SAMPLES a space this small; enumeration covers it.
module Reference (equivalence) where

import CE.FourClass.Cost (destFloor)
import CE.FourClass.Provenance (classify)
import CE.FourClass.Wire
import Data.Aeson (Value (Null))
import qualified Data.Set as S
import Data.Word (Word64)

type Run = [(Int, Word64)]

-- | Instance family: pair 0 removes (1-2 runs), pair 1 adds (1-2
-- runs), pair 2 optionally removes one extra line (the
-- source-attribution probe). Two sub-families trade alphabet size
-- against length so both depth and hash diversity get full coverage.
instances :: [[Pair]]
instances =
  [ [Pair 0 rem0 [] [], Pair 1 [] add1 []] ++ probe
  | (rem0, add1) <- sides
  , probe <- [] : [[Pair 2 [[(30, h)]] [] []] | h <- [1, 2, 3]]
  ]
 where
  sides :: [([Run], [Run])]
  sides = both 2 4 ++ both 3 3
  both :: Word64 -> Int -> [([Run], [Run])]
  both alphabet maxLen =
    [(r, a) | r <- family alphabet maxLen, a <- family alphabet maxLen]
  family :: Word64 -> Int -> [[Run]]
  family alphabet maxLen =
    [ runsAt hs k
    | n <- [0 .. maxLen]
    , hs <- sequencesOf alphabet n
    , k <- if n >= 2 then 0 : [1 .. n - 1] else [0]
    ]
  sequencesOf :: Word64 -> Int -> [[Word64]]
  sequencesOf alphabet n = mapM (const [1 .. alphabet]) [1 .. n]
  runsAt :: [Word64] -> Int -> [Run]
  runsAt hs 0 = [zip [1 ..] hs | not (null hs)]
  runsAt hs k =
    let (a, b) = splitAt k hs
     in [zip [1 ..] a, zip [k + 3 ..] b] -- gap of 2 between runs

-- | Every maximal (not extendable in either direction) common
-- contiguous hash segment of length >= destFloor between a removed
-- run and an added run of different pairs.
refBlocks :: [Pair] -> S.Set (Int, [Int], Int, [Int])
refBlocks ps =
  S.fromList
    [ (pIdx p, map fst segR, pIdx q, map fst segA)
    | p <- ps
    , q <- ps
    , pIdx p /= pIdx q
    , r <- pRem p
    , a <- pAdd q
    , x <- [0 .. length r - 1]
    , y <- [0 .. length a - 1]
    , n <- [destFloor .. min (length r - x) (length a - y)]
    , let segR = take n (drop x r)
    , let segA = take n (drop y a)
    , map snd segR == map snd segA
    , not (equalAt r a (x - 1) (y - 1)) -- maximal to the left
    , not (equalAt r a (x + n) (y + n)) -- maximal to the right
    ]
 where
  equalAt r a i j =
    i >= 0 && j >= 0 && i < length r && j < length a && snd (r !! i) == snd (a !! j)

-- | The set-form phases over the reference blocks.
refMarks :: [Pair] -> S.Set (Int, [Int], Int, [Int]) -> (S.Set (Int, Int), S.Set (Int, Int))
refMarks ps blocks = (outs `S.union` attributed, ins `S.union` extended)
 where
  outs = S.fromList [(p, l) | (p, ls, _, _) <- S.toList blocks, l <- ls]
  ins = S.fromList [(q, l) | (_, _, q, ls) <- S.toList blocks, l <- ls]
  edges = S.fromList [(p, q) | (p, _, q, _) <- S.toList blocks]
  remHashes p = S.fromList [h | pr <- ps, pIdx pr == p, (_, h) <- concat (pRem pr)]
  extended =
    S.fromList
      [ (pIdx q, l)
      | q <- ps
      , run <- pAdd q
      , any (\(l', _) -> (pIdx q, l') `S.member` ins) run
      , (l, h) <- run
      , (pIdx q, l) `S.notMember` ins
      , any (\(src, dst) -> dst == pIdx q && h `S.member` remHashes src) (S.toList edges)
      ]
  inHashes q =
    S.fromList
      [h | pr <- ps, pIdx pr == q, (l, h) <- concat (pAdd pr), (q, l) `S.member` allIns]
  allIns = ins `S.union` extended
  attributed =
    S.fromList
      [ (pIdx p, l)
      | p <- ps
      , (l, h) <- concat (pRem p)
      , any (\q -> pIdx q /= pIdx p && h `S.member` inHashes (pIdx q)) ps
      ]

-- | Production output as comparable sets.
shipped :: [Pair] -> (S.Set (Int, [Int], Int, [Int]), S.Set (Int, Int), S.Set (Int, Int))
shipped ps = (blocks, outs, ins)
 where
  r = classify (Request Null ps)
  blocks = S.fromList [(bFromPair b, bFromLines b, bToPair b, bToLines b) | b <- resBlocks r]
  outs = S.fromList [(i, l) | (i, ls, _) <- resMoved r, l <- ls]
  ins = S.fromList [(i, l) | (i, _, ls) <- resMoved r, l <- ls]

equivalence :: IO Bool
equivalence = do
  let bad = take 3 [ps | ps <- instances, mismatch ps]
      total = length instances
  case bad of
    [] -> do
      putStrLn ("ok   reference equivalence on " <> show total <> " exhaustive instances")
      pure True
    (ps : _) -> do
      putStrLn ("FAIL reference equivalence, first mismatch on " <> show (dump ps))
      pure False
 where
  mismatch ps =
    let (blocks, outs, ins) = shipped ps
        rb = refBlocks ps
        (ro, ri) = refMarks ps rb
     in blocks /= rb || outs /= ro || ins /= ri
  dump = map (\p -> (pIdx p, pRem p, pAdd p))
