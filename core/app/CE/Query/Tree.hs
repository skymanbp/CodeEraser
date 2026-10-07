-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The query family's directory tables (plan v2.33 W1 item 3, additive
-- inside the unreleased 9.0.0; the port of `tree_tables` in
-- cli/src/query/facts/graph.rs at c2abca5d): the measuring side sends
-- every node's path in node order and which nodes are packages; this
-- module builds the tree over the other nodes' paths (CE.Structure.Tree),
-- seats each package at its own directory, and answers `in_dir`, `dir`,
-- `parent` and `dir_name` — the ones the program reads — with each
-- directory's label (the root `.`), its name and the name's hash for the
-- reply.
module CE.Query.Tree (NodeTree (..), parseTree, treeTables) where

import CE.Query.LexState (sym)
import CE.Structure.Tree (Dir (..), Tree (..), build, dirId, dirOf, dirPaths, splitDir)
import Data.Aeson (Object, (.!=), (.:), (.:?))
import Data.Aeson.Types (Parser)
import Data.List (mapAccumL, sort)
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S
import Data.Word (Word64)

-- | The node paths in node order and the package nodes.
data NodeTree = NodeTree
  { ntPaths :: [String]
  , ntPackages :: S.Set Integer
  }

-- | The tree form, when the request carries `tree`.
parseTree :: Object -> Parser (Maybe NodeTree)
parseTree o = do
  tree <- o .:? "tree"
  case tree of
    Nothing -> pure Nothing
    Just t -> Just <$> (NodeTree <$> t .: "paths" <*> fmap S.fromList (t .:? "packages" .!= []))

-- | The four tables by schema code (`in_dir` 2, `dir` 3, `parent` 4,
-- `dir_name` 5), each ascending, and per directory `(label, name, hash)`.
treeTables :: NodeTree -> ([(Int, [[Integer]])], [(String, String, Word64)])
treeTables nt = (tables, [(label p, name p, sym (name p)) | p <- paths])
 where
  nodes = zip [0 :: Integer ..] (ntPaths nt)
  package i = S.member i (ntPackages nt)
  t0 = build [p | (i, p) <- nodes, not (package i)]
  (t, inDir) = mapAccumL seat t0 nodes
  seat tr (i, p)
    | package i = let (tr', d) = dirId tr p in (tr', [i, toInteger d])
    | otherwise = (tr, [i, maybe 0 toInteger (dirOf tr p)])
  dirs = IM.toAscList (tDirs t)
  paths = dirPaths t
  tables =
    [ (2, sort inDir)
    , (3, [[toInteger d] | (d, _) <- dirs])
    , (4, [[toInteger d, toInteger (dParent dir)] | (d, dir) <- dirs, d /= 0])
    , (5, [[toInteger d, toInteger (sym (name p))] | (d, p) <- zip [0 :: Int ..] paths])
    ]
  -- `rsplit('/').next()`, an empty last segment (the root) read as `.`
  name p = case snd (splitDir p) of
    "" -> "."
    n -> n
  label p = if null p then "." else p
