-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The query family's front end (plan v2.33 W1, moved here from
-- cli/src/query/program.rs and columns.rs at 1324c927): the lexed
-- program as the measuring side reads it back — the token count, the
-- clauses and how many are the prelude's, every token's `where
-- line:column`, the predicate names by code, the globs and where each
-- was spelled, the names by hash, the schema predicates the program
-- reads (the fact tables the request must carry), and every goal's
-- head as spelled — or the lexical fault that stopped the program.
-- The goals' column names (design booklet §4.5): the core answers a
-- goal's tuples by position, and the names are the program's own
-- text. A query's columns are the variables its body binds, in the
-- order CE.Query.Check.Safety.outerVars binds them (a positive atom's
-- new variables in term order, a `Var = …` binding its variable;
-- `not`, comparisons and an aggregate's body bind nothing outward);
-- an assertion's columns are its head's terms as written.
module CE.Query.Front (Head (..), goals, lexedValue, located) where

import CE.Query.Cost (idbFloor, kindAnon, kindPred, kindSet, kindVar)
import CE.Query.Lex (Fault (..), Lexed (..), Tok (..), referenced)
import CE.Query.Schema (schemaNames)
import Data.Aeson (Value (Null), object, (.=))
import Data.List (find)
import Data.Maybe (listToMaybe)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | One goal as the program spelled it, in goal order: an assertion's
-- predicate name (a query has none) and its columns.
data Head = Head {hClause :: Int, hAssert :: Bool, hName :: Maybe String, hColumns :: [String]}
  deriving (Eq, Show)

-- | `where line:column` of a token: its source's word, then its place.
located :: Int -> Int -> Int -> String
located src line col = sourceWord src <> " " <> show line <> ":" <> show col

sourceWord :: Int -> String
sourceWord src = case src of
  0 -> "prelude"
  1 -> "rules"
  _ -> "query"

-- | The `lexed` object of a lex reply: the fault, or the program's
-- tables (`inspect` adds the raw tokens and variable tables, for the
-- differential leg against the frozen Rust scanner).
lexedValue :: Bool -> Either Fault Lexed -> Value
lexedValue inspect lexed = case lexed of
  Left f ->
    object $
      ["fault" .= [located (fSrc f) (fLine f) (fCol f), fWhat f]]
        <> ["inspect" .= object ["fault" .= (fSrc f, fLine f, fCol f, fWhat f)] | inspect]
  Right p -> object (tables p <> ["inspect" .= raw p | inspect])
 where
  tables p =
    [ "fault" .= Null
    , "tokens" .= length (lTokens p)
    , "clauses" .= lClauses p
    , "prelude" .= lPrelude p
    , "at" .= [located (tSrc t) (tLine t) (tCol t) | t <- lTokens p]
    , "preds" .= ([(toInteger c, n) | (c, n) <- schemaNames] <> zip [idbFloor ..] (lPreds p))
    , "sets" .= lSets p
    , "setAt" .= [maybe "?" (\t -> located (tSrc t) (tLine t) (tCol t)) (setToken p i) | i <- [0 .. length (lSets p) - 1]]
    , "names" .= M.toAscList (lNames p)
    , "referenced" .= S.toAscList (referenced p)
    , "heads" .= [(hName h, hColumns h) | h <- goals (lTokens p)]
    ]
  setToken p i = find (\t -> tKind t == kindSet && tValue t == toInteger i) (lTokens p)
  raw p =
    object
      [ "tokens" .= [(tKind t, tValue t, tSrc t, tLine t, tCol t, tSpell t) | t <- lTokens p]
      , "vars" .= lVars p
      , "goals" .= [(hClause h, hAssert h, hName h, hColumns h) | h <- goals (lTokens p)]
      , "preds" .= lPreds p
      ]

-- | Every goal clause's head, in clause order (the core's goal index).
goals :: [Tok] -> [Head]
goals toks = [h | (c, cl) <- zip [0 ..] (split "." True toks), Just h <- [headOf c cl]]

-- | The slices between depth-0 occurrences of `at` (parentheses nest),
-- the separator kept with the slice before it when `keep`; the tail
-- after the last separator is a slice when it holds a token.
split :: String -> Bool -> [Tok] -> [[Tok]]
split at keep = go (0 :: Int) []
 where
  go _ acc [] = [reverse acc | not (null acc)]
  go d acc (t : ts) = case tSpell t of
    "(" -> go (d + 1) (t : acc) ts
    ")" -> go (max 0 (d - 1)) (t : acc) ts
    s | s == at && d == 0 -> reverse (if keep then t : acc else acc) : go d [] ts
    _ -> go d (t : acc) ts

headOf :: Int -> [Tok] -> Maybe Head
headOf clause toks = case toks of
  t : rest | tSpell t == "?-" -> Just (Head clause False Nothing (outerVars (body rest)))
  t : rest | tSpell t == "assert" ->
    let hd = takeWhile ((/= ":-") . tSpell) rest
     in Just (Head clause True (tSpell <$> listToMaybe hd) (map spelled (drop 1 (filter ((< 10) . tKind) hd))))
  _ -> Nothing

-- | A term as the head wrote it; an anonymous one reads `_`.
spelled :: Tok -> String
spelled t = if tKind t == kindAnon then "_" else tSpell t

-- | The body without its closing `.`.
body :: [Tok] -> [Tok]
body toks = case reverse toks of
  t : rest | tSpell t == "." -> reverse rest
  _ -> toks

-- | The variables a body binds outward, by first binding.
outerVars :: [Tok] -> [String]
outerVars toks = foldl (\bound v -> if v `elem` bound then bound else bound <> [v]) [] (concatMap binds (split "," False toks))
 where
  binds lit = case lit of
    f : _ | tKind f == kindPred -> [tSpell t | t <- lit, tKind t == kindVar]
    f : e : _ | tKind f == kindVar && tSpell e == "=" -> [tSpell f]
    _ -> []
