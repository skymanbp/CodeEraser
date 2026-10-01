-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The document.request shape and its boundary contract (plan v2.32
-- step 3; design booklet docs/reference/authority-track.md §5): one
-- family's name, the size of every universe a document may refer into
-- (`ranges`), the integer tables the document is assembled from
-- (`rows`: the rows the judgment answered, sent back, and the facts the
-- document needs that the judgment does not), the scalar facts
-- (`facts`) and, when the judgment did not happen, the index of the
-- reason text the measuring side holds (`degraded`). Each family
-- states its request as a text table (`Spec`, read here): the ranges,
-- the facts, the tables with their widths and the universe each column
-- indexes, and the reference classes its document carries. A string
-- the measured repository owns never crosses: the document names it
-- as `{"$": [class, integers…]}` and the measuring side resolves it.
module CE.Document.Contract (DocFamily (..), DocReq (..), Spec (..), Table (..), codes, coreReasons, counted, langName, degradedOf, dense, docFamily, docRowCap, bits, fact, flag, judgedFacts, nameOf, spelled, offence, optional, range, readSpec, ref, rows, single, totalRows, whyRef) where

import CE.Graph (graphTooLarge)
import CE.Lang (languages)
import CE.Lang.Spec (Language (..))
import CE.Text (Catalogue, Lang, Line)
import CE.Verdict (verdictTooLarge)
import Control.Monad (guard)
import Data.Aeson (FromJSON (..), Value (..), object, toJSON, withObject, (.:), (.:?), (.=))
import Data.Aeson.Key (fromString)
import Data.Aeson.Types (Pair)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe, isNothing)

-- | The rows one request may carry, every table together: past it the
-- reply is the family's empty document, degraded `document_too_large`.
docRowCap :: Integer
docRowCap = 1048576

-- | The request. Absent tables are Nothing so the contract names them.
data DocReq = DocReq
  { dId :: Value
  , dFamily :: Maybe String
  , dRanges :: Maybe (M.Map String Integer)
  , dRows :: Maybe (M.Map String [[Integer]])
  , dFacts :: Maybe (M.Map String Integer)
  , dDegraded :: Maybe Integer
  , dLang :: Maybe Integer
  }

instance FromJSON DocReq where
  parseJSON = withObject "DocReq" $ \o ->
    DocReq <$> o .: "id" <*> o .:? "family" <*> o .:? "ranges" <*> o .:? "rows" <*> o .:? "facts" <*> o .:? "degraded" <*> o .:? "lang"

-- | One request table: its width (`open` = at least that many), whether
-- it may ride beside a `degraded` reason, and the universe each fixed
-- column indexes (Nothing = a plain integer).
data Table = Table
  { tName :: String
  , tWidth :: Int
  , tOpen :: Bool
  , tKept :: Bool
  , tCols :: [Maybe String]
  }

-- | A family's statement: ranges, facts (kept under `degraded` or
-- not), tables, the reference classes with the universe of each
-- integer, and the facts and tables a request may leave out.
data Spec = Spec
  { spRanges :: [String]
  , spFacts :: [(String, Bool)]
  , spTables :: [Table]
  , spRefs :: [(String, [Maybe String])]
  , spOptional :: [String]
  }

-- | One family as the dispatcher holds it: its name, the schema id its
-- document carries, its statement, the checks its assembly relies on
-- beyond the statement, the assembly, what else the catalogue lists
-- for it, whether the face prints the document indented (scan and
-- dedup; every other family one line), its console lines in a language
-- (step 5: the console form is part of the statement), the text
-- catalogue they are spoken from and its veto — the exit the face
-- reads, one bit.
data DocFamily = DocFamily
  { dfName :: String
  , dfSchema :: String
  , dfSpec :: Spec
  , dfCheck :: DocReq -> Maybe String
  , dfAssemble :: DocReq -> Value
  , dfCatalogue :: [Pair]
  , dfPretty :: Bool
  , dfText :: Catalogue
  , dfLines :: Lang -> DocReq -> [Line]
  , dfExit :: DocReq -> Bool
  }

-- | A family from its name, schema id, statement text, own checks,
-- assembly and catalogue extras; no console line and no veto until
-- `spoken` gives it its own.
docFamily :: String -> String -> String -> (DocReq -> Maybe String) -> (DocReq -> Value) -> [Pair] -> DocFamily
docFamily name schemaId statement check assemble extras =
  DocFamily name schemaId (readSpec statement) check assemble extras False [] (\_ _ -> []) (const False)

