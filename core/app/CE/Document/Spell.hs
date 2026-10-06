-- | The strings a document spells from the measuring side's strings
-- rather than reading one straight off a list (plan v2.33 W7): each a
-- rule that lived in a face's `Resolve` (frozen in
-- cli/tests/unit/document/frozen/), named after the Rust it replaces,
-- and the tables that only carried a string's measure across while
-- strings could not cross — each path's place in string order
-- (`document::ranks`) and a directory's width (arch `widths`) — now
-- measured here off the strings the request carries.
module CE.Document.Spell (derived, resolverFor) where

import CE.Document.Bind (Resolver, Strings (..), indexed, nested, textOf)
import CE.Document.Contract (DocReq (..), rows)
import CE.Lang (segmentKinds)
import CE.Resolve.Chars (isRustWhite)
import Control.Applicative ((<|>))
import Data.Aeson (Result (..), Value (..), fromJSON)
import Data.Bits (shiftL)
import qualified Data.IntMap.Strict as IM
import Control.Monad (join)
import Data.List (sort)
import qualified Data.Map.Strict as M

-- | A family's resolver over one request: the plain lookup, and the
-- rules of the families whose faces spelled a string.
resolverFor :: String -> DocReq -> Resolver
resolverFor family req = resolve
 where
  resolve cls ints = case (family, cls, ints) of
    ("arch", "slashed", [d]) -> slashed <$> plain "dir" [d]
    (_, "node_name", [n]) -> nodeName <$> plain "path" [n] <*> plain "node_unit" [n]
    ("flow", "unit", [f, nth]) -> unitOf f nth
    ("flow", "var", [f, nth, v]) -> varOf f nth v
    ("merge", "text", [k, post, postEnd]) -> holeText (k, post, postEnd)
    ("merge", "clipped", [k, post, postEnd, cap]) | cap >= 0 -> clip cap <$> holeText (k, post, postEnd)
    ("erase", "diff", first : _) -> plain "diff" [first]
    ("trend", "short", [i]) -> take 12 <$> plain "commit" [i]
    ("docdup", "seg", [s]) -> seg s
    _ -> plain cls ints
  -- each rule's tables are built once per request, on first use
  strings = indexed req
  plain = nested strings
  unitOf = flowUnit strings
  varOf = flowVar strings
  holeText = mergeText req plain
  seg = segName req plain

-- | arch `slashed`: a directory as an arc end, the root as `./`.
slashed :: String -> String
slashed d = if null d then "./" else d <> "/"

-- | deadcode `node_name`: a node's path, `path#unit` for a unit node.
nodeName :: String -> String -> String
nodeName path unit = if null unit then path else path <> "#" <> unit

-- | Per file of a flow class (`units`, `unlowered`, `vars`): its
-- entries keyed by their first element `nth`, the first in the order
-- sent winning (`lookup`, `find`); Nothing for a file that is not a
-- list. Built once per request, each file's map on first use.
byNth :: M.Map String Strings -> String -> (Strings -> Maybe (Integer, a)) -> Integer -> Maybe (Maybe (M.Map Integer a))
byNth strings cls keyOf = \f -> if f < 0 || f > toInteger (maxBound :: Int) then Nothing else IM.lookup (fromInteger f) files
 where
  files = case M.lookup cls strings of
    Just (Branch m) -> IM.map firsts m
    _ -> IM.empty
  firsts s = case s of
    Branch m -> Just (M.fromListWith (\_ first -> first) [kv | Just kv <- map keyOf (IM.elems m)])
    Leaf _ -> Nothing

