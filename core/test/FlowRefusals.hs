-- | Every refusal the flow contract can name (plan v2.31 step 3), in
-- the text form FlowCases reads: a header `! <message>` and the
-- smallest rows that trip it — the row shapes and ranges
-- (CE.Flow.Contract), the neighbour checks of the tree builder
-- (CE.Flow.Tree), the shape of the tree (CE.Flow.Shape). Forty-two
-- cases, one per refusal a request can reach; the tree builder's
-- three totality clauses (`malformed stmt / var / use`, a row of the
-- wrong width) sit behind the contract's row-shape check and have no
-- case, since no request gets past that check to them.
module FlowRefusals (refusals) where

import FlowCases (Case, casesOf)

refusals :: [Case]
refusals = casesOf (rowShapes <> treeNeighbours <> treeShape)

-- | The unit table, then each table's row shape and ranges.
rowShapes :: String
rowShapes =
  "! unit 0: malformed unit (need [u,lang,params])\n\
  \u 0 3\n\
  \! unit 0: negative unit value\n\
  \u 0 -1 0\n\
  \! unit 1: not strictly ascending\n\
  \u 0 3 0\n\
  \u 0 3 0\n\
  \! unit 0: params disagree with the var table\n\
  \u 0 3 1\n\
  \! stmt 0: malformed stmt (need [u,seq,parent,kind,flags,aux])\n\
  \s 0 0 -1 1 0\n\
  \! stmt 0: negative stmt value\n\
  \s 0 0 -2 1 0 0\n\
  \! stmt 0: unknown kind\n\
  \s 0 0 -1 16 0 0\n\
  \! stmt 0: flags outside five bits\n\
  \s 0 0 -1 1 32 0\n\
  \! var 0: malformed var (need [u,v,declSeq,flags])\n\
  \v 0 0 -1\n\
  \! var 0: negative var value\n\
  \v 0 0 -2 1\n\
  \! var 0: flags outside four bits\n\
  \v 0 0 -1 17\n\
  \! use 0: malformed use (need [u,seq,v,mode])\n\
  \x 0 0 0\n\
  \! use 0: negative use value\n\
  \x 0 -1 0 0\n\
  \! use 0: unknown mode\n\
  \x 0 0 0 3\n"

-- | The checks that need a row's neighbours: units declared and
-- ascending, seqs and ids consecutive, parents earlier, declaring
-- statements inside the unit, parameters at no statement, accesses
-- ordered and naming what exists.
treeNeighbours :: String
treeNeighbours =
  "! stmt 0: unit not declared\n\
  \s 1 0 -1 1 0 0\n\
  \! stmt 1: not strictly ascending\n\
  \u 0 3 0\n\
  \u 1 3 0\n\
  \s 1 0 -1 1 0 0\n\
  \s 0 0 -1 1 0 0\n\
  \! stmt 1: seq not consecutive\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 2 -1 1 0 0\n\
  \! stmt 0: parent not before\n\
  \s 0 0 0 1 0 0\n\
  \! var 0: unit not declared\n\
  \v 1 0 -1 1\n\
  \! var 1: not strictly ascending\n\
  \u 0 3 1\n\
  \u 1 3 1\n\
  \v 1 0 -1 1\n\
  \v 0 0 -1 1\n\
  \! var 1: v not consecutive\n\
  \u 0 3 2\n\
  \v 0 0 -1 1\n\
  \v 0 2 -1 1\n\
  \! var 0: declSeq outside the unit\n\
  \v 0 0 0 0\n\
  \! var 0: param declared at a statement\n\
  \u 0 3 1\n\
  \s 0 0 -1 1 0 0\n\
  \v 0 0 0 1\n\
  \! var 0: local declared at no statement\n\
  \v 0 0 -1 0\n\
  \! use 0: unit not declared\n\
  \x 1 0 0 0\n\
  \! use 1: out of order\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 1 -1 1 0 0\n\
  \v 0 0 0 0\n\
  \x 0 1 0 0\n\
  \x 0 0 0 0\n\
  \! use 0: seq outside the unit\n\
  \x 0 0 0 0\n\
  \! use 0: var not declared\n\
  \s 0 0 -1 1 0 0\n\
  \x 0 0 0 0\n"

-- | The tree's shape: flags and aux on the right kinds, jumps with
-- targets the graph can honour, an if's branches, a switch's arms
-- and default, where a case and the handlers may sit, a try's
-- handler order, leaves without children.
treeShape :: String
treeShape =
  "! stmt 0: flag on the wrong kind\n\
  \s 0 0 -1 1 1 0\n\
  \! stmt 0: aux on a non-jump\n\
  \s 0 0 -1 1 0 3\n\
  \! stmt 0: break target is not an enclosing loop or switch\n\
  \s 0 0 -1 11 0 0\n\
  \! stmt 2: continue target is not an enclosing loop\n\
  \s 0 0 -1 4 0 0\n\
  \s 0 1 0 5 0 0\n\
  \s 0 2 1 12 0 0\n\
  \! stmt 0: goto target is not a label\n\
  \s 0 0 -1 13 0 0\n\
  \! stmt 0: if branches disagree with has_else\n\
  \s 0 0 -1 2 0 0\n\
  \! stmt 0: switch child is not a case\n\
  \s 0 0 -1 4 0 0\n\
  \s 0 1 0 1 0 0\n\
  \! stmt 0: default on an empty switch\n\
  \s 0 0 -1 4 1 0\n\
  \! stmt 0: case outside a switch\n\
  \s 0 0 -1 5 0 0\n\
  \! stmt 0: handler outside a try\n\
  \s 0 0 -1 7 0 0\n\
  \! stmt 0: body statement after a handler\n\
  \s 0 0 -1 6 0 0\n\
  \s 0 1 0 7 0 0\n\
  \s 0 2 0 1 0 0\n\
  \! stmt 0: catch after finally\n\
  \s 0 0 -1 6 0 0\n\
  \s 0 1 0 8 0 0\n\
  \s 0 2 0 7 0 0\n\
  \! stmt 0: second finally\n\
  \s 0 0 -1 6 0 0\n\
  \s 0 1 0 8 0 0\n\
  \s 0 2 0 8 0 0\n\
  \! stmt 0: children under a leaf statement\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 1 0 1 0 0\n"