-- | Statement lines stating each name a judged fact; and the check
-- that the named facts are bits, each 0 or 1.
judgedFacts :: [String] -> String
judgedFacts = concatMap (\k -> "fact " <> k <> " judged\n")
bits :: DocReq -> [String] -> Maybe String
bits req ks = asum [Just ("facts: " <> k <> " is not 0 or 1") | k <- ks, fact req k > 1]

-- | The statement text, one line per entry: `range NAME`, `fact NAME
-- kept|judged`, `rows NAME WIDTH[+] kept|judged COLUMN…` (a COLUMN is a
-- range name or `-`, one per fixed column), `ref CLASS ARG…`,
-- `optional NAME…` (facts or tables the request may leave out: absent
-- reads as 0 / empty, present is held to its statement). A line that
-- does not read is a defect of the family's source, named.
readSpec :: String -> Spec
readSpec text = foldr entry (Spec [] [] [] [] []) (filter (not . null) (map words (lines text)))
 where
  entry ws sp = case ws of
    ["range", n] -> sp {spRanges = n : spRanges sp}
    ["fact", n, m] -> sp {spFacts = (n, mode m) : spFacts sp}
    ("rows" : n : w : m : cols) -> sp {spTables = table n w m cols : spTables sp}
    ("ref" : c : args) -> sp {spRefs = (c, map column args) : spRefs sp}
    ("optional" : ns) -> sp {spOptional = ns <> spOptional sp}
    _ -> error ("document statement line does not read: " <> unwords ws)
  mode m = m == "kept" || (m /= "judged" && error ("document statement mode: " <> m))
  table n w m cols = case reads w of
    [(k, rest)] | rest `elem` ["", "+"], length cols == k -> Table n k (rest == "+") (mode m) (map column cols)
    _ -> error ("document statement table: " <> n)
  column c = if c == "-" then Nothing else Just c

-- | A string position: the class and the integers it is resolved by.
ref :: String -> [Integer] -> Value
ref cls ints = object ["$" .= (toJSON cls : map toJSON ints)]

-- | The `degraded` field: null, or the reason text the measuring side
-- holds.
whyRef :: DocReq -> Value
whyRef = maybe Null (\i -> ref "why" [i]) . dDegraded

-- | A table's rows, a fact, a range (the contract has held each one
-- present; absent reads as empty / 0 for the empty document).
rows :: DocReq -> String -> [[Integer]]
rows req k = M.findWithDefault [] k (fromMaybe M.empty (dRows req))

fact :: DocReq -> String -> Integer
fact req k = M.findWithDefault 0 k (fromMaybe M.empty (dFacts req))

range :: DocReq -> String -> Integer
range req k = M.findWithDefault 0 k (fromMaybe M.empty (dRanges req))

-- | Every row's column `c` a code in [lo, hi].
codes :: DocReq -> String -> Int -> Integer -> Integer -> Maybe String
codes req t c lo hi = asum [Just (t <> " " <> show i <> ": column " <> show c <> " is not a code " <> show lo <> ".." <> show hi) | (i, r) <- zip [0 :: Int ..] (rows req t), any outside (take 1 (drop c r))]
 where
  outside x = x < lo || x > hi

-- | A counts object: the names beside their numbers.
counted :: [String] -> [Integer] -> Value
counted names = object . zipWith (\k n -> fromString k .= n) names

-- | A table with one row per slot, in slot order: its first column
-- 0, 1, … n − 1.
dense :: DocReq -> String -> Integer -> Maybe String
dense req t n
  | map (take 1) (rows req t) == [[i] | i <- [0 .. n - 1]] = Nothing
  | otherwise = Just (t <> ": not one row per slot in order")

-- | A table of at most one row: the value a nullable field carries.
single :: DocReq -> String -> Maybe String
single req t = if length (rows req t) <= 1 then Nothing else Just (t <> ": more than one row")

-- | A nullable field: its one-row table's value, or null.
optional :: DocReq -> String -> Value
optional req t = case rows req t of
  [v : _] -> toJSON v
  _ -> Null

-- | A 0 / 1 fact as a boolean.
flag :: DocReq -> String -> Bool
flag req k = fact req k /= 0

-- | A product name by its code; the contract has held the code inside
-- the table (`codes`).
spelled :: [String] -> Integer -> Value
spelled table c = toJSON (nameOf table c)

