-- | The seeded generator behind ReferenceQuery (design booklet §4.7):
-- two hundred programs, each over its own seeded fact set, drawn from
-- seven parts any of which may be in or out — a transitive closure
-- over the import arcs (recursion), an aggregate per file (count,
-- min, max or sum, over one or two variables, over the arcs or over
-- the closure — a file with no arcs folds nothing), arithmetic on
-- line counts (every operator, a literal or a fact's integer on the
-- right, now and then a second operator; zeros in the facts and the
-- literals make divisions by zero), a comparison of two files' line
-- counts (every operator, `=` on a bound variable among them), a
-- negation of one of the parts before it, an aggregate over that
-- negation (a third or fourth stratum; it may fold nothing), and one
-- program in ten a negative cycle the checker must refuse — then a
-- query over each part and, one program in two, an assertion that
-- holds or is violated. The program's own predicates start at
-- p1200, clear of the prelude's. Spelled for `ReferenceQuery.toks`;
-- deterministic, the batteries' one LCG.
module ReferenceQueryGen (QCase (..), cases) where

import Data.Bits (shiftR, (.&.))
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S
import ReferenceFlowGen (G, S (..), lcgStep, rand, runG)
import ReferenceQuery (Db)

-- | One case: the shapes the program was built with, its spelling,
-- whether its last goal is an assertion, its fact set.
data QCase = QCase {qTags :: [String], qText :: String, qAssert :: Bool, qFacts :: Db}

cases :: [QCase]
cases = [runG (qcase n) (S (n * 104729 + 3) 0 0 0) | n <- [1 .. 200]]

pick :: [a] -> G a
pick xs = (xs !!) <$> rand (length xs)

chance :: Int -> G Bool
chance n = (== 0) <$> rand n

qcase :: Int -> G QCase
qcase n = do
  flags <- mapM chance [2, 2, 2, 2, 2, 2, 10, 2]
  let at = (flags !!)
      (reach, aggs, arith, cmps, neg, top, bad, asserting) = (at 0, at 1, at 2, at 3, at 4, at 5, at 6, at 7)
  let reach' = reach || not (or [aggs, arith, cmps, neg, bad])
  aggPart <- aggregate reach'
  arithPart <- arithmetic
  cmpPart <- comparison
  negPart <- negation reach' aggs
  topPart <- (\op -> "p1205 ( v1 ) :- v1 = " <> op <> " ( v2 : p1204 ( v0 ) , p8 ( v0 , v2 ) ) .") <$> pick aggOps
  k <- rand 5
  let parts =
        [ (reach', "p1200 ( v0 , v1 ) :- p9 ( v0 , v1 , _ , _ ) . p1200 ( v0 , v1 ) :- p1200 ( v0 , v2 ) , p9 ( v2 , v1 , _ , _ ) .", "?- p1200 ( v0 , v1 ) .")
        , (aggs, aggPart, "?- p1201 ( v0 , v1 ) .")
        , (arith, arithPart, "?- p1202 ( v0 , v1 ) .")
        , (cmps, cmpPart, "?- p1203 ( v0 , v1 ) .")
        , (neg, negPart, "?- p1204 ( v0 ) .")
        , (neg && top, topPart, "?- p1205 ( v0 ) .")
        , (bad, "p1206 ( v0 ) :- p1 ( v0 ) , not p1207 ( v0 ) . p1207 ( v0 ) :- p1206 ( v0 ) .", "?- p1206 ( v0 ) .")
        ]
      assertion = if neg then "assert p1208 ( v0 ) :- p1 ( v0 ) , not p1204 ( v0 ) ." else "assert p1208 ( v0 , v1 ) :- p8 ( v0 , v1 ) , v1 > i" <> show k <> " ."
      chosen = [(r, q) | (True, r, q) <- parts]
      text = unwords (map fst chosen <> map snd chosen <> [assertion | asserting])
  pure (QCase (["recursion" | reach'] <> ["negation" | neg]) text asserting (factSet (toInteger n)))

aggOps :: [String]
aggOps = ["count", "min", "max", "sum"]

aggregate :: Bool -> G String
aggregate reach = do
  op <- pick aggOps
  vars <- pick ["v3", "v3 , v2"]
  src <- pick (["p9 ( v0 , v2 , _ , _ )"] <> ["p1200 ( v0 , v2 )" | reach])
  pure ("p1201 ( v0 , v1 ) :- p1 ( v0 ) , v1 = " <> op <> " ( " <> vars <> " : " <> src <> " , p8 ( v2 , v3 ) ) .")

arithmetic :: G String
arithmetic = do
  op <- pick ["+", "-", "*", "/", "%"]
  right <- pick ["v3", "i0", "i1", "i2", "i3"]
  more <- chance 3
  op2 <- pick ["+", "-", "*", "/", "%"]
  k <- rand 4
  let tailExpr = if more then " " <> op2 <> " i" <> show k else ""
  pure ("p1202 ( v0 , v1 ) :- p8 ( v0 , v2 ) , p9 ( v0 , _ , _ , v3 ) , v1 = v2 " <> op <> " " <> right <> tailExpr <> " .")

comparison :: G String
comparison = (\op -> "p1203 ( v0 , v1 ) :- p8 ( v0 , v2 ) , p8 ( v1 , v3 ) , v2 " <> op <> " v3 , v0 < v1 .") <$> pick ["=", "!=", "<", "<=", ">", ">="]

negation :: Bool -> Bool -> G String
negation reach aggs = do
  src <- pick (["p7 ( v0 , s5 )"] <> ["p1200 ( v0 , _ )" | reach] <> ["p1200 ( _ , v0 )" | reach] <> ["p1201 ( v0 , i0 )" | aggs])
  pure ("p1204 ( v0 ) :- p1 ( v0 ) , not " <> src <> " .")

-- | A fact set over four nodes (codes 0..3): `file` for every node,
-- `ref` arcs drawn from the stream with a count of 0 to 3, `role`
-- entry bits, `lines` counts of 0 to 3, `in_dir` placements.
factSet :: Integer -> Db
factSet seed =
  IM.fromList
    [ (1, S.fromList [[n] | n <- nodes])
    , (9, S.fromList [[a, b, 0, bit (i + 20) .&. 3] | (i, (a, b)) <- arcs])
    , (7, S.fromList [[n, 5] | n <- nodes, odd (bit n)])
    , (8, S.fromList [[n, (bit (n + 4) `shiftR` 1) .&. 3] | n <- nodes])
    , (2, S.fromList [[n, bit (n + 8) .&. 1] | n <- nodes])
    ]
 where
  nodes = [0 .. 3]
  stream = drop 1 (iterate lcgStep seed)
  bit :: Integer -> Integer
  bit i = (stream !! fromInteger i) `shiftR` 16
  arcs = [(i, (a, b)) | (i, (a, b)) <- zip [12 ..] [(a, b) | a <- nodes, b <- nodes], bit i .&. 3 == 0]
