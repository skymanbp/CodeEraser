-- | The merge family's constants (plan v2.31 step 6; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §3 and
-- §6): the two caps, the parameter ceiling, the call's line price,
-- the family codes, the position classes the measuring side's tables
-- emit and the reason codes a suggestion answers. Nothing here is a
-- condition bit: merge/1 is advisory — the face reports, no gate
-- reads it.
module CE.Merge.Cost (
  groupCap,
  treeNodeCap,
  paramCap,
  callLines,
  familyExact,
  familyNear,
  slotStatement,
  slotExpression,
  slotType,
  slotName,
  slotCeil,
  reasonNone,
  reasonPosition,
  reasonType,
  reasonAcross,
  reasonParams,
  reasonNoSavings,
) where

-- | Groups and tree nodes are the request's two dimensions (§3's
-- row): either past its ceiling answers a complete degraded reply
-- (the scan C15 discipline — every dimension a request has is
-- priced). The node cap counts every member tree's nodes; it is named
-- for the trees because CE.Graph.Cost.nodeCap already names the
-- graph's (the how page reads each constant by its bare name).
groupCap, treeNodeCap :: Integer
groupCap = 4096
treeNodeCap = 1048576

-- | A merged function taking more than six parameters is not a
-- suggestion anyone would take (§6.3; step-6 ruling 6): reason 4.
paramCap :: Int
paramCap = 6

-- | Each member left behind becomes one call line (step-6 ruling 7:
-- savings = Σ lines − (skeleton lines + members × callLines)).
callLines :: Integer
callLines = 1

-- | Group families (step-6 ruling 1): 0 = a T1/T2 group of two or
-- more members whose trees are isomorphic, 1 = a T3 pair of exactly
-- two members aligned by the tree-edit mapping.
familyExact, familyNear :: Integer
familyExact = 0
familyNear = 1

-- | A node's position class, the `slot` column (§6.2): a statement,
-- an expression, a type, a declared name, anything else.
slotStatement, slotExpression, slotType, slotName :: Int
slotStatement = 0
slotExpression = 1
slotType = 2
slotName = 3

-- | A slot above this is refused by name.
slotCeil :: Int
slotCeil = 4

-- | A suggestion's reason (step-6 ruling 6), the first infeasible
-- hole's in hole order: 0 feasible; 1 a hole at a position no
-- parameter can stand for (a statement or other position); 2 a hole
-- at a type position; 3 a gap hole whose forests hold a statement
-- (a subtree hole across statements — it outranks 1 and 2); 4 every
-- hole feasible but more parameters than `paramCap`; 5 every hole
-- feasible, at most `paramCap` parameters, and no line saved (savings
-- at most zero — a merge no one would take, the stance of 4).
reasonNone, reasonPosition, reasonType, reasonAcross, reasonParams, reasonNoSavings :: Integer
reasonNone = 0
reasonPosition = 1
reasonType = 2
reasonAcross = 3
reasonParams = 4
reasonNoSavings = 5
