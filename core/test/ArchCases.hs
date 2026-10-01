-- | The arch battery's hand-written cases (plan v2.31 step 8) as text
-- tables, FlowCases' posture: a judgment case is a header line
-- `# <leg name>`, its request rows — `f F D lines` a file, `d D
-- parent` a directory, `e F G w` a file edge, `p F D w` a package
-- edge, `x F` one focus file — and the answer rows it expects, one
-- letter per reply table: `= L D level` layers, `= K D E w exact`
-- cuts, `= C F cluster` clusters, `= M F D` misplaced, `= I F depth`
-- impact, `= T D in out instability` metrics. A case asserts exactly
-- the tables whose letter it names; a bare `= X` line names table X
-- with no row, so an empty answer is asserted too. A refusal case
-- (module ArchRefusals) is a header `! <message>` and the smallest
-- rows that trip it. A case without a `d` row gets the root alone.
module ArchCases (Case (..), casesOf, judgments) where

import Data.Aeson (Value, toJSON)
import Data.List (groupBy)
import WireHarness (setKey, tabledRequest)

data Case = Case {caseName :: String, caseRequest :: Value, caseExpect :: [(String, [[Integer]])], caseRefusal :: Maybe String}

judgments :: [Case]
judgments = casesOf (cycles <> components <> groups <> reaches)

-- | A header line opens a case and the lines up to the next header
-- are its rows: a group runs from a header while no header follows.
casesOf :: String -> [Case]
casesOf text = [caseOf header body | header : body <- groupBy (\_ l -> take 1 l `notElem` ["#", "!"]) (lines text)]

caseOf :: String -> [String] -> Case
caseOf header body = Case name request expect refusal
 where
  name = drop 2 header
  refusal = if take 1 header == "!" then Just name else Nothing
  tables = [("files", table "f"), ("dirs", dirs), ("edges", table "e"), ("pkgEdges", table "p")]
  request = setKey "focus" (toJSON (concat (table "x"))) (tabledRequest "7.0.0" "arch.request" tables)
  table :: String -> [[Integer]]
  table tag = [map read rest | (t : rest) <- map words body, t == tag]
  dirs = case table "d" of
    [] -> [[0, -1]]
    rows -> rows
  expect = [(key, [map read rest | ("=" : l : rest) <- map words body, l == letter, not (null rest)]) | (letter, key) <- replies, named letter]
  named letter = or [l == letter | ("=" : l : _) <- map words body]
  replies = [("L", "layers"), ("K", "cuts"), ("C", "clusters"), ("M", "misplaced"), ("I", "impact"), ("T", "metrics")]

-- | The feedback arc set: the lighter arc of a two-cycle, a weight tie
-- broken by fewer arcs, an arc-count tie broken by (from, to).
cycles :: String
cycles =
  "# two directories referencing each other: the lighter arc is cut exactly and the layers are 1 over 0\n\
  \d 0 -1\nd 1 0\nd 2 0\n\
  \f 0 1 10\nf 1 2 10\n\
  \e 0 1 3\ne 1 0 1\n\
  \= K 2 1 1 1\n\
  \= L 0 0\n= L 1 1\n= L 2 0\n\
  \# a three-directory tie on weight keeps the cut of fewer arcs\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\n\
  \e 0 1 2\ne 1 0 1\ne 1 2 1\ne 2 0 1\n\
  \= K 1 2 2 1\n\
  \= L 0 0\n= L 1 0\n= L 2 2\n= L 3 1\n\
  \# a three-directory cycle of equal weights cuts the arc least by (from, to)\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\n\
  \e 0 1 1\ne 1 2 1\ne 2 0 1\n\
  \= K 1 2 1 1\n\
  \= L 0 0\n= L 1 0\n= L 2 2\n= L 3 1\n\
  \# a package edge folds into the directory graph and closes a cycle\n\
  \d 0 -1\nd 1 0\nd 2 0\n\
  \f 0 1 1\nf 1 2 1\n\
  \e 0 1 2\np 1 1 1\n\
  \= K 2 1 1 1\n\
  \= L 0 0\n= L 1 1\n= L 2 0\n\
  \= T 0 0 0 -1\n= T 1 1 1 500\n= T 2 1 1 500\n"

