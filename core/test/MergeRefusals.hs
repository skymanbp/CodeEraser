-- | Every refusal the merge contract can name (plan v2.31 step 6), in
-- the text form MergeCases reads: a header `! <message>` and the
-- smallest rows that trip it — the group and member rows
-- (CE.Merge.Contract through CE.Wire's row and table checks), the
-- members against their groups, each group's member count, the tree
-- count, each tree (clone/1's shared shape contract, then the leaf,
-- slot, own and text columns) and a T1/T2 group that is not
-- isomorphic.
module MergeRefusals (refusals) where

import MergeCases (Case, casesOf)

refusals :: [Case]
refusals = casesOf (groupsAndMembers <> treeShapes <> ownText <> treeColumns)

-- | The group table, the member table, the two joined.
groupsAndMembers :: String
groupsAndMembers =
  "! group 0: malformed group (need [g,family,helper])\n\
  \g 0\n\
  \! group 0: negative group value\n\
  \g -1 0 0\n\
  \! group 0: unknown family\n\
  \g 0 2 0\n\
  \! group 1: not strictly ascending\n\
  \g 3 0 0\n\
  \g 3 1 0\n\
  \! member 0: malformed member (need [g,m,unit,lines,fileIndeg])\n\
  \g 0 0 0\n\
  \m 0 0 0 5\n\
  \! member 0: negative member value\n\
  \g 0 0 0\n\
  \m 0 0 0 -5 0\n\
  \! member 1: not strictly ascending\n\
  \g 0 0 0\n\
  \m 0 1 1 5 0\n\
  \m 0 0 0 5 0\n\
  \! member 0: unknown group\n\
  \g 0 0 0\n\
  \m 1 0 0 5 0\n\
  \! member 1: m not consecutive\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 2 1 5 0\n\
  \! member 0: m not consecutive\n\
  \g 0 0 0\n\
  \m 0 1 0 5 0\n\
  \! group 0: one member\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \! group 0: a T3 pair needs exactly two members\n\
  \g 0 1 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \m 0 2 2 5 0\n\
  \! group 1: no members\n\
  \g 0 0 0\n\
  \g 1 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n"

-- | The trees: their count and clone/1's shape contract.
treeShapes :: String
treeShapes =
  "! trees: 1 trees for 2 members\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 0: empty tree\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t  |  |  | \n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 1: lab/lld length mismatch\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 | 11 0 | 1 4\n\
  \! tree 0: leaf length mismatch\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n"

-- | The own and text columns (merge generation 2, ruling R1).
ownText :: String
ownText =
  "! tree 0: own missing\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | - | 11 21\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 1: text missing\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | 0 0 | -\n\
  \! tree 0: own length mismatch\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | 0 | 11 21\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 0: text length mismatch\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | 0 0 | 11\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 0: node 1: negative own\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | 0 -3 | 11 21\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 1: node 0: negative text\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4 | 0 0 | -11 21\n"

-- | The leaf and slot columns, then isomorphism.
treeColumns :: String
treeColumns =
  "! tree 0: leaf missing\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | - | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 1: slot missing\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 9 | 0 0 | 11 0 | -\n\
  \! tree 0: slot length mismatch\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! tree 0: node 1: unknown slot\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 5\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \! group 0: members are not isomorphic\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 2 9 | 0 1 0 | 11 12 0 | 1 1 4\n\
  \! group 0: members are not isomorphic\n\
  \g 0 0 0\n\
  \m 0 0 0 5 0\n\
  \m 0 1 1 5 0\n\
  \t 1 9 | 0 0 | 11 0 | 1 4\n\
  \t 1 8 | 0 0 | 11 0 | 1 4\n"
