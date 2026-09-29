-- | The query family's constants (plan v2.31 step 1; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §4):
-- the caps every request dimension counts against, the token
-- vocabulary the measuring side's lexer emits and this side's parser
-- reads, the sort codes the checker infers and echoes, and the
-- program-error codes a normal reply names. Nothing here is a
-- condition bit: a query is answered, an assert counts its
-- violations, and the face decides what to do with the count.
module CE.Query.Cost (
  tokenCap,
  factCap,
  derivedCap,
  proofCap,
  idbFloor,
  kindPred,
  kindVar,
  kindInt,
  kindSet,
  kindSym,
  kindAnon,
  punctFloor,
  punctCeil,
  tokRule,
  tokComma,
  tokDot,
  tokOpen,
  tokClose,
  tokNot,
  tokQuery,
  tokAssert,
  tokEq,
  tokNe,
  tokLt,
  tokLe,
  tokGt,
  tokGe,
  tokAdd,
  tokSub,
  tokMul,
  tokDiv,
  tokMod,
  tokCount,
  tokMin,
  tokMax,
  tokSum,
  tokColon,
  sortNode,
  sortDir,
  sortUnit,
  sortInt,
  sortSym,
  sortSet,
  sortOpen,
  errSyntax,
  errUnknownPred,
  errArity,
  errSort,
  errUnsafe,
  errUnstratified,
  errPrelude,
  errAggregate,
  errHead,
) where

-- | Program tokens, fact rows, derived tuples and proof nodes are four
-- request dimensions, each with its own ceiling (the scan C15
-- discipline). Tokens and fact rows are counted before judging and
-- answer a complete degraded reply; the derived ceiling is hit while
-- judging and answers the same reply with the count it reached; the
-- proof ceiling never degrades — it stops expanding and counts the
-- answers it left unexplained.
tokenCap, factCap, derivedCap, proofCap :: Integer
tokenCap = 65536
factCap = 4194304
derivedCap = 2097152
proofCap = 16384

-- | Predicate codes below this are the fact schema (CE.Query.Schema);
-- at or above it they are the program's own predicates, numbered by
-- the lexer in order of first appearance.
idbFloor :: Integer
idbFloor = 1000

-- | Token kinds: `[kind, value]` per token. 5 was an enum kind in the
-- booklet's first draft and is deliberately unassigned — enum
-- constants ride as symbols (kind 4), one hashing road for every
-- name-shaped constant.
kindPred, kindVar, kindInt, kindSet, kindSym, kindAnon :: Integer
kindPred = 0
kindVar = 1
kindInt = 2
kindSet = 3
kindSym = 4
kindAnon = 6

-- | Punctuation and keywords occupy one contiguous range; a kind
-- outside `[0..6] ∪ [punctFloor..punctCeil]` (or 5) is refused.
punctFloor, punctCeil :: Integer
punctFloor = 10
punctCeil = 33

tokRule, tokComma, tokDot, tokOpen, tokClose, tokNot, tokQuery, tokAssert :: Integer
tokRule = 10
tokComma = 11
tokDot = 12
tokOpen = 13
tokClose = 14
tokNot = 15
tokQuery = 16
tokAssert = 17

tokEq, tokNe, tokLt, tokLe, tokGt, tokGe :: Integer
tokEq = 18
tokNe = 19
tokLt = 20
tokLe = 21
tokGt = 22
tokGe = 23

tokAdd, tokSub, tokMul, tokDiv, tokMod :: Integer
tokAdd = 24
tokSub = 25
tokMul = 26
tokDiv = 27
tokMod = 28

tokCount, tokMin, tokMax, tokSum, tokColon :: Integer
tokCount = 29
tokMin = 30
tokMax = 31
tokSum = 32
tokColon = 33

-- | Argument sorts. The three id sorts are opaque request-local
-- numberings the measuring side owns the legend of; int is the only
-- sort arithmetic and ordering apply to; sym is a name hash; set is
-- a path-set id (the `in(F, "glob")` sugar). `sortOpen` is what the
-- checker echoes for a program predicate whose position no fact or
-- literal ever constrained.
sortNode, sortDir, sortUnit, sortInt, sortSym, sortSet, sortOpen :: Integer
sortNode = 0
sortDir = 1
sortUnit = 2
sortInt = 3
sortSym = 4
sortSet = 5
sortOpen = -1

-- | Program errors, answered in a normal reply's `errors` table as
-- `[tokenIndex, code]` — a misspelled rule is the program's fault,
-- never the core's, so it is not a contract refusal.
errSyntax, errUnknownPred, errArity, errSort, errUnsafe, errUnstratified, errPrelude, errAggregate, errHead :: Integer
errSyntax = 1
errUnknownPred = 2
errArity = 3
errSort = 4
errUnsafe = 5
errUnstratified = 6
errPrelude = 7
errAggregate = 8
errHead = 9
