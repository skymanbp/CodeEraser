-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The query.request shape and its boundary contract (design
-- booklet §4.4 and §3's refusal rule): what a well-formed request
-- IS — every token a `[kind, value]` pair of a known kind, every
-- fact table a schema code whose rows have that predicate's arity,
-- non-negative values and strictly ascend as tuples (a table is a
-- set) — and which dimension the cap prices. A misspelled RULE is
-- the program's fault and answers in the `errors` table
-- (CE.Query); a malformed TABLE is the measuring side's fault and
-- is refused here by name, before any judging.
module CE.Query.Contract (QueryReq (..), facts, offence, overCap, tokens) where

import CE.Query.Cost (factCap, idbFloor, kindAnon, kindInt, kindPred, kindSet, kindSym, kindVar, punctCeil, punctFloor, tokenCap)
import CE.Query.Front (located)
import CE.Query.Lex (Fault (..), Lexed (..), Tok (..), lexProgram)
import CE.Query.Schema (arityOf)
import CE.Query.Tree (NodeTree, parseTree, treeTables)
import CE.Wire (rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Char (isDigit)
import Data.Foldable (asum)
import Data.Maybe (listToMaybe)
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import Data.Word (Word64)

-- | The request: id, the token stream, the fact tables keyed by
-- predicate code (decimal strings — JSON object keys), how many
-- leading clauses are the prelude, whether queries carry proofs,
-- whether the schema rides back. Since 9.0.0 (plan v2.33 W1, additive
-- in the unreleased minor) the program may ride as its three texts —
-- the prelude, the rules file, the query (`null` when absent) — and
-- this side lexes them (CE.Query.Lex); `lex` asks for the lexed
-- program alone (CE.Query.Front), `inspect` adds the raw tokens. Since
-- item 3 of the same lane the request may carry `tree` — every node's path
-- and the package nodes (CE.Query.Tree) — and this side builds the four
-- directory tables the program reads; `dirsOf` keeps each directory's
-- label, name and hash for the reply, `clashOf` names a directory table
-- sent beside the tree.
data QueryReq = QueryReq
  { reqId :: Value
  , lexedOf :: Maybe (Either Fault Lexed)
  , lexOf :: Bool
  , inspectOf :: Bool
  , programOf :: [[Integer]]
  , tablesOf :: [(String, [[Integer]])]
  , preludeOf :: Integer
  , whyOf :: Bool
  , schemaOf :: Bool
  , dirsOf :: Maybe [(String, String, Word64)]
  , clashOf :: Maybe String
  }

instance FromJSON QueryReq where
  parseJSON = withObject "QueryReq" $ \o -> do
    lexed <- fmap (\(p, r, q) -> lexProgram p r q) <$> o .:? "texts"
    let program = [[tKind t, tValue t] | Just (Right l) <- [lexed], t <- lTokens l]
        prelude = [toInteger (lPrelude l) | Just (Right l) <- [lexed]]
    req <-
      QueryReq
        <$> o .: "id"
        <*> pure lexed
        <*> o .:? "lex" .!= False
        <*> o .:? "inspect" .!= False
        <*> maybe (o .:? "program" .!= []) (const (pure program)) lexed
        <*> fmap M.toList (o .:? "facts" .!= M.empty)
        <*> maybe (o .:? "prelude" .!= 0) pure (listToMaybe prelude)
        <*> o .:? "why" .!= False
        <*> o .:? "schema" .!= False
        <*> pure Nothing
        <*> pure Nothing
    maybe req (withTree req) <$> parseTree o

-- | The directory tables built from the tree, the ones the program reads
-- (a schema predicate among its tokens), added to the fact tables; a
-- directory table also sent is named.
withTree :: QueryReq -> NodeTree -> QueryReq
withTree req nt = req {tablesOf = M.toList (M.union sent built), dirsOf = Just dirs, clashOf = clash}
 where
  (tables, dirs) = treeTables nt
  read' = S.fromList [v | [k, v] <- programOf req, k == kindPred, v < idbFloor]
  built = M.fromList [(show code, rows) | (code, rows) <- tables, S.member (toInteger code) read']
  sent = M.fromList (tablesOf req)
  clash = case M.keys (M.intersection sent built) of
    k : _ -> Just ("facts " <> k <> ": a directory table beside the tree")
    [] -> Nothing

-- | Tokens and fact rows are priced separately: each is its own
-- request dimension with its own ceiling (CE.Query.Cost). A lex
-- request is never priced: it carries no facts, and its answer is the
-- token count the judging request is priced by.
overCap :: QueryReq -> Bool
overCap req =
  not (lexOf req)
    && ( toInteger (length (programOf req)) > tokenCap
          || toInteger (sum [length rows | (_, rows) <- tablesOf req]) > factCap
       )

-- | The first offender in request order: the tokens, the prelude
-- count, then each table (unknown code first, then its rows).
offence :: QueryReq -> Maybe String
offence req =
  asum
    [ textsShape req
    , clashOf req
    , asum (zipWith tokenShape [0 :: Int ..] (programOf req))
    , if preludeOf req < 0 then Just "negative prelude" else Nothing
    , asum (map tableShape (tablesOf req))
    ]

-- | A lex request needs its texts; a judging request's texts must lex
-- (the measuring side asked for the lexed program first).
textsShape :: QueryReq -> Maybe String
textsShape req = case (lexOf req, lexedOf req) of
  (True, Nothing) -> Just "lex without texts"
  (False, Just (Left f)) -> Just ("texts: " <> located (fSrc f) (fLine f) (fCol f) <> ": " <> fWhat f)
  _ -> Nothing

tokenShape :: Int -> [Integer] -> Maybe String
tokenShape = rowCheck "token" "malformed token (need [kind,value])" 2 tokenChecks

tokenChecks :: [Integer] -> Maybe String
tokenChecks tok = case tok of
  [kind, v]
    | not (knownKind kind) -> Just ("unknown token kind " <> show kind)
    | kind /= kindInt && v < 0 -> Just "negative token value"
    | kind == kindPred && v < idbFloor && arityOf (fromInteger v) == Nothing -> Just ("unknown predicate code " <> show v)
  _ -> Nothing

knownKind :: Integer -> Bool
knownKind kind = kind `elem` [kindPred, kindVar, kindInt, kindSet, kindSym, kindAnon] || (kind >= punctFloor && kind <= punctCeil)

-- | A fact table: its key must be a schema code, its rows that
-- predicate's arity of non-negative values, strictly ascending.
tableShape :: (String, [[Integer]]) -> Maybe String
tableShape (key, rows) = case tableCode key >>= \c -> fmap ((,) c) (arityOf c) of
  Nothing -> Just ("facts " <> key <> ": unknown fact predicate")
  Just (code, arity) -> tableOffence label id (rowCheck label ("malformed fact (need " <> show arity <> " values)") arity negative) rows
   where
    label = "facts " <> show code
    negative row = if any (< 0) row then Just "negative fact value" else Nothing

-- | A key is a code when it is a plain decimal (no sign, no leading
-- zero beyond "0" itself), so "007" and "-1" cannot alias a table.
tableCode :: String -> Maybe Int
tableCode key = case key of
  "0" -> Just 0
  c : _ | c /= '0' && all isDigit key -> Just (read key)
  _ -> Nothing

-- | The tokens as pairs, after the contract fixed their shape.
tokens :: QueryReq -> [(Integer, Integer)]
tokens req = [(k, v) | [k, v] <- programOf req]

-- | The tables keyed by code, after the contract fixed their keys.
facts :: QueryReq -> [(Int, [[Integer]])]
facts req = [(code, rows) | (key, rows) <- tablesOf req, Just code <- [tableCode key]]