-- | The same name as a plain string, empty off the table.
nameOf :: [String] -> Integer -> String
nameOf table c = concat (take 1 (drop (fromInteger c) table))

-- | A language's report name by its wire code (CE.Lang's rows), empty
-- off the table.
langName :: Integer -> String
langName c = concat [lgName l | l <- take 1 (filter ((== c) . toInteger . lgCode) languages)]

-- | The reasons a judgment reply names when it degraded, by code: the
-- graph family's and the verdict family's over-cap refusals, read from
-- the modules that answer them (CE.Graph, CE.Verdict). A document
-- carries the core's word for them.
coreReasons :: [String]
coreReasons = [graphTooLarge, verdictTooLarge]

-- | The `degraded` field of a family whose judgment may degrade: the
-- measuring side's reason when the judgment never happened, else the
-- first reason the named reply tables carry (in the order given), else
-- null.
degradedOf :: DocReq -> [String] -> Value
degradedOf req tables = case dDegraded req of
  Just _ -> whyRef req
  Nothing -> case [c | t <- tables, c : _ <- rows req t] of
    c : _ -> spelled coreReasons c
    [] -> Null

totalRows :: DocReq -> Integer
totalRows req = toInteger (sum (map length (M.elems (fromMaybe M.empty (dRows req)))))

-- | The first offender against a family's statement, in request
-- order: the three objects present, each holding exactly its stated
-- keys, no negative range or fact, every row its width with every
-- indexing column inside its universe, the reason inside `why`, and
-- under a reason no judged row or fact; then the family's own checks.
offence :: DocFamily -> DocReq -> Maybe String
offence fam req = case (dRanges req, dRows req, dFacts req) of
  (Nothing, _, _) -> Just "document: missing ranges"
  (_, Nothing, _) -> Just "document: missing rows"
  (_, _, Nothing) -> Just "document: missing facts"
  (Just rs, Just ts, Just fs) ->
    asum
      [ keyed "ranges" (spRanges sp) [] rs
      , negative "ranges" rs
      , keyed "rows" (map tName (spTables sp)) (spOptional sp) ts
      , asum [shaped rs t (M.findWithDefault [] (tName t) ts) | t <- spTables sp]
      , keyed "facts" (map fst (spFacts sp)) (spOptional sp) fs
      , negative "facts" fs
      , reason rs
      , asum [judgedWith t | t <- spTables sp, not (tKept t), not (null (rows req (tName t)))]
      , asum [factWith n | (n, False) <- spFacts sp, fact req n /= 0]
      , dfCheck fam req
      ]
 where
  sp = dfSpec fam
  reason rs = do
    i <- dDegraded req
    guard (i < 0 || i >= M.findWithDefault 0 "why" rs)
    pure "degraded: out of range why"
  judgedWith t = do
    _ <- dDegraded req
    pure ("degraded: with rows " <> tName t)
  factWith n = do
    _ <- dDegraded req
    pure ("degraded: with fact " <> n)

-- | An object holding exactly the stated keys (an optional one may be
-- absent): the first missing one, else the first unknown one.
keyed :: String -> [String] -> [String] -> M.Map String a -> Maybe String
keyed what stated optional' m =
  asum
    [ asum [Just (what <> ": missing " <> k) | k <- stated, k `notElem` optional', isNothing (M.lookup k m)]
    , asum [Just (what <> ": unknown key " <> k) | k <- M.keys m, k `notElem` stated]
    ]

negative :: String -> M.Map String Integer -> Maybe String
negative what m = asum [Just (what <> ": " <> k <> " is negative") | (k, v) <- M.toList m, v < 0]

-- | Every row of a table its width, every indexing column inside its
-- universe: `<table> <i>: <reason>`.
shaped :: M.Map String Integer -> Table -> [[Integer]] -> Maybe String
shaped rs t = asum . zipWith row [0 :: Int ..]
 where
  row i r
    | length r < tWidth t || (not (tOpen t) && length r > tWidth t) =
        Just (label i <> "malformed row (need " <> show (tWidth t) <> (if tOpen t then "+" else "") <> " columns)")
    | otherwise = asum (zipWith (cell i) [0 :: Int ..] (zip (tCols t) r))
  cell i c (Just u, v)
    | v < 0 || v >= M.findWithDefault 0 u rs = Just (label i <> "column " <> show c <> " out of range " <> u)
  cell _ _ _ = Nothing
  label i = tName t <> " " <> show i <> ": "
