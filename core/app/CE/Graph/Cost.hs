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
module CE.Graph.Cost (confidence, nodeCap, edgeCap, symCap, mountCap, unmentionedCap, unmentionedHardCap, minRung, entryMask, sccFloor, granFile, assetKind, refdefKind, roleBits, exportVisBit, publicFlagBit, unmentionedVisMask, exemptCategories, restrictedVisBit, reexportMountBit, pkgPrivateMountBit) where

-- | Real oversize protection for graph requests (the envelope byte
-- precheck is relaxed for the trusted same-machine child, so these
-- caps are the guard). Sizing anchor: 100k LOC of the M5-2 design
-- brief measures ~20k nodes / ~60k edges, so the caps carry ~6x
-- headroom on nodes and ~8x on edges. A request with EVERY table at
-- its cap (6.2.0: nodes, edges, symbols, mounts, unmentioned at the
-- hard cap) runs ~13 MB with small integers and ~23 MB with six-digit
-- node indices in every row — under the 32 MiB envelope either way.
-- Over cap => degraded graph_too_large, never a truncated graph.
nodeCap :: Integer
nodeCap = 131072

edgeCap :: Integer
edgeCap = 524288

-- | Cap for the symbol table (4.1.0). The table is DEDUPED to (node,
-- visibility) pairs with the visibility masked to one bit, so it
-- cannot exceed two rows per node: 40k rows for the brief's 20k
-- files, which this cap clears three times over. Against the
-- STRUCTURAL ceiling of 2 x nodeCap = 262,144 it reaches only half —
-- a 4.1.0 debt named here, not repeated by the advisory caps below —
-- and it exists so an oversize request degrades by name instead of
-- by exhaustion, like its siblings above.
symCap :: Integer
symCap = 131072

-- | Caps for the two advisory tables (6.2.0), each its own literal:
-- an alias of nodeCap would move three caps under one ablation and a
-- dead knob could hide (the rule at the top of this file; symCap set
-- the precedent). `mounts` is validated to at most one row per node
-- and the node count is bounded by nodeCap, so 1 row/node x 131072
-- nodes = 131072 covers the table's structural ceiling exactly.
mountCap :: Integer
mountCap = 131072

-- | The soft cap of the `unmentioned` table: above it the core still
-- judges the graph but DROPS the table (`unmentionedDropped`), never
-- the request — an advisory can never turn the gate red. The producer
-- truncates at the same number (cli/src/mention UNMENTIONED_SOFT_CAP,
-- pinned equal source-to-source by docs_consts): a smaller Rust cap
-- would truncate silently, a larger one would resurrect a dropped
-- table. Sized as the one-row-per-node anchor, 1 row/node x 131072
-- nodes = 131072: at the brief's 20k-node sizing anchor that holds
-- 6.5 rows/node, above the measured peak (the self corpus: 332 rows
-- = 0.57 rows/node, peak 6 per node). The measurement is a value,
-- not the bound; and the provable per-node ceiling — 2^3 visibility
-- words x 2^12 category words = 32,768 rows — prices nothing.
unmentionedCap :: Integer
unmentionedCap = 131072

-- | The hard cap of the same table (a famOverCap disjunct, so an
-- unbounded table is priced before the row validator walks it): its
-- own literal, not an alias of edgeCap (one constant, one ablation
-- target), and the SAME threshold the largest existing table,
-- edgeCap, already carries — a wire family's outer bound, not a
-- multiple of the soft cap. Only a defective or hostile client can
-- reach it: the producer self-limits at unmentionedCap, so a
-- well-formed request never does. Past it => graph_too_large.
unmentionedHardCap :: Integer
unmentionedHardCap = 524288

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
-- | The dead-row confidence (H3, 2.32.0): how far the dead node's
-- OWN language can vouch for its verdict, judged from the request's
-- per-language site ledger [[lang, unresolvedSites, totalSites]].
-- Codes: 0 unvouched — the language still carries unresolved sites,
-- so "nothing references this" assumed no in-corpus lands (the
-- erase.md trust boundary, executed HERE since this minor); 1
-- vacuous — no site of that language ever existed, an absence of
-- evidence stated apart from evidence of absence; 2 vouched — a
-- fully resolved reference population. An absent language row reads
-- (0, 0): vacuous. Validation bounds and orders the ledger, so the
-- first match is the only match.
confidence :: [[Integer]] -> Integer -> Integer
confidence unres lang = case [(u, t) | [l, u, t] <- unres, l == lang] of
  ((u, _) : _) | u > 0 -> 0
  ((_, t) : _) | t > 0 -> 2
  _ -> 1

