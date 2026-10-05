{-# LANGUAGE OverloadedStrings #-}

-- | The TS / TSX rungs (moved from cli/src/graph/ladder/ts.rs and
-- ts_node.rs in plan v2.33 W2-text stage E — each function below is the
-- Rust function of the same name).
--   R1 relative + extension order — the order IS the norm, so several
--      hits are not ambiguous.
--   R2 the ESM `.js` → `.ts` rewrite (`.mjs` → `.mts`, `.cjs` → `.cts`),
--      only when the TS twin is walked and the JS twin is no file.
--   R3 the nearest tsconfig's baseUrl and paths (the whole extends
--      chain, arrays included; a cycle or an unreadable target is
--      config_depth; two distinct paths hits are ambiguous_paths).
--   R4 a workspace member by name, its subpath through the exports
--      map's exact key or its one matching pattern (Node's
--      PACKAGE_EXPORTS_RESOLVE); duplicate names are ambiguous.
--   R5 a bare specifier: a Node builtin (`fs`, `node:fs`), a name the
--      nearest package.json declares, or one present under a
--      node_modules/ of the importer's directory or any ancestor ⇒
--      External.
-- Everything else is unresolved: a relative specifier with no walked
-- target is out_of_scope, a `node:` name Node has no module for too,
-- and a unique member whose export target is no walked file ends there
-- (falling to R5 would call an in-corpus package External). What the
-- rungs need of the file system comes from the facts
-- (CE.Resolve.TsFacts): a rung that lacks one waits for it.
module CE.Resolve.Ts (TsEnv (..), resolveTs) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str (ancestors, joinRel, parentDir, splitOnce, stripSuffix, utf8Len)
import CE.Resolve.Tables (nodeBuiltins, nodePrefixOnly)
import CE.Resolve.TsConfig
import CE.Resolve.TsFacts
import CE.Resolve.World (World, member)
import Data.Aeson (Value (..))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.List (elemIndex, isPrefixOf, stripPrefix)
import qualified Data.Set as Set

-- | What the rungs read: the world, the facts, each importer
-- directory's tsconfig chain (computed once per directory) and the
-- walk's package.json files, read.
data TsEnv = TsEnv
  { teWorld :: World
  , teFacts :: Facts
  , teChain :: String -> Need TsChain
  , teMembers :: Need [Package]
  }

-- | `EXTS`: the extension order, first hit wins.
exts :: [String]
exts = ["ts", "tsx", "d.ts", "mts", "cts"]

-- | `resolve`: one site.
resolveTs :: TsEnv -> String -> String -> Need Answer
resolveTs env from spec
  | "./" `isPrefixOf` spec || "../" `isPrefixOf` spec = case relative w dir spec of
      Just path -> pure (AFile path 1)
      Nothing -> maybe (AUnresolved OutOfScope) (`AFile` 2) <$> esmRewrite env dir spec
  | otherwise = case tsconfigRung env dir spec of
      Left waiting -> Left (waiting <> either id (const Set.empty) rest)
      Right (Left why) -> pure (AUnresolved why)
      Right (Right (Just a)) -> pure a
      Right (Right Nothing) -> rest
 where
  w = teWorld env
  dir = parentDir from
  -- a builtin is Node's before any package's; a `node:` name outside
  -- the tables is nothing Node loads (asked beside the third rung when
  -- that one waits: neither depends on the other)
  rest
    | isBuiltin spec = pure (AExternal 5)
    | "node:" `isPrefixOf` spec = pure (AUnresolved OutOfScope)
    | otherwise = workspaceRung env spec >>= maybe (bareRung env dir spec) pure

-- | `first_hit`: the exact file, then base.{ext}, then base/index.{ext}.
firstHit :: World -> String -> Maybe String
firstHit w base = case filter (member w) candidates of
  (p : _) -> Just p
  [] -> Nothing
 where
  candidates = base : [base <> "." <> e | e <- exts] <> [base <> "/index." <> e | e <- exts]

-- | `relative` (R1).
relative :: World -> String -> String -> Maybe String
relative w dir spec = joinRel dir spec >>= firstHit w

-- | `esm_rewrite` (R2): the TS twin walked and the JS twin no file.
esmRewrite :: TsEnv -> String -> String -> Need (Maybe String)
esmRewrite env dir spec = case [(stem, ts) | (js, ts) <- twins, Just stem <- [stripSuffix ("." <> js) spec]] of
  ((stem, ts) : _)
    | Just base <- joinRel dir stem
    , Just jsTwin <- joinRel dir spec ->
        let target = base <> "." <> ts
         in if member (teWorld env) target
              then (\file -> if file then Nothing else Just target) <$> isFile (teFacts env) jsTwin
              else pure Nothing
  _ -> pure Nothing
 where
  twins = [("js", "ts"), ("mjs", "mts"), ("cjs", "cts")] :: [(String, String)]

-- | `tsconfig_rung` (R3): every paths pattern that matches, each of its
-- targets with the capture in place of the first `*`; one distinct hit
-- resolves, more are ambiguous_paths; then the baseUrl join.
tsconfigRung :: TsEnv -> String -> String -> Need (Either Reason (Maybe Answer))
tsconfigRung env dir spec = verdict <$> teChain env dir
 where
  w = teWorld env
  verdict chain = case chain of
    TsNone -> Right Nothing
    TsBroken -> Left ConfigDepth
    TsOk opts ->
      let cands = [(toPathsAnchor opts, replaceFirst captured t) | (p, ts) <- toPaths opts, Just captured <- [matchPattern p spec], t <- ts]
       in case Set.toList (distinctHits w cands) of
            [path] -> Right (Just (AFile path 3))
            (_ : _ : _) -> Left AmbiguousPaths
            [] -> Right ((\path -> AFile path 3) <$> (toBaseDir opts >>= \b -> joinRel b spec >>= firstHit w))
  replaceFirst captured t = case break (== '*') t of
    (a, _ : b) -> a <> captured <> b
    _ -> t

