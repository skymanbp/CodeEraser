-- | The query language's syntax tree (design booklet §4.1) as the
-- parser builds it from the token stream. Every node that an error
-- can point at carries the index of the token it started on, and the
-- lexed program names each token's place (`located`, CE.Query.Front),
-- so an error is reported at `line:column`.
module CE.Query.Syntax (
  Atom (..),
  Clause (..),
  Expr (..),
  Lit (..),
  Term (..),
  TermV (..),
  clauseBody,
  clauseHead,
  isIdb,
  litAtoms,
) where

import CE.Query.Cost (idbFloor)

-- | A term with the token it stands on.
data Term = Term {termAt :: Int, termOf :: TermV}
  deriving (Eq, Show)

data TermV
  = TVar Int
  | TAnon
  | TInt Integer
  | TSym Integer
  | TSet Integer
  deriving (Eq, Show)

-- | `pred(args…)`; `atomAt` is the predicate token.
data Atom = Atom {atomPred :: Int, atomArgs :: [Term], atomAt :: Int}
  deriving (Eq, Show)

-- | An expression: a variable (with its token), an integer literal, a
-- name hash (with its token — legal beside `=` / `!=`, a sort error
-- inside an operator), a binary operator by token code (24 `+` … 28
-- `%`).
data Expr = EVar Int Int | EInt Integer | ESym Integer Int | EBin Integer Expr Expr
  deriving (Eq, Show)

-- | One body literal. `LAssign` is `Var = expr` as written; whether it
-- binds or compares is the checker's call (bound on the left =
-- comparison). `LAgg v op vars body at` is `V = op(vars : body)`.
data Lit
  = LPos Atom
  | LNeg Atom Int
  | LCmp Integer Expr Expr Int
  | LAssign Int Expr Int
  | LAgg Int Integer [(Int, Int)] [Lit] Int
  deriving (Eq, Show)

-- | A clause: a rule (a fact is a rule with an empty body), a query,
-- or an assertion whose head names the witness columns.
data Clause
  = Rule Atom [Lit]
  | Query [Lit]
  | Assert Atom [Lit]
  deriving (Eq, Show)

clauseBody :: Clause -> [Lit]
clauseBody c = case c of
  Rule _ body -> body
  Query body -> body
  Assert _ body -> body

clauseHead :: Clause -> Maybe Atom
clauseHead c = case c of
  Rule h _ -> Just h
  Query _ -> Nothing
  Assert h _ -> Just h

isIdb :: Int -> Bool
isIdb code = toInteger code >= idbFloor

-- | The atoms a literal reads, with the sign each is read under
-- (True = positive) — an aggregate's body counts as negative
-- dependence, the stratification the booklet fixes.
litAtoms :: Lit -> [(Bool, Atom)]
litAtoms lit = case lit of
  LPos a -> [(True, a)]
  LNeg a _ -> [(False, a)]
  LCmp {} -> []
  LAssign {} -> []
  LAgg _ _ _ body _ -> [(False, a) | l <- body, (_, a) <- litAtoms l]