-- | A `[nth, value]` pair with `nth` an integer.
pairOf :: Strings -> Maybe (Integer, Value)
pairOf s = case s of
  Branch p | [Leaf (Number n'), Leaf v] <- IM.elems p, Success n <- fromJSON (Number n') -> Just (n, v)
  _ -> Nothing

-- | flow `unit`: the first lowered unit of file `f` at `nth`, else the
-- first unit the lowering left out at `nth`, else empty (`unit_at`,
-- then `unlowered`); Nothing for a file the request does not hold.
flowUnit :: M.Map String Strings -> Integer -> Integer -> Maybe String
flowUnit strings = \f nth -> do
  units <- join (units' f)
  left <- join (left' f)
  if nth < 0 then Nothing else Just (maybe "" id ((M.lookup nth units <|> M.lookup nth left) >>= textOf))
 where
  units' = byNth strings "units" pairOf
  left' = byNth strings "unlowered" pairOf

-- | flow `var`: variable `v` of the first lowered unit of file `f` at
-- `nth` (`unit_at`, then its legend's `var_name`).
flowVar :: M.Map String Strings -> Integer -> Integer -> Integer -> Maybe String
flowVar strings = \f nth v -> do
  vars <- join (vars' f)
  Branch names <- join (M.lookup nth vars)
  if v < 0 then Nothing else IM.lookup (fromInteger v) names >>= leafText
 where
  vars' = byNth strings "vars" legendOf
  -- a vars entry whose first element is an integer, with its second
  -- element (the names) if it has one
  legendOf s = case s of
    Branch p | Just (Leaf n) <- IM.lookup 0 p, Success x <- fromJSON n -> Just (x, IM.lookup 1 p)
    _ -> Nothing
  leafText s = case s of
    Leaf x -> textOf x
    _ -> Nothing

-- | merge `text`: member `k`'s text from node `post` to node `postEnd`
-- — the span text the measuring side cut for the hole row that names
-- that member and those nodes (`holeText`, one per `holes` row).
mergeText :: DocReq -> Resolver -> (Integer, Integer, Integer) -> Maybe String
mergeText req plain = \key@(k, _, _) ->
  if k < 0 || k >= members then Nothing else M.lookup key keyed >>= \h -> plain "holeText" [h]
 where
  members = toInteger (length (rows req "members"))
  ids = M.fromList [((g, m), kk) | kk : g : m : _ <- rows req "members"]
  keyed = M.fromListWith (\_ first -> first) [((M.findWithDefault (-1) (g, m) ids, p, e), i) | (i, g : _ : _ : m : p : e : _) <- zip [0 :: Integer ..] (rows req "holes")]

-- | merge `clipped`: one line, at most `cap` characters, `…` when cut
-- (`clip`: Rust's `split_whitespace` joined by one space).
clip :: Integer -> String -> String
clip cap text = if toInteger (length flat) <= cap then flat else take (fromInteger cap) flat <> "…"
 where
  flat = case rustWords text of
    [] -> ""
    ws -> foldr1 (\a b -> a <> " " <> b) ws

-- | Rust's `str::split_whitespace`: maximal runs of non-White_Space.
rustWords :: String -> [String]
rustWords s = case dropWhile isRustWhite s of
  [] -> []
  t -> let (w, rest) = break isRustWhite t in w : rustWords rest

-- | docdup `seg`: `path:start-end kind` (`docdup::judge::name`); a
-- kind code past the vocabulary reads `kind?`, as the stored column a
-- stale index carries did there.
segName :: DocReq -> Resolver -> Integer -> Maybe String
segName req plain = \s -> do
  [f, a, b, k] <- M.lookup s segs
  path <- plain "path" [f]
  pure (path <> ":" <> show a <> "-" <> show b <> " " <> kindName k)
 where
  segs = M.fromListWith (\_ first -> first) [(i, rest) | i : rest <- rows req "segs"]
  kindName k = case drop (fromInteger k) segmentKinds of
    name : _ | k >= 0 -> name
    _ -> "kind?"

-- | The tables a family's request carried as measures of its strings,
-- measured here when the request carries the strings: each path's
-- place in string order (arch, flow, join, sites) and each arch
-- directory's width in bytes and in characters.
derived :: DocReq -> DocReq
derived req = case (dFamily req, dStrings req) of
  (Just fam, Just _) -> req {dRows = M.union (M.fromList (tablesOf fam)) <$> dRows req}
  _ -> req
 where
  list cls = case M.lookup cls (indexed req) of
    Just (Branch m) -> [t | Leaf v <- IM.elems m, Just t <- [textOf v]]
    _ -> []
  numbered rs = [[i, r] | (i, r) <- zip [0 ..] rs]
  tablesOf fam = case fam of
    "arch" ->
      let (files, dirs) = (list "path", list "dir")
          (byFile, byDir) = splitAt (length files) (ranks (files <> map slashed dirs))
       in [("rankFiles", numbered byFile), ("rankDirs", numbered byDir), ("widths", [[i, utf8Length d, toInteger (length d)] | (i, d) <- zip [0 ..] dirs])]
    "flow" -> [("rankFiles", numbered (ranks (list "path")))]
    "sites" -> [("rankFiles", numbered (ranks (list "path")))]
    "join" -> [("rankPaths", numbered (ranks (list "path")))]
    _ -> []

-- | Each string's place in their joint byte order, ties sharing one
-- (`document::ranks`: the strings below it).
ranks :: [String] -> [Integer]
ranks xs = map below xs
 where
  m = M.fromListWith (\_ first -> first) (zip (sort xs) [0 ..])
  below x = M.findWithDefault 0 x m

-- | A string's length in UTF-8 bytes (Rust's `str::len`).
utf8Length :: String -> Integer
utf8Length = sum . map width
 where
  width c
    | fromEnum c < 0x80 = 1
    | fromEnum c < shiftL 1 11 = 2
    | fromEnum c < 0x10000 = 3
    | otherwise = 4 :: Integer
