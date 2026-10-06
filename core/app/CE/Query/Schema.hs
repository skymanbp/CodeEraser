-- | The fact schema the query family judges against (design booklet
-- §4.2): one row per fact predicate — its code, the sort of each
-- argument. The measuring side assembles these tables from the index
-- and mirrors this table by code (pinned by the `schema` echo, a
-- request key that asks for it back); the predicate NAMES are the
-- lexer's vocabulary and never cross the wire. A code absent here is
-- a contract refusal on the facts object, so a table the measuring
-- side invents cannot be judged by accident.
module CE.Query.Schema (arityOf, codeNamed, schemaNames, schemaRows, sortsOf) where

import CE.Query.Cost (sortDir, sortInt, sortNode, sortSet, sortSym, sortUnit)
import qualified Data.IntMap.Strict as IM

-- | The table, one row per predicate — `code name sort…` — in the
-- booklet's order. The name is the reader's (and the measuring side's
-- lexer vocabulary); a row is one literal, so the table is not a run
-- of same-shaped tuples for the clone gate to count.
rows :: [String]
rows =
  [ "0 node node sym"
  , "1 file node"
  , "2 in_dir node dir"
  , "3 dir dir"
  , "4 parent dir dir"
  , "5 dir_name dir sym"
  , "6 lang node sym"
  , "7 role node sym"
  , "8 lines node int"
  , "9 ref node node sym int"
  , "10 unresolved node int"
  , "11 unit unit node"
  , "12 unit_kind unit sym"
  , "13 unit_lines unit int"
  , "14 unit_at unit int"
  , "15 named unit sym"
  , "16 exported unit"
  , "17 coc unit int"
  , "18 cyclo unit int"
  , "19 nesting unit int"
  , "20 params unit int"
  , "21 clone unit unit sym"
  , "22 dup node node int"
  , "23 docdup node node"
  , "24 mention sym node"
  , "25 class node sym"
  , "26 set set node"
  ]

-- | Code → argument sorts. A row that does not read is left out, and
-- the schema echo's golden pins every row that must be here.
schema :: IM.IntMap [Integer]
schema = IM.fromList [entry | row <- rows, Just entry <- [parsed (words row)]]

parsed :: [String] -> Maybe (Int, [Integer])
parsed (code : _name : sorts) | [(n, "")] <- reads code = (,) n <$> traverse sortNamed sorts
parsed _ = Nothing

sortNamed :: String -> Maybe Integer
sortNamed name = lookup name [("node", sortNode), ("dir", sortDir), ("unit", sortUnit), ("int", sortInt), ("sym", sortSym), ("set", sortSet)]

-- | The sorts of a fact predicate's arguments; Nothing = not a fact
-- predicate (a program predicate, or an unknown code).
sortsOf :: Int -> Maybe [Integer]
sortsOf code = IM.lookup code schema

arityOf :: Int -> Maybe Int
arityOf = fmap length . sortsOf

-- | Each fact predicate's name by code, ascending — the lexer's
-- vocabulary (CE.Query.Lex; plan v2.33 W1 moved the scanner here).
schemaNames :: [(Int, String)]
schemaNames = [(n, name) | c : name : _ <- map words rows, [(n, "")] <- [reads c], IM.member n schema]

-- | A fact predicate's code by name.
codeNamed :: String -> Maybe Int
codeNamed name = lookup name [(n, c) | (c, n) <- schemaNames]

-- | The echo: `[code, arity, sort…]` per predicate, ascending by code.
schemaRows :: [[Integer]]
schemaRows = [toInteger code : toInteger (length sorts) : sorts | (code, sorts) <- IM.toAscList schema]