-- | The unused-reference-definition edge kind (H1 slice 16,
-- 2.29.0): the second liveness-inert kind beside assetKind — a
-- definition that renders nothing must not keep its target alive
-- (user decision D3), and since this minor the rule is executed
-- HERE, in the same comprehension as the rung filter, where the
-- ablation battery can flip it.
refdefKind :: Integer
refdefKind = 5

-- | Which `symbols` visibility bit means "exported" (4.1.0). The
-- visibility word is a MEASURED fact — a local syntactic property of
-- the declaration (`pub` / `export` / a leading underscore), read in
-- the file that declares it — and which of its bits counts as an
-- export surface is the judgment, so it lives here where an ablation
-- can move it. Producer: cli/src/fourclass/visibility/.
exportVisBit :: Integer
exportVisBit = 0

-- | Which node FLAG bit an export surface sets: bit 0, the
-- public/private axis Dead.deadTable has always split on and that no
-- producer could ever set — entry standing is measured per file, and
-- "bit 0 stays unset at file granularity, public-ness is a symbol
-- fact" is what cli/src/graph/deadcode/flags.rs:9 says. With the
-- symbols table riding, a file that declares an exported symbol
-- finally carries it, so verdict codes 2 and 4 (unref_public,
-- unreach_public) become reachable for the first time. It is
-- deliberately NOT in entryMask: an export surface is a verdict
-- axis, never an entry claim (RG10).
publicFlagBit :: Integer
publicFlagBit = 0

-- | Which `unmentioned` rows are judged at all (6.2.0): the row's
-- visibility word must carry every bit of this mask. Default = bits
-- 0 and 1, exported AND scope-exported — a name its own file lets
-- out; a `pub fn` inside a private `mod` is unreachable from outside
-- and not a public-surface question. The ablation 1<<2 alone narrows
-- the advisory to restricted (`pub(crate)`) declarations (K19).
unmentionedVisMask :: Integer
unmentionedVisMask = 3

-- | The convention categories that exempt an unmentioned declaration
-- (sealed criterion §3.2), by bit position in the row's `conv` word:
-- 0 main, 1 test, 2 FFI, 3 registration, 4 protocol, 5 member, 6 Go
-- dispatch method, 7 Go API method, 8 default export, 9 ambient,
-- 10 allow claim. Bit 11 — a Rust `cfg` naming no `test` — is
-- measured and rendered but never exempts: a platform-gated
-- declaration is still one nobody spells. A LIST, not a mask, so an
-- ablation drops one category and watches exactly its rows appear
-- (K19/K36). Exemption reads the `conv` word alone; the visibility
-- word never exempts (K36 pins that a row with an empty category
-- word is judged whatever its visibility says).
exemptCategories :: [Integer]
exemptCategories = [0 .. 10]

-- | The visibility bit that means "restricted" (`pub(crate)` and
-- kin): exported, but no further than the named scope. Producer:
-- cli/src/fourclass/visibility/ VIS_RESTRICTED. Advisory code 2.
restrictedVisBit :: Int
restrictedVisBit = 2

-- | The `mounts` bits, positions frozen with the producer
-- (cli/src/graph/mounts.rs MOUNT_REEXPORTED / MOUNT_PKG_PRIVATE):
-- bit 0 = a façade re-exports the file (advisory code 3), bit 1 = the
-- file's own package keeps it private (advisory code 1).
reexportMountBit :: Int
reexportMountBit = 0

pkgPrivateMountBit :: Int
pkgPrivateMountBit = 1