-- | `distinct_hits`: each (anchor, target) through the join and the
-- extension order, the distinct walked hits.
distinctHits :: World -> [(String, String)] -> Set.Set String
distinctHits w cands = Set.fromList [hit | (anchor, target) <- cands, Just base <- [joinRel anchor target], Just hit <- [firstHit w base]]

-- | `match_pattern`: a single-`*` glob; the captured text ("" for an
-- exact pattern).
matchPattern :: String -> String -> Maybe String
matchPattern pattern spec = case splitOnce '*' pattern of
  Nothing -> if pattern == spec then Just "" else Nothing
  Just (pre, post) -> stripPrefix pre spec >>= stripSuffix post

-- | `workspace_rung` (R4): a walked package.json naming the package;
-- none falls on, one resolves through its exports, more are ambiguous.
workspaceRung :: TsEnv -> String -> Need (Maybe Answer)
workspaceRung env spec = pick . filter ((== Just name) . pkName) <$> teMembers env
 where
  (name, subpath) = splitBare spec
  pick members = case members of
    [] -> Nothing
    [m] -> Just (memberTarget (teWorld env) m subpath)
    _ -> Just (AUnresolved AmbiguousWorkspace)

-- | `member_target`: every string leaf of the subpath's exports entry, a
-- pattern's capture in place of every `*`; one distinct walked hit
-- resolves, more are ambiguous_exports, none is out_of_scope.
memberTarget :: World -> Package -> String -> Answer
memberTarget w m subpath = case Set.toList (distinctHits w [(pkDir m, leaf) | leaf <- leaves]) of
  [path] -> AFile path 4
  [] -> AUnresolved OutOfScope
  _ -> AUnresolved AmbiguousExports
 where
  leaves = case pkExports m >>= (`exportsEntry` subpath) of
    Just (entry, captured) -> map (concatMap (\c -> if c == '*' then captured else [c])) (stringLeaves entry)
    Nothing -> []

-- | `exports_entry`: a top-level map keyed by "./…" selects the entry —
-- the exact key first, else the one pattern key (a single `*`) whose
-- prefix and suffix enclose the subpath with something between them,
-- the longest prefix then the longest key (in bytes) winning, the last
-- of equals; a bare (non-map or condition-only) exports value IS the
-- "." entry.
exportsEntry :: Value -> String -> Maybe (Value, String)
exportsEntry exports subpath = case exports of
  Object m | any (("." `isPrefixOf`) . K.toString) (KM.keys m) -> case KM.lookup (K.fromString key) m of
    Just exact -> Just (exact, "")
    Nothing -> best [(rank, (target, captured)) | (k, target) <- KM.toList m, Just (rank, captured) <- [patternOf (K.toString k)]]
  other -> if key == "." then Just (other, "") else Nothing
 where
  key = if null subpath then "." else "./" <> subpath
  patternOf pattern = do
    (pre, post) <- splitOnce '*' pattern
    if '*' `elem` post then Nothing else Just ()
    captured <- stripPrefix pre key >>= stripSuffix post
    if null captured then Nothing else Just ((utf8Len pre, utf8Len pattern), captured)
  best ranked = case ranked of
    [] -> Nothing
    (r : rs) -> Just (snd (foldl (\a b -> if fst b >= fst a then b else a) r rs))

-- | `string_leaves`: the strings under an exports entry (conditions
-- nest).
stringLeaves :: Value -> [String]
stringLeaves v = case v of
  String s -> [K.toString (K.fromText s)]
  Object m -> concatMap stringLeaves (KM.elems m)
  _ -> []

-- | `bare_rung` (R5): declared in the nearest package.json, or present
-- under a node_modules/ of the importer's directory or an ancestor.
bareRung :: TsEnv -> String -> String -> Need Answer
bareRung env dir spec = verdict <$> both declared vendored
 where
  (name, _) = splitBare spec
  declared = maybe False ((name `elem`) . pkDeps) <$> nearestPackage (teFacts env) dir
  vendored = (/= Nothing) <$> firstTrue [((), isDir (teFacts env) d name) | d <- ancestors dir]
  verdict (d, v) = if d || v then AExternal 5 else AUnresolved OutOfScope

-- | `split_bare`: package name and subpath ("@scope/name/sub" →
-- "@scope/name", "sub").
splitBare :: String -> (String, String)
splitBare spec = go (if "@" `isPrefixOf` spec then 2 else 1 :: Int) 0
 where
  go 0 idx = (take (idx - 1) spec, drop idx spec)
  go k idx = case elemIndex '/' (drop idx spec) of
    Nothing -> (spec, "")
    Just i -> go (k - 1) (idx + i + 1)

-- | `ts_node::is_builtin`: the bare name from the first table, or
-- `node:` before a name from either.
isBuiltin :: String -> Bool
isBuiltin spec = case stripPrefix "node:" spec of
  Just name -> Set.member name nodeBuiltins || Set.member name nodePrefixOnly
  Nothing -> Set.member spec nodeBuiltins
