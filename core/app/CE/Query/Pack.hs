-- | The built-in rule pack (plan v2.33, the `[rules.pack]` table of
-- ce.toml): common architecture checks a repository switches on without
-- writing CE Datalog. The measuring side reads the table and sends the
-- declared items as rows `[name, params…]` — integers and strings only;
-- this side renders each item to CE Datalog text, one clause set per
-- item, and that text is lexed as a fourth source between the prelude
-- and the rules file (CE.Query.Lex), so the assertions' witness rows and
-- their proofs come out of the same machinery as a hand-written rule's.
-- Every predicate rendered here is spelled `pack_…` (CE.Query.Cost
-- `packPrefix`, reserved by CE.Query.Check). The five items:
--
-- * `no_dir_cycles` — no two directories reference each other, directly
--   or through others: the transitive closure of the prelude's `dir_ref`,
--   one witness row per unordered pair (`D < E`);
-- * `fan_in_max N` / `fan_out_max N` — a file referenced by more than N
--   other files / referencing more than N other files, counted over
--   file nodes only;
-- * `layers g0 g1 …` — ordered globs, top first: a file in a lower layer
--   may not reference a file in a higher one, one assertion per ordered
--   pair i > j (a file belongs to every layer whose glob it matches);
-- * `forbid from to` — a file matching the first glob may not reference
--   a file matching the second, one assertion per pair.
--
-- A glob is spelled into the text as a quoted string and read by the
-- rules file's own `in(F, "glob")` sugar, so a glob the exclude dialect
-- cannot read is reported by the measuring side at the glob's place in
-- the pack, as a rules file's would be. What cannot be rendered is
-- refused by name: an unknown item, a malformed row, an item declared
-- twice (`forbid` excepted — it is a list), fewer than two layers, a
-- list past its cap, a negative or out-of-range count, a glob holding a
-- quote or a newline (the string token ends at either).
module CE.Query.Pack (renderPack, reservedAt) where

import CE.Query.Cost (forbidCap, layerCap, packPrefix)
import Data.Aeson (Result (..), Value, fromJSON)
import Data.List (intercalate, isPrefixOf, mapAccumL)

-- | The positions, among a program's predicate names in code order,
-- of those spelled with the reserved prefix — whether or not a pack
-- declared them (CE.Query.Check refuses a rule that defines one).
reservedAt :: [String] -> [Integer]
reservedAt names = [i | (i, n) <- zip [0 ..] names, packPrefix `isPrefixOf` n]

-- | One declared item, its parameters read.
data Item = Cycles | FanIn Integer | FanOut Integer | Layers [String] | Forbid String String

-- | The rows to the pack's text, or the first refusal (prefixed
-- `pack`): the rows are read in order, then the duplicates, then the
-- `forbid` cap, then the clauses rendered in the order the rows came.
renderPack :: [[Value]] -> Either String String
renderPack rows = do
  items <- traverse item rows
  case twice [name | (n : _) <- rows, Just name <- [text n], name /= "forbid"] of
    Just w -> Left ("pack " <> w <> ": declared twice")
    _ | pairs items > forbidCap -> Left ("pack forbid: " <> show (pairs items) <> " pairs, over the cap " <> show forbidCap)
    _ -> Right (concat (snd (mapAccumL render 0 items)))
 where
  pairs items = length [() | Forbid _ _ <- items]

-- | The first name seen before.
twice :: [String] -> Maybe String
twice = go []
 where
  go _ [] = Nothing
  go seen (v : vs) = if v `elem` seen then Just v else go (v : seen) vs

