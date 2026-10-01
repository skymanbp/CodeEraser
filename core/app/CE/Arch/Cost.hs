-- | The arch family's constants (plan v2.31 step 8; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §7):
-- the two caps the request counts against, the arc bound under which
-- a strongly connected component's feedback arc set is searched
-- exhaustively, the per-mille scale of a directory's instability,
-- and the code domains the reply's integer columns carry. Nothing
-- here is a condition bit: arch/1 is advisory (§1 ruling 3), and its
-- faces never fail a run on what it answers. The ruling numbers
-- below are the step-8 brief's §1.0, the booklet §7.2's details.
module CE.Arch.Cost (
  fileCap,
  refCap,
  exactVertexCap,
  instabilityScale,
  instabilityNone,
  exactYes,
  exactNo,
  rootParent,
) where

-- | Files, one dimension (booklet §3's cap column): a request whose
-- `files` table is longer answers a complete degraded reply, never a
-- truncated one (the scan C15 discipline).
fileCap :: Integer
fileCap = 131072

-- | References, one dimension over two tables (booklet §3): `edges`
-- and `pkgEdges` rows counted together, since both become arcs of
-- the one directory graph (ruling 1).
refCap :: Integer
refCap = 524288

-- | Ruling 2 as amended in the step-8 lane (the 12-arc exhaustive
-- search left a 4× gap to the minimum on a 13-arc component): an SCC
-- of the directory graph with at most this many VERTICES, whatever
-- its arc count, has its feedback arc set found by the ordering DP
-- over vertex subsets (2^14 = 16,384 sets × 14 placements); a larger
-- one goes through the Eades–Lin–Smyth order and the redundancy pass.
exactVertexCap :: Int
exactVertexCap = 14

-- | Ruling 8: instability is ⌊scale · out ÷ (in + out)⌋ — per mille,
-- an integer, no division left over the wire.
instabilityScale :: Integer
instabilityScale = 1000

-- | Ruling 8: a directory no arc touches has no instability to speak
-- of; its row carries this value instead of a quotient by zero.
instabilityNone :: Integer
instabilityNone = -1

-- | Ruling 3: the fourth column of a `cuts` row — 1 when its SCC was
-- searched exhaustively (the cut set is a minimum), 0 when it came
-- from the greedy order (a minimal cut set, not a proven minimum).
exactYes, exactNo :: Integer
exactYes = 1
exactNo = 0

-- | The `dirs` table's root row (row 0) carries this parent, and no
-- other row may (booklet §7.1: a dense tree, root 0).
rootParent :: Integer
rootParent = -1
