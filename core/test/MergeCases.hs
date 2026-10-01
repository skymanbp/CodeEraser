-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The merge battery's hand-written judgments (plan v2.31 step 6) as
-- text tables, read with FlowCases' header grammar: a case is a
-- header `# <leg name>` (a judgment) or `! <message>` (a refusal,
-- module MergeRefusals) and its rows — `g g family helper` a group,
-- `m g m unit lines fileIndeg` a member, `t lab… | lld… | leaf… |
-- slot… | own… | text…` a member's tree (`|`-separated sections in
-- member order; a section `-` leaves that key out; four sections leave
-- own at 0 and text at ReferenceTreeGen's model), `= s …` an expected
-- suggestion row and `= h …` an expected hole row (six columns, the
-- last two a member's first and last root of the hole). Text, not
-- lists of row literals: those normalise to one token run the clone
-- gate names.
module MergeCases (Case (..), casesOf, judgments, mergeRequest, modelTree, treeValue) where

import Data.Aeson (Value, object, toJSON, (.=))
import qualified Data.Aeson.Key as Key
import FlowCases (sections)
import ReferenceTreeGen (textColumn)
import WireHarness (setKey, tabledRequest)

data Case = Case {caseName :: String, caseRequest :: Value, caseRows :: ([[Integer]], [[Integer]]), caseRefusal :: Maybe String}

judgments :: [Case]
judgments = casesOf (exact <> positions <> sevenParams <> noSavings <> near <> nearMixed <> several)

casesOf :: String -> [Case]
casesOf = map (uncurry caseOf) . sections

caseOf :: String -> [String] -> Case
caseOf header body = Case name request (expect "s", expect "h") refusal
 where
  name = drop 2 header
  refusal = if take 1 header == "!" then Just name else Nothing
  request = mergeRequest (table "g") (table "m") [tree (drop 1 l) | 't' : l <- body]
  table tag = [map read rest | tag' : rest <- map words body, tag' == tag]
  expect tag = [map read rest | "=" : tag' : rest <- map words body, tag' == tag]
  tree l = case map words (splitOn l) of
    [lab, lld, leaf, slot] -> modelTree (map read lab) (map read lld) (column leaf) (column slot)
    [lab, lld, leaf, slot, own, text] -> treeValue (map read lab) (map read lld) (map column [leaf, slot, own, text])
    _ -> toJSON ([] :: [Int])
  column ["-"] = Nothing
  column ws = Just (map read ws)
  splitOn l = case break (== '|') l of
    (piece, _ : rest) -> piece : splitOn rest
    (piece, []) -> [piece]

-- | A merge.request with its groups, members and trees.
mergeRequest :: [[Integer]] -> [[Integer]] -> [Value] -> Value
mergeRequest groups members trees =
  setKey "trees" (toJSON trees) (tabledRequest "7.0.0" "merge.request" [("groups", groups), ("members", members)])

-- | One wire tree: lab, lld, then the leaf, slot, own and text
-- columns in that order; an absent column leaves its key out.
treeValue :: [Integer] -> [Integer] -> [Maybe [Integer]] -> Value
treeValue lab lld columns =
  object (["lab" .= lab, "lld" .= lld] <> [Key.fromString k .= v | (k, Just v) <- zip ["leaf", "slot", "own", "text"] columns])

-- | A wire tree whose own column is every node 0 (no operator differs)
-- and whose text column is ReferenceTreeGen's model of a token stream.
modelTree :: [Integer] -> [Integer] -> Maybe [Integer] -> Maybe [Integer] -> Value
modelTree lab lld leaf slot = treeValue lab lld [leaf, slot, Just (map (const 0) lab), Just text]
 where
  ints = map fromInteger
  text = textColumn (ints lab) (ints lld) (maybe (map (const 0) lab) id leaf)

-- | T1/T2 groups: no hole, holes sharing a vector, distinct vectors.
exact :: String
exact =
  "# a T1 pair with no hole: no parameter, the pair saves its lines less the kept member and two calls\n\
  \g 0 0 0\n\
  \m 0 0 0 10 1\n\
  \m 0 1 1 10 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \= s 0 0 0 8 1 0\n\
  \# a T2 group of three whose two holes share one value vector takes one parameter; the tied in-degree keeps the least m\n\
  \g 0 0 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 6 2\n\
  \m 0 2 2 6 2\n\
  \t 1 1 2 9 | 0 1 2 0 | 21 21 5 0 | 1 1 1 4\n\
  \t 1 1 2 9 | 0 1 2 0 | 31 31 5 0 | 1 1 1 4\n\
  \t 1 1 2 9 | 0 1 2 0 | 41 41 5 0 | 1 1 1 4\n\
  \= s 0 1 1 9 1 0\n\
  \= h 0 0 0 0 0 0\n\
  \= h 0 0 0 1 0 0\n\
  \= h 0 0 0 2 0 0\n\
  \= h 0 1 0 0 1 1\n\
  \= h 0 1 0 1 1 1\n\
  \= h 0 1 0 2 1 1\n\
  \# two holes of different vectors are two parameters, a leaf kind may differ, and a declared-name hole is feasible\n\
  \g 0 0 0\n\
  \m 0 0 0 8 0\n\
  \m 0 1 1 8 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 3 1 4\n\
  \t 1 3 9 | 0 1 0 | 21 22 0 | 3 1 4\n\
  \= s 0 2 0 6 1 0\n\
  \= h 0 0 0 0 0 0\n\
  \= h 0 0 0 1 0 0\n\
  \= h 0 1 1 0 1 1\n\
  \= h 0 1 1 1 1 1\n"

-- | Position classes: statement, other, type first in order.
positions :: String
positions =
  "# a hole at a statement position is no parameter: reason 1\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 0 4\n\
  \t 1 2 9 | 0 1 0 | 11 13 0 | 1 0 4\n\
  \= s 0 1 0 3 0 1\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 1 1\n\
  \# a hole at an other position is no parameter either: reason 1\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 4 4\n\
  \t 1 2 9 | 0 1 0 | 11 13 0 | 1 4 4\n\
  \= s 0 1 0 3 0 1\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 1 1\n\
  \# the first infeasible hole in hole order names the reason: a type position before a statement is reason 2\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 2 0 4\n\
  \t 1 2 9 | 0 1 0 | 21 22 0 | 2 0 4\n\
  \= s 0 2 0 3 0 2\n\
  \= h 0 0 0 0 0 0\n\
  \= h 0 0 0 1 0 0\n\
  \= h 0 1 1 0 1 1\n\
  \= h 0 1 1 1 1 1\n"

-- | Seven parameters, one past the ceiling.
sevenParams :: String
sevenParams =
  "# seven feasible holes of seven vectors are more parameters than six: reason 4\n\
  \g 0 0 0\n\
  \m 0 0 0 12 0\n\
  \m 0 1 1 12 0\n\
  \t 1 1 1 1 1 1 1 9 | 0 1 2 3 4 5 6 0 | 11 12 13 14 15 16 17 0 | 1 1 1 1 1 1 1 4\n\
  \t 1 1 1 1 1 1 1 9 | 0 1 2 3 4 5 6 0 | 21 22 23 24 25 26 27 0 | 1 1 1 1 1 1 1 4\n\
  \= s 0 7 0 10 0 4\n\
  \= h 0 0 0 0 0 0\n\
  \= h 0 0 0 1 0 0\n\
  \= h 0 1 1 0 1 1\n\
  \= h 0 1 1 1 1 1\n\
  \= h 0 2 2 0 2 2\n\
  \= h 0 2 2 1 2 2\n\
  \= h 0 3 3 0 3 3\n\
  \= h 0 3 3 1 3 3\n\
  \= h 0 4 4 0 4 4\n\
  \= h 0 4 4 1 4 4\n\
  \= h 0 5 5 0 5 5\n\
  \= h 0 5 5 1 5 5\n\
  \= h 0 6 6 0 6 6\n\
  \= h 0 6 6 1 6 6\n"

-- | A parameterisation every hole admits that saves no line: one-line
-- members pay the skeleton and a call each — a merge no one would take.
noSavings :: String
noSavings =
  "# a pair of one-line members with one feasible hole saves 2 - (1 + 2) = -1 lines: reason 5\n\
  \g 0 0 0\n\
  \m 0 0 0 1 0\n\
  \m 0 1 1 1 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 13 0 | 1 1 4\n\
  \= s 0 1 0 -1 0 5\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 1 1\n"

-- | T3 pairs: a relabel, an inserted subtree, two inserted sibling
-- subtrees (the gap's first and last root).
near :: String
near =
  "# a T3 relabel is a leaf hole; the skeleton is the kept member's lines in its kept-pair share, rounded up\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 7 1\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 3 9 | 0 1 0 | 11 13 0 | 1 1 4\n\
  \= s 0 1 1 4 1 0\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 1 1\n\
  \# a subtree only one side has is a gap hole, empty on member 0's side: a structural difference at an expression position, reason 1\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 7 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 5 2 9 | 0 1 2 0 | 11 15 12 0 | 1 1 1 4\n\
  \= s 0 1 0 5 0 1\n\
  \= h 0 0 0 0 -1 -1\n\
  \= h 0 0 0 1 1 1\n\
  \# a gap of two sibling subtrees answers the side's first and last root, the empty side -1 -1\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 8 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 5 6 2 9 | 0 1 2 3 0 | 11 15 16 12 0 | 1 1 1 1 4\n\
  \= s 0 1 0 6 0 1\n\
  \= h 0 0 0 0 -1 -1\n\
  \= h 0 0 0 1 1 2\n"

-- | T3 pairs: a leaf hole and a gap together, a gap across a
-- statement, roots the mapping leaves apart.
nearMixed :: String
nearMixed =
  "# a gap empty on member 0's side sits before the kept child it precedes, so it is the first hole and names the reason: position\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 7 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 5 2 9 | 0 1 2 0 | 11 15 13 0 | 1 1 1 4\n\
  \= s 0 2 0 5 0 1\n\
  \= h 0 0 0 0 -1 -1\n\
  \= h 0 0 0 1 1 1\n\
  \= h 0 1 1 0 1 1\n\
  \= h 0 1 1 1 2 2\n\
  \# a gap whose forest holds a statement crosses statements: reason 3, though its first root is an expression\n\
  \g 0 1 0\n\
  \m 0 0 0 6 0\n\
  \m 0 1 1 8 0\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \t 1 7 6 2 9 | 0 1 1 3 0 | 11 17 16 12 0 | 1 0 1 1 4\n\
  \= s 0 1 0 6 0 3\n\
  \= h 0 0 0 0 -1 -1\n\
  \= h 0 0 0 1 2 2\n\
  \# roots the mapping leaves apart have no skeleton: the whole pair is one gap, a structural difference, and nothing is kept\n\
  \g 0 1 0\n\
  \m 0 0 0 3 0\n\
  \m 0 1 1 2 0\n\
  \t 5 1 | 0 0 | 15 0 | 1 1\n\
  \t 5 | 0 | 15 | 1\n\
  \= s 0 1 0 3 0 1\n\
  \= h 0 0 0 0 1 1\n\
  \= h 0 0 0 1 0 0\n"

-- | Two groups in one request answer in g order; savings may be
-- negative and are reported as they fall, and a group that saves no
-- line is not feasible (reason 5).
several :: String
several =
  "# two groups answer in g order, and a pair of one-line members saves minus one: reason 5\n\
  \g 0 0 0\n\
  \g 5 0 0\n\
  \m 0 0 0 1 0\n\
  \m 0 1 1 1 0\n\
  \m 5 0 2 4 0\n\
  \m 5 1 3 4 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \= s 0 0 0 -1 0 5\n\
  \= s 5 0 0 2 1 0\n"
