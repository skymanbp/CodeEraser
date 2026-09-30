-- | The flow family's constants (plan v2.31 step 3; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §5):
-- the one cap the four tables count against together, the statement
-- kinds and flag bits the measuring side's FlowSpec tables emit, the
-- variable flags, the access modes and the finding kinds. Nothing
-- here is a condition bit: the core answers findings, and the face
-- and the guard decide what a finding means under `[flow] tier`.
module CE.Flow.Cost (
  rowCap,
  kindBlock,
  kindStmt,
  kindIf,
  kindLoop,
  kindSwitch,
  kindCase,
  kindTry,
  kindCatch,
  kindFinally,
  kindReturn,
  kindThrow,
  kindBreak,
  kindContinue,
  kindGoto,
  kindLabel,
  kindNoreturn,
  kindCeil,
  flagElse,
  flagInfinite,
  flagFallthrough,
  flagDynamic,
  flagEmpty,
  stmtFlagCeil,
  varParam,
  varCaptured,
  varIgnored,
  varAddress,
  varFlagCeil,
  modeRead,
  modeWrite,
  modeReadWrite,
  modeCeil,
  findUnreachable,
  findDeadStore,
  findUnusedLocal,
  findUnusedParam,
) where

-- | The four tables — units, statements, variables, accesses — are
-- one request dimension: their rows are counted together before
-- judging, and crossing the ceiling answers a complete degraded
-- reply (the scan C15 discipline: every dimension a request has is
-- priced, none walks in uncapped).
rowCap :: Integer
rowCap = 524288

-- | Statement kinds, the `kind` column of a `stmts` row. A block,
-- a label and a case run their children in order; an if runs its
-- first child and, under `flagElse`, its second; a loop runs its
-- children and comes back to its own head; a switch runs one of its
-- case children; a try runs its body children, may hand any of them
-- to a catch child and leaves through its finally child; the five
-- jumps and the two exits have no children.
kindBlock, kindStmt, kindIf, kindLoop, kindSwitch, kindCase, kindTry, kindCatch :: Int
kindBlock = 0
kindStmt = 1
kindIf = 2
kindLoop = 3
kindSwitch = 4
kindCase = 5
kindTry = 6
kindCatch = 7

kindFinally, kindReturn, kindThrow, kindBreak, kindContinue, kindGoto, kindLabel, kindNoreturn :: Int
kindFinally = 8
kindReturn = 9
kindThrow = 10
kindBreak = 11
kindContinue = 12
kindGoto = 13
kindLabel = 14
kindNoreturn = 15

-- | A kind above this is refused by name.
kindCeil :: Int
kindCeil = 15

-- | Statement flag bits. `flagElse` on an if says a second child is
-- the else branch, on a switch that a default arm exists (no path
-- skips every case); `flagInfinite` on a loop says its head never
-- exits on its own (Rust `loop`, Go `for {}`, Python `while True`, C
-- `for (;;)`); `flagFallthrough` on a case says its end runs into
-- the next case; `flagDynamic` on any statement of a unit marks the
-- unit as calling an evaluator by name (`eval`, `exec`, `load`…), and
-- such a unit is not judged; `flagEmpty` records an empty branch or
-- body and changes nothing.
flagElse, flagInfinite, flagFallthrough, flagDynamic, flagEmpty :: Int
flagElse = 0
flagInfinite = 1
flagFallthrough = 2
flagDynamic = 3
flagEmpty = 4

-- | Flags above the five bits are refused by name.
stmtFlagCeil :: Integer
stmtFlagCeil = 31

-- | Variable flag bits. A parameter is declared by the unit, at no
-- statement (`declSeq` −1); a captured variable is read by a nested
-- unit or closure the host side cannot see into; an ignored variable
-- carries the language's discard spelling (`_x`); an address-taken
-- variable has been handed out by reference. The last three are
-- exemptions: no finding is ever raised on them.
varParam, varCaptured, varIgnored, varAddress :: Int
varParam = 0
varCaptured = 1
varIgnored = 2
varAddress = 3

-- | Flags above the four bits are refused by name.
varFlagCeil :: Integer
varFlagCeil = 15

-- | Access modes, in evaluation order within a statement: a read, a
-- write, or a read followed by a write (`x += 1`, `x++`).
modeRead, modeWrite, modeReadWrite :: Int
modeRead = 0
modeWrite = 1
modeReadWrite = 2

-- | A mode above this is refused by name.
modeCeil :: Integer
modeCeil = 2

-- | Finding kinds, the second column of a `findings` row. An
-- unreachable run names its first and last statement; a dead store
-- names the statement that wrote and the variable; an unused local
-- names its declaring statement; an unused parameter is advisory
-- (an interface's or an override's parameter may be unused on
-- purpose) and names the variable at `declSeq` −1.
findUnreachable, findDeadStore, findUnusedLocal, findUnusedParam :: Integer
findUnreachable = 0
findDeadStore = 1
findUnusedLocal = 2
findUnusedParam = 3
