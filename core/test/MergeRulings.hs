-- | The merge battery's cases for the second generation's rulings
-- (plan v2.31 steps 6-7, merge generation 2; design booklet §6.2–§6.3
-- and §13), in MergeCases' text form — each ruling a case and its
-- reverse probe, the trees spelling their own and text columns where
-- the ruling reads them: R2 an operator that differs is a hole of class
-- other (the same operator opens none); R3 a member name under an
-- expression widens to the expression (a target root does not), and a
-- whitespace-only difference is no hole (a token that differs is); R4 a
-- run one member lacks is structure (an argument that differs in value
-- is not); R5 a gap sits in member 0's postorder before the kept child
-- it precedes; R6 a fragment's helper lines enter the savings; R7 one
-- parameter per distinct text vector, whatever the places' kinds.
module MergeRulings (rulings) where

import MergeCases (Case, casesOf)

rulings :: [Case]
rulings = casesOf (operators <> widening <> quietAndTexts <> gapsAndOrder <> helpers)

-- | Ruling R2: `a < b` against `a > b`, and against itself.
operators :: String
operators =
  "# R2: an operator that differs is a hole of class other at its node, never a parameter: reason 1\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 1 7 9 | 0 1 0 0 | 11 12 0 0 | 1 1 1 4 | 0 0 61 0 | 11 12 501 601\n\
  \t 1 1 7 9 | 0 1 0 0 | 11 12 0 0 | 1 1 1 4 | 0 0 62 0 | 11 12 502 602\n\
  \= s 0 1 0 3 0 1\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 2 2\n\
  \# R2 reverse: the same operator on both sides opens no hole\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 1 7 9 | 0 1 0 0 | 11 12 0 0 | 1 1 1 4 | 0 0 61 0 | 11 12 501 601\n\
  \t 1 1 7 9 | 0 1 0 0 | 11 12 0 0 | 1 1 1 4 | 0 0 61 0 | 11 12 501 601\n\
  \= s 0 0 0 3 1 0\n"

-- | Ruling R3: `x.foo` against `x.bar` as a T1/T2 pair and as a T3
-- pair, and the same with the member a target root (class other).
widening :: String
widening =
  "# R3: a member name of class other under an expression widens to the expression, one feasible parameter whose texts are the expressions'\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 41 0 0 | 1 4 1 4 | 0 0 81 0 | 31 41 501 601\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 42 0 0 | 1 4 1 4 | 0 0 81 0 | 31 42 502 602\n\
  \= s 0 1 0 3 1 0\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 2 2\n\
  \# R3 in a T3 pair: the kept expression pair over a relabelled member name is one widened hole\n\
  \g 0 1 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 41 0 0 | 1 4 1 4 | 0 0 81 0 | 31 41 501 601\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 42 0 0 | 1 4 1 4 | 0 0 81 0 | 31 42 502 602\n\
  \= s 0 1 0 3 1 0\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 2 2\n\
  \# R3 reverse: under a parent of class other (an assignment's target root) the member name stays where it is: reason 1\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 41 0 0 | 1 4 4 4 | 0 0 81 0 | 31 41 501 601\n\
  \t 1 5 6 9 | 0 1 0 0 | 31 42 0 0 | 1 4 4 4 | 0 0 81 0 | 31 42 502 602\n\
  \= s 0 1 0 3 0 1\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 1 1\n"

