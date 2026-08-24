-- | Graph judgment constants — nothing but constants, so the plan
-- §7.4 sensitivity test and the ablation table have exactly one
-- target and a dead knob cannot hide (the CE.FourClass.Cost
-- convention). The M5-2g knobs (minRung / entryMask / sccFloor) are
-- consumed as PARAMETERS by the judgment modules and bound to these
-- constants only at the CE.Graph boundary, so the dead-knob test
-- can perturb each one and watch a verdict count move.
--
-- Integer per the 2026-08-12 blocking decision ③ (the Anchor.hs
-- overflow lesson generalized: guards stay out of bounded arithmetic
-- even while today's only use is a comparison — the Opus review
-- caught the first draft shipping Int against the decided spec).
module CE.Graph.Cost (nodeCap, edgeCap, minRung, entryMask, sccFloor, granFile, assetKind, roleBits) where

-- | Real oversize protection for graph requests (the envelope byte
-- precheck is relaxed for the trusted same-machine child, so these
-- caps are the guard). Sizing anchor: 100k LOC of the M5-2 design
-- brief measures ~20k nodes / ~60k edges, so the caps carry ~6x
-- headroom; a request at cap is ~8 MB, well under the relaxed
-- envelope ceiling. Over cap => degraded graph_too_large, never a
-- truncated graph.
nodeCap :: Integer
nodeCap = 131072

edgeCap :: Integer
edgeCap = 524288

-- | Which rungs count as references: an edge resolved at rung
-- <= minRung is a reference claim. The Rust ladder never guesses
-- (ambiguity is Unresolved and never crosses the wire), so every
-- rung it emits (1..5) is admitted by default; the 2h cut table
-- publishes per-language precision by rung, and lowering this
-- constant is the ablation lever that trades recall for certainty.
minRung :: Integer
minRung = 5

-- | Which flag bits make a node an entry root — the single constant
-- that drives deadcode FPR (design §2). Bits: 1 main, 2 test,
-- 3 entry-glob, 4 dyn-referenced (RG11: dynamic dispatch keeps its
-- target alive and the cost stays visible here), 5 doc-entry,
-- 6 ce:allow(deadcode) (an exemption IS a liveness claim, so an
-- exempt node also keeps its dependencies — the FPR-safe fold).
-- Bit 0 (exported) is deliberately absent: exported-ness is the
-- public/private VERDICT axis, so a library's unreferenced API
-- surfaces as unref_public — the RG10 firewall — never as silence
-- and never as plain dead.
entryMask :: Integer
entryMask = 126

-- | Smallest SCC reported as a cycle: 2 = only true multi-node
-- cycles. A self-loop singleton stays unreported at this floor —
-- widening to self-loops is a knob change the dead-knob test can
-- see, not a code change.
sccFloor :: Integer
sccFloor = 2

-- | The node kind whose dead verdicts FAIL (batch-7 slice 4, RG9:
-- aggregates are not code entities — a package or section verdict
-- is informational `reported`, never a failing `dead` row and never
-- an erase licence). The kind vocabulary is the wire node row
-- [lang, kind, flags]: 0 file / 1 package / 2 section
-- (cli/src/graph/wire.rs GRAN_*). This split lived as an unnamed
-- Rust branch before; a policy the core cannot see is a policy no
-- ablation can price.
granFile :: Integer
granFile = 0

-- | The edge kind that NEVER counts as a reference (batch-7 slice
-- 13, design 4 Markdown row: an image link renders bytes, it does
-- not make its target reachable code). The kind vocabulary is the
-- wire edge row [src, dst, kind, rung]: 0 import / 1 doc-link /
-- 2 doc-ref / 3 asset (cli/src/graph/wire.rs EDGE_*). Rust used to
-- drop these rows before the wire, where no ablation and no cut
-- table could see the rule.
assetKind :: Integer
assetKind = 3

-- | The role→entry-bit table (2.28.0, batch-7 slice 3 main body):
-- graph node rows may carry a 4th column of role FACTS, and the
-- category membership Rust used to fuse into the flags column lands
-- on the entry bits HERE — data an ablation can perturb row by row.
-- Role bits: 0 named main, 1 executable dir, 2 test convention,
-- 3 entry glob, 4 doc entry, 5 ce:allow claim, 6 declared build
-- target. Flag bits per entryMask above; roles 0, 1 and 6 share
-- bit 1 (all three are "an executable the build knows about"), and
-- role 6 is the slice-3 defect fix — a declared [[bin]] path or
-- cabal main-is target is a root, where before only the name
-- conventions were.
roleBits :: [(Integer, Integer)]
roleBits = [(0, 1), (1, 1), (2, 2), (3, 3), (4, 5), (5, 6), (6, 1)]