-- | One row: its name picks the reader of its parameters.
item :: [Value] -> Either String Item
item row = case row of
  n : params | Just w <- text n -> maybe (Left ("pack: unknown item " <> show w)) (\r -> r params) (lookup w readers)
  _ -> Left "pack: malformed item (need [name, params…])"
 where
  readers =
    [ ("no_dir_cycles", \ps -> if null ps then Right Cycles else shape "no_dir_cycles" "no parameters")
    , ("fan_in_max", fmap FanIn . oneCount "fan_in_max")
    , ("fan_out_max", fmap FanOut . oneCount "fan_out_max")
    , ("layers", layers)
    , ("forbid", forbid)
    ]

shape :: String -> String -> Either String a
shape name need = Left ("pack " <> name <> ": malformed (need " <> need <> ")")

oneCount :: String -> [Value] -> Either String Integer
oneCount name ps = case ps of
  [v] | Success n <- fromJSON v -> if n < 0 || n > 9223372036854775807 then Left ("pack " <> name <> ": " <> show n <> " is not a count") else Right n
  _ -> shape name "one integer"

layers :: [Value] -> Either String Item
layers ps
  | length ps < 2 = Left ("pack layers: " <> show (length ps) <> " glob(s), a layering needs at least two")
  | length ps > layerCap = Left ("pack layers: " <> show (length ps) <> " globs, over the cap " <> show layerCap)
  | otherwise = Layers <$> traverse (glob "layers") ps

forbid :: [Value] -> Either String Item
forbid ps = case ps of
  [a, b] -> Forbid <$> glob "forbid" a <*> glob "forbid" b
  _ -> shape "forbid" "two globs"

-- | A glob as a string the scanner reads back whole.
glob :: String -> Value -> Either String String
glob name v = case text v of
  Just t
    | any (`elem` ['"', '\n']) t -> Left ("pack " <> name <> ": the glob " <> show t <> " holds a quote or a newline")
    | otherwise -> Right t
  Nothing -> shape name "globs as strings"

-- | A JSON string's text.
text :: Value -> Maybe String
text v = case fromJSON v of
  Success t -> Just t
  Error _ -> Nothing

-- | One item's clauses under a comment naming the declaration; the
-- accumulator numbers the `forbid` pairs from 0.
render :: Int -> Item -> (Int, String)
render k it = case it of
  Cycles ->
    ( k
    , unlines
        [ "# [rules.pack] no_dir_cycles = true"
        , "pack_dir_reach(D, E) :- dir_ref(D, E)."
        , "pack_dir_reach(D, F) :- pack_dir_reach(D, E), dir_ref(E, F)."
        , "assert pack_no_dir_cycles(D, E) :- pack_dir_reach(D, E), pack_dir_reach(E, D), D < E."
        ]
    )
  FanIn n -> (k, fan "fan_in_max" n ("G", "F"))
  FanOut n -> (k, fan "fan_out_max" n ("F", "G"))
  Layers gs ->
    ( k
    , unlines $
        ("# [rules.pack] layers = " <> list gs)
          : [ reads' ("pack_layers_" <> show i <> "_" <> show j) lower upper
            | (i, lower) <- zip [0 :: Int ..] gs
            , (j, upper) <- zip [0 ..] gs
            , i > j
            ]
    )
  Forbid a b -> (k + 1, unlines ["# [rules.pack] forbid " <> list [a, b], reads' ("pack_forbid_" <> show k) a b])
 where
  list gs = "[" <> intercalate ", " (map quoted gs) <> "]"
  -- the file kept and the files counted: fan-in keeps the target G and
  -- counts the sources F, fan-out the reverse
  fan name n (keep, other) =
    unlines
      [ "# [rules.pack] " <> name <> " = " <> show n
      , "assert pack_" <> name <> "(" <> keep <> ", N) :- file(" <> keep <> "), N = count(" <> other <> " : ref(F, G, _, _), file(" <> other <> "), F != G), N > " <> show n <> "."
      ]
  reads' name from to = "assert " <> name <> "(F, G) :- in(F, " <> quoted from <> "), ref(F, G, _, _), in(G, " <> quoted to <> ")."

quoted :: String -> String
quoted g = "\"" <> g <> "\""