-- | Components: two cut on their own, one of thirteen arcs on five
-- directories cut exactly (the line is fourteen vertices, not an arc
-- count), one of fifteen directories on the greedy road, references
-- that stay inside a directory.
components :: String
components =
  "# two independent components are each cut exactly\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\nd 4 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\nf 3 4 1\n\
  \e 0 1 1\ne 1 0 2\ne 2 3 5\ne 3 2 4\n\
  \= K 1 2 1 1\n= K 4 3 4 1\n\
  \= L 0 0\n= L 1 0\n= L 2 1\n= L 3 1\n= L 4 0\n\
  \# a component of thirteen arcs on five directories is cut exactly\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\nd 4 0\nd 5 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\nf 3 4 1\nf 4 5 1\n\
  \e 0 1 1\ne 0 2 1\ne 0 3 1\ne 0 4 1\ne 1 2 1\ne 1 3 1\ne 1 4 1\n\
  \e 2 0 1\ne 2 3 1\ne 2 4 1\ne 3 1 1\ne 3 4 1\ne 4 0 1\n\
  \= K 3 1 1 1\n= K 4 2 1 1\n= K 5 1 1 1\n\
  \= L 0 0\n= L 1 4\n= L 2 3\n= L 3 2\n= L 4 1\n= L 5 0\n\
  \# a component of fifteen directories takes the greedy road and its one cut arc closes the ring\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\nd 4 0\nd 5 0\nd 6 0\nd 7 0\nd 8 0\nd 9 0\nd 10 0\nd 11 0\nd 12 0\nd 13 0\nd 14 0\nd 15 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\nf 3 4 1\nf 4 5 1\nf 5 6 1\nf 6 7 1\nf 7 8 1\nf 8 9 1\nf 9 10 1\nf 10 11 1\nf 11 12 1\nf 12 13 1\nf 13 14 1\nf 14 15 1\n\
  \e 0 1 1\ne 1 2 1\ne 2 3 1\ne 3 4 1\ne 4 5 1\ne 5 6 1\ne 6 7 1\ne 7 8 1\ne 8 9 1\ne 9 10 1\ne 10 11 1\ne 11 12 1\ne 12 13 1\ne 13 14 1\ne 14 0 1\n\
  \= K 15 1 1 0\n= M\n\
  \= L 0 0\n= L 1 14\n= L 2 13\n= L 3 12\n= L 4 11\n= L 5 10\n= L 6 9\n= L 7 8\n= L 8 7\n= L 9 6\n= L 10 5\n= L 11 4\n= L 12 3\n= L 13 2\n= L 14 1\n= L 15 0\n\
  \# references inside one directory, file or package, are no arc\n\
  \d 0 -1\nd 1 0\n\
  \f 0 1 1\nf 1 1 1\n\
  \e 0 1 1\ne 1 0 1\np 0 1 1\n\
  \= K\n\
  \= L 0 0\n= L 1 0\n\
  \= T 0 0 0 -1\n= T 1 0 0 -1\n\
  \= M\n"

-- | Clusters and the files outside their cluster's directory.
groups :: String
groups =
  "# two clear cliques make two clusters and name the one file outside its cluster's directory\n\
  \d 0 -1\nd 1 0\nd 2 0\n\
  \f 0 1 1\nf 1 1 1\nf 2 2 1\nf 3 2 1\nf 4 2 1\nf 5 2 1\n\
  \e 0 1 1\ne 0 2 1\ne 1 2 1\ne 2 3 1\ne 3 4 1\ne 3 5 1\ne 4 5 1\n\
  \= C 0 0\n= C 1 0\n= C 2 0\n= C 3 1\n= C 4 1\n= C 5 1\n\
  \= M 2 1\n\
  \# a file alone in its cluster is never misplaced\n\
  \d 0 -1\nd 1 0\nd 2 0\n\
  \f 0 1 1\nf 1 1 1\nf 2 2 1\n\
  \e 0 1 1\n\
  \= C 0 0\n= C 1 0\n= C 2 1\n\
  \= M\n\
  \# an isolated directory is level 0 with instability -1\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 2 1\n\
  \e 0 1 1\n\
  \= L 0 0\n= L 1 1\n= L 2 0\n= L 3 0\n\
  \= T 0 0 0 -1\n= T 1 0 1 1000\n= T 2 1 0 0\n= T 3 0 0 -1\n\
  \# fan-in and fan-out count distinct directories, before the cut\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 1 1\nf 2 2 1\nf 3 3 1\n\
  \e 0 2 5\ne 0 3 1\ne 1 2 1\ne 2 3 1\np 3 2 1\n\
  \= K 2 3 1 1\n\
  \= L 0 0\n= L 1 2\n= L 2 0\n= L 3 1\n\
  \= T 0 0 0 -1\n= T 1 0 2 1000\n= T 2 2 1 333\n= T 3 2 1 333\n"

-- | The impact, a directory tree past two levels, the default root.
reaches :: String
reaches =
  "# the impact of a focus reaches two hops, one through a package edge\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\nf 3 3 1\n\
  \e 1 0 1\np 2 2 1\nx 0\n\
  \= I 0 0\n= I 1 1\n= I 2 2\n\
  \= C 0 0\n= C 1 0\n= C 2 1\n= C 3 2\n\
  \= M\n\
  \# an empty focus answers an empty impact\n\
  \d 0 -1\nd 1 0\nd 2 0\nd 3 0\n\
  \f 0 1 1\nf 1 2 1\nf 2 3 1\nf 3 3 1\n\
  \e 1 0 1\np 2 2 1\n\
  \= I\n\
  \# a directory tree past two levels lists each parent before its child\n\
  \d 0 -1\nd 1 0\nd 2 1\nd 3 2\nd 4 1\n\
  \f 0 3 1\nf 1 4 1\n\
  \e 0 1 1\n\
  \= L 0 0\n= L 1 0\n= L 2 0\n= L 3 1\n= L 4 0\n\
  \= T 0 0 0 -1\n= T 1 0 0 -1\n= T 2 0 0 -1\n= T 3 0 1 1000\n= T 4 1 0 0\n\
  \# a package edge into a directory with no file of its own is still an arc\n\
  \d 0 -1\nd 1 0\nd 2 0\n\
  \f 0 1 1\n\
  \p 0 2 3\n\
  \= L 0 0\n= L 1 1\n= L 2 0\n\
  \= T 0 0 0 -1\n= T 1 0 1 1000\n= T 2 1 0 0\n"
