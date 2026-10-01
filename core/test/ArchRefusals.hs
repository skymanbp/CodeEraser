-- | Every refusal the arch contract can name (plan v2.31 step 8), in
-- the text form ArchCases reads: a header `! <message>` and the
-- smallest rows that trip it — each table's row width, identity,
-- ranges and order (CE.Arch.Contract), and the one ordering rule the
-- contract spells out: a directory row's own shape is named before a
-- file's directory range, since a malformed directory table has no
-- trustworthy length. Twenty-two cases, one per refusal.
module ArchRefusals (refusals) where

import ArchCases (Case, casesOf)

refusals :: [Case]
refusals = casesOf (filesAndDirs <> references)

-- | The files and the directory tree.
filesAndDirs :: String
filesAndDirs =
  "! file 0: malformed file (need [F,D,lines])\n\
  \f 0 0\n\
  \! file 1: F must be 1\n\
  \f 0 0 1\nf 2 0 1\n\
  \! file 0: negative lines\n\
  \f 0 0 -1\n\
  \! file 0: D out of range\n\
  \f 0 1 1\n\
  \! dir 0: malformed dir (need [D,parent])\n\
  \d 0\n\
  \! dir 1: D must be 1\n\
  \d 0 -1\nd 2 0\n\
  \! dir 0: root parent must be -1\n\
  \d 0 0\n\
  \! dir 1: second root\n\
  \d 0 -1\nd 1 -1\n\
  \! dir 1: parent out of range\n\
  \d 0 -1\nd 1 1\n\
  \! dir 0: root parent must be -1\n\
  \f 0 3 1\nd 0 2\n"

-- | The two reference tables and the focus.
references :: String
references =
  "! edge 0: malformed edge (need [F,G,w])\n\
  \f 0 0 1\nf 1 0 1\ne 0 1\n\
  \! edge 0: file out of range\n\
  \f 0 0 1\ne 0 1 1\n\
  \! edge 0: self edge\n\
  \f 0 0 1\ne 0 0 1\n\
  \! edge 0: weight below 1\n\
  \f 0 0 1\nf 1 0 1\ne 0 1 0\n\
  \! edge 1: not strictly ascending\n\
  \f 0 0 1\nf 1 0 1\ne 1 0 1\ne 0 1 1\n\
  \! pkgEdge 0: malformed pkgEdge (need [F,D,w])\n\
  \f 0 0 1\np 0 0\n\
  \! pkgEdge 0: file out of range\n\
  \f 0 0 1\np 1 0 1\n\
  \! pkgEdge 0: dir out of range\n\
  \f 0 0 1\np 0 1 1\n\
  \! pkgEdge 0: weight below 1\n\
  \f 0 0 1\np 0 0 0\n\
  \! pkgEdge 1: not strictly ascending\n\
  \f 0 0 1\np 0 0 1\np 0 0 1\n\
  \! focus 0: file out of range\n\
  \f 0 0 1\nx 1\n\
  \! focus 1: not strictly ascending\n\
  \f 0 0 1\nf 1 0 1\nx 1\nx 0\n"
