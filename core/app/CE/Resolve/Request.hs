{-# LANGUAGE OverloadedStrings #-}

-- | The resolve.request shape (plan v2.33 wave W2a): integer tables
-- only. The measuring side interns every string the search needs into
-- one segment table and sends ids; a directory is a list of segment
-- ids, a file a directory and a basename. Each language's
-- configuration rides as its own sub-object, absent read as empty.
--
-- Two spellings of a directory travel. A /normalized/ directory (a
-- module's, a database's, a search directory the compile chain placed)
-- is its pieces with the root as the empty list. A /raw/ directory —
-- written in a configuration as text (`[graph.search_roots]`,
-- `pyproject.toml`, a Lua `package.path` template) — is the text split
-- at every `/` with empty pieces kept, so two raw directories are one
-- key exactly when their texts are one string; the search joins a raw
-- directory with its empty pieces dropped, the way the path join
-- always did.
module CE.Resolve.Request (
  ResolveReq (..),
  PyFacts (..),
  LuaFacts (..),
  GoFacts (..),
  CFacts (..),
  tableRows,
) where

import Data.Aeson (FromJSON (..), Key, Value, withObject, (.!=), (.:), (.:?))
import Data.Aeson.Types (Parser)

-- | The request: the segment count, the vocabulary's ids, the affix
-- rows, the directory and file tables, the sites, each language's facts.
data ResolveReq = ResolveReq
  { rqId :: Value
  , rqSegs :: Integer
  , rqVocab :: [Integer]
  , rqAffixes :: [[Integer]]
  , rqDirs :: [[Integer]]
  , rqFiles :: [[Integer]]
  , rqSites :: [[Integer]]
  , rqPy :: PyFacts
  , rqLua :: LuaFacts
  , rqGo :: GoFacts
  , rqC :: CFacts
  }

-- | Python: the raw extra import roots (`pyproject.toml`'s package
-- directories) and the declared dependency names, one segment each.
data PyFacts = PyFacts {pyRoots :: [[Integer]], pyDeps :: [Integer]}

-- | Lua: the declared search roots (raw) and the `package.path`
-- templates the tree writes, `[head, n, dir… (n raw pieces), tail…]` —
-- the piece before the template's first `/` after the `?`, the
-- directory before the `?`, the pieces after.
data LuaFacts = LuaFacts {luaRoots :: [[Integer]], luaTemplates :: [[Integer]]}

-- | Go: each go.mod that declares a module, `[n, dir… (n), module…]`,
-- in the configs' order; each replace directive of one,
-- `[mod, form, n, old… (n), new…]`, in its file's order.
data GoFacts = GoFacts {goMods :: [[Integer]], goReplaces :: [[Integer]]}

-- | C and C++: the declared roots (raw), the distinct compile chains
-- `[msvc, ownDir]`, each chain's searched places
-- `[chain, class, kind, dir…]` (class 0 quote, 1 bracket, 2 system;
-- kind 0 a directory, 1 a framework directory), its forced includes
-- `[chain, how, piece…]`, the database entries that seat a file
-- `[file, chain, placed, dir…]`, the directories whose probe found a
-- JSON database, the directories holding a `compile_flags.txt` with its
-- chain `[chain, dir…]`, and every C-family file's include lines
-- `[file, system, piece…]` in file order then line order.
data CFacts = CFacts
  {cRoots, cChains, cSearches, cForced, cSeats, cJsonDirs, cFlags, cIncludes :: [[Integer]]}

instance FromJSON ResolveReq where
  parseJSON = withObject "ResolveReq" $ \o ->
    ResolveReq
      <$> o .: "id"
      <*> o .: "segs"
      <*> o .: "vocab"
      <*> o .:? "affixes" .!= []
      <*> o .:? "dirs" .!= []
      <*> o .:? "files" .!= []
      <*> o .:? "sites" .!= []
      <*> o .:? "py" .!= PyFacts [] []
      <*> o .:? "lua" .!= LuaFacts [] []
      <*> o .:? "go" .!= GoFacts [] []
      <*> o .:? "c" .!= CFacts [] [] [] [] [] [] [] []

instance FromJSON PyFacts where
  parseJSON = twoTables "py" "roots" "deps" PyFacts

instance FromJSON LuaFacts where
  parseJSON = twoTables "lua" "roots" "templates" LuaFacts

instance FromJSON GoFacts where
  parseJSON = twoTables "go" "mods" "replaces" GoFacts

-- | A language object of two tables, each absent read as empty.
twoTables :: (FromJSON a, FromJSON b) => String -> Key -> Key -> ([a] -> [b] -> f) -> Value -> Parser f
twoTables name ka kb make = withObject name $ \o ->
  make <$> o .:? ka .!= [] <*> o .:? kb .!= []

-- | The C object's eight tables, read by key in field order.
instance FromJSON CFacts where
  parseJSON = withObject "c" $ \o -> do
    tables <- traverse (\k -> o .:? k .!= []) ["roots", "chains", "searches", "forced", "seats", "jsonDirs", "flags", "includes"]
    case tables of
      [a, b, c, d, e, f, g, h] -> pure (CFacts a b c d e f g h)
      _ -> fail "c: eight tables"

-- | Every row-carrying table, named, in contract order — the cap counts
-- them together and the contract walks them in this order.
tableRows :: ResolveReq -> [(String, [[Integer]])]
tableRows r =
  [ ("affix", rqAffixes r)
  , ("dir", rqDirs r)
  , ("file", rqFiles r)
  , ("site", rqSites r)
  , ("py.root", pyRoots (rqPy r))
  , ("lua.root", luaRoots (rqLua r))
  , ("lua.template", luaTemplates (rqLua r))
  , ("go.mod", goMods (rqGo r))
  , ("go.replace", goReplaces (rqGo r))
  , ("c.root", cRoots c)
  , ("c.chain", cChains c)
  , ("c.search", cSearches c)
  , ("c.forced", cForced c)
  , ("c.seat", cSeats c)
  , ("c.jsonDir", cJsonDirs c)
  , ("c.flags", cFlags c)
  , ("c.include", cIncludes c)
  ]
 where
  c = rqC r
