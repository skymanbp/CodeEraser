{-# LANGUAGE OverloadedStrings #-}

-- | The resolve.request shape (plan v2.33 wave W2a; text since W2-text,
-- proto 9.0.0, algorithm-track §3): the measuring side sends what it
-- read, as text — the walked paths, the sites' specifiers as the
-- detector produced them, the `ce.toml` values the ladders read, and
-- the configuration files: each go.mod's and each R DESCRIPTION's text,
-- every walked Java file's header as the walk read it (CE.Resolve.JavaHeader),
-- each .cabal file's text and which cabal owns each walked Haskell file
-- (the nearest one the measuring side's directory scan found),
-- the root `pyproject.toml` and every `compile_commands.json` as the
-- decoded document (format decoding is a library read on that side;
-- every rule applied to the document is here), each `compile_flags.txt`
-- and each response file a database names as text. What needs the file system
-- stays a small fact: the absolute root's text, the databases a probe
-- found, each C-family file's include list (read in the walk, a key
-- input there). Every object and table may be absent, read as empty.
module CE.Resolve.Request (
  ResolveReq (..),
  Site (..),
  CReq (..),
  Db (..),
  searchRoots,
) where

import CE.Resolve.JavaHeader (JHeader, headerRow)
import Data.Aeson (FromJSON (..), Key, Object, Value, withObject, (.!=), (.:), (.:?))
import qualified Data.Aeson.Key as K
import Data.Aeson.Types (Parser)
import qualified Data.Map.Strict as M

data ResolveReq = ResolveReq
  { rqId :: Value
  , rqFiles :: [String]
  , rqOrigins :: [String]
  , rqSites :: [Site]
  , rqSearch :: M.Map String [String]
  , rqPyproject :: Maybe Value
  , rqLuaTemplates :: [(String, String)]
  , rqGoMods :: [(String, String)]
  , rqDescriptions :: [(String, String)]
  , rqJavaHeaders :: [(String, JHeader)]
  , rqHsCabals :: [(String, String)]
  , rqHsOwners :: [(String, String)]
  , rqC :: CReq
  , rqInspect :: Maybe Value
  }

-- | `[lang, kind, from, spec]`: the language and site-kind codes, the
-- index of the site's file in files ++ origins, the specifier; a Java
-- site adds its 1-based source line (`[lang, kind, from, spec, line]`):
-- the Java rungs read the header's import on that line and the types
-- enclosing it.
data Site = Site {sLang :: Integer, sKind :: Integer, sFrom :: Int, sSpec :: String, sLine :: Maybe Int}

-- | One database a probe found: the probed directory, the probe's index
-- (0 `compile_commands.json`, 1 `build/compile_commands.json`, 2
-- `compile_flags.txt`), the file's repo-relative path.
data Db = Db String Int String

-- | The C and C++ facts: the root's text, the databases found, each
-- JSON database's rows (null = it does not read as an array), each
-- flags file's text and each response file's text (null = unreadable),
-- and every walked C-family file's include specifiers (by its path) in
-- line order.
data CReq = CReq
  { cRoot :: String
  , cDbs :: [Db]
  , cJson :: [(String, Maybe [Value])]
  , cFlags :: [(String, Maybe String)]
  , cResponses :: [(String, Maybe String)]
  , cIncludes :: [(String, [String])]
  }

instance FromJSON ResolveReq where
  parseJSON = withObject "ResolveReq" $ \o -> do
    cfg <- o .:? "config"
    py <- o .:? "py"
    ResolveReq
      <$> o .: "id"
      <*> o .:? "files" .!= []
      <*> o .:? "origins" .!= []
      <*> o .:? "sites" .!= []
      <*> maybe (pure M.empty) (withObject "config" (\c -> c .:? "searchRoots" .!= M.empty)) cfg
      <*> maybe (pure Nothing) (withObject "py" (.:? "pyproject")) py
      <*> listAt o "lua" "templates"
      <*> listAt o "go" "mods"
      <*> listAt o "r" "descriptions"
      <*> (mapM headerRow =<< listAt o "java" "headers")
      <*> listAt o "hs" "cabals"
      <*> listAt o "hs" "owners"
      <*> o .:? "c" .!= CReq "" [] [] [] [] []
      <*> o .:? "inspect"

instance FromJSON Site where
  parseJSON v = do
    cols <- parseJSON v
    case cols :: [Value] of
      [l, k, f, s] -> Site <$> parseJSON l <*> parseJSON k <*> parseJSON f <*> parseJSON s <*> pure Nothing
      [l, k, f, s, n] -> Site <$> parseJSON l <*> parseJSON k <*> parseJSON f <*> parseJSON s <*> (Just <$> parseJSON n)
      _ -> fail "malformed site (need [lang,kind,from,spec] or [lang,kind,from,spec,line])"

instance FromJSON Db where
  parseJSON v = (\(d, p, r) -> Db d p r) <$> parseJSON v

instance FromJSON CReq where
  parseJSON = withObject "c" $ \o ->
    CReq
      <$> o .:? "root" .!= ""
      <*> o .:? "dbs" .!= []
      <*> o .:? "json" .!= []
      <*> o .:? "flags" .!= []
      <*> o .:? "responses" .!= []
      <*> o .:? "includes" .!= []

-- | The list at `section.key` of a request; an absent section or key is
-- empty, a section that is no object is refused under its own name.
listAt :: (FromJSON a) => Object -> Key -> Key -> Parser [a]
listAt o section k = o .:? section >>= maybe (pure []) (withObject (K.toString section) (\s -> s .:? k .!= []))

-- | One language's declared `[graph.search_roots]` directories.
searchRoots :: String -> ResolveReq -> [String]
searchRoots lang = M.findWithDefault [] lang . rqSearch