-- | Rulings R3 (a quiet hole) and R7 (texts, not kinds).
quietAndTexts :: String
quietAndTexts =
  "# R3: `()` against `( )` differs in its bytes only, not its tokens: the hole is quiet, no parameter, feasible\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 3 2 9 | 0 1 0 0 | 21 61 0 0 | 1 4 1 4 | 0 71 0 0 | 21 71 901 1001\n\
  \t 1 3 2 9 | 0 1 0 0 | 21 62 0 0 | 1 4 1 4 | 0 71 0 0 | 21 71 901 1001\n\
  \= s 0 0 0 3 1 0\n\
  \# R3 reverse: a token that differs there is a hole, widened to the call\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 3 2 9 | 0 1 0 0 | 21 61 0 0 | 1 4 1 4 | 0 71 0 0 | 21 71 901 1001\n\
  \t 1 3 2 9 | 0 1 0 0 | 21 63 0 0 | 1 4 1 4 | 0 73 0 0 | 21 73 902 1002\n\
  \= s 0 1 0 3 1 0\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 2 2\n\
  \# R7: two places of different kinds with the same texts on every member are one parameter\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 2 9 | 0 1 0 | 51 51 0 | 3 1 4 | 0 0 0 | 51 51 601\n\
  \t 1 2 9 | 0 1 0 | 52 52 0 | 3 1 4 | 0 0 0 | 52 52 602\n\
  \= s 0 1 0 3 1 0\n\
  \= h 0 0 0 0 0 0\n\
  \= h 0 0 0 1 0 0\n\
  \= h 0 1 0 0 1 1\n\
  \= h 0 1 0 1 1 1\n"

-- | Rulings R4 (an argument one member lacks) and R5 (the order).
gapsAndOrder :: String
gapsAndOrder =
  "# R4: `f(a, b)` against `f(a)` - the argument member 1 lacks is a gap, a structural difference: reason 1, before the argument list's own-token hole\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 5 0\n\
  \t 1 1 4 3 2 9 | 0 1 2 1 0 0 | 21 22 23 0 0 0 | 1 1 1 4 1 4 | 0 0 0 71 0 0 | 21 22 23 801 901 1001\n\
  \t 1 1 3 2 9 | 0 1 1 0 0 | 21 22 0 0 0 | 1 1 4 1 4 | 0 0 72 0 0 | 21 22 802 902 1002\n\
  \= s 0 2 0 4 0 1\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 -1 -1\n\
  \= h 0 1 1 0 3 3\n\
  \= h 0 1 1 1 2 2\n\
  \# R4 reverse: `f(a, b)` against `f(a, c)` - an argument that differs in value is one feasible expression hole\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 6 0\n\
  \t 1 1 4 3 2 9 | 0 1 2 1 0 0 | 21 22 23 0 0 0 | 1 1 1 4 1 4 | 0 0 0 71 0 0 | 21 22 23 801 901 1001\n\
  \t 1 1 4 3 2 9 | 0 1 2 1 0 0 | 21 22 24 0 0 0 | 1 1 1 4 1 4 | 0 0 0 71 0 0 | 21 22 24 803 903 1003\n\
  \= s 0 1 0 4 1 0\n\
  \= h 0 0 0 0 2 2\n\
  \= h 0 0 0 1 2 2\n\
  \# R5: a statement member 1 adds between two kept statements is the first hole, ahead of a hole inside the later statement: reason 3\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 7 0\n\
  \t 5 1 6 9 | 0 1 1 0 | 11 21 0 0 | 0 4 0 4\n\
  \t 5 7 1 6 9 | 0 1 2 2 0 | 11 12 22 0 0 | 0 0 4 0 4\n\
  \= s 0 2 0 5 0 3\n\
  \= h 0 0 0 0 -1 -1\n\
  \= h 0 0 0 1 1 1\n\
  \= h 0 1 1 0 1 1\n\
  \= h 0 1 1 1 2 2\n"

-- | Ruling R6: a fragment's helper lines.
helpers :: String
helpers =
  "# R6: a fragment group's helper head and closing lines join the merged function's: two 10-line runs save 20 - (10 + 2 + 2) = 6\n\
  \g 0 0 2\n\
  \m 0 0 0 10 1\n\
  \m 0 1 1 10 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \= s 0 0 0 6 1 0\n\
  \# R6: the helper turns a one-line saving into none: two 3-line runs save 6 - (3 + 2 + 2) = -1, reason 5\n\
  \g 0 0 2\n\
  \m 0 0 0 3 0\n\
  \m 0 1 1 3 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \= s 0 0 0 -1 0 5\n\
  \# R6 reverse: the same whole units carry no helper and save 6 - (3 + 2) = 1\n\
  \g 0 0 0\n\
  \m 0 0 0 3 0\n\
  \m 0 1 1 3 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \= s 0 0 0 1 1 0\n"
