-- | The Rust rungs (moved from cli/src/graph/ladder/rs.rs, rs_use.rs and
-- rs_bind.rs in plan v2.33 W2-text stage F — each function below is the
-- Rust function of the same name).
--   R1 `mod foo;` mounts dir/foo.rs | dir/foo/mod.rs under the
--      declarer's child directory, one directory per enclosing bodied
--      `mod`; a `#[path]` value answers outright, against the file's own
--      directory at file level.
--   R2 `use crate::…` walks the module tree from the covering crate
--      roots to the deepest walked module; two roots of one package
--      settle on the one whose top level holds the next segment.
--   R3 `self::` / `super::` — inline `mod` depth consumed before any file
--      climb; a bare head declared as a module in the site's namespace is
--      the same local tree, read before any crate name.
--   R4 a crate name: the toolchain's crates and declared dependencies are
--      External, a walked package by its normalized name anchors at its
--      lib root and its tree is descended.
--   R5 a walk that left segments unconsumed asks the terminal's re-export
--      surface for one hop to the definition file (a named `pub use`, a
--      glob whose module exports the name, a `pub extern crate`).
-- What the rungs read of a file's syntax tree and of the Cargo manifests
-- comes from the facts (CE.Resolve.TsFacts, CE.Resolve.RsSurface): a rung
-- that lacks one waits for it.
module CE.Resolve.Rs (RsEnv (..), Ctx, ctxFor, needAll, resolveRs) where

import CE.Resolve.Answer
import CE.Resolve.Cargo (Package (..), crateRoots, libRoot, package)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.RsSurface
import CE.Resolve.RsTree
import CE.Resolve.Str (joinDir, joinRel, parentDir, replaceChar, rustTrim, splitOnStr, stripSuffix)
import CE.Resolve.Chars (isRustWhite)
import CE.Resolve.Tables (kindModDecl, kindUse, rsBuiltin)
import CE.Resolve.TsFacts
import CE.Resolve.World (World (..), member)
import Data.List (dropWhileEnd, isPrefixOf, isSuffixOf, tails)
import qualified Data.Set as Set

-- | A file's package context: its nearest Cargo.toml read (none when it
-- does not read) and the crate roots — the manifest's and the declared
-- ones (`[graph] crate_roots`).
type Ctx = (Maybe Package, Set.Set String)

-- | What the rungs read: the world, the facts, each directory's context
-- (computed once per directory) and the walk's Cargo.toml files, read.
data RsEnv = RsEnv
  { reWorld :: World
  , reFacts :: Facts
  , reCtx :: String -> Need Ctx
  , reMembers :: Need [Package]
  }

-- | Every need side by side: all results, or every fact any waits for.
needAll :: [Need a] -> Need [a]
needAll = foldr (\a acc -> uncurry (:) <$> both a acc) (Right [])

-- | `ctx_for`: the nearest Cargo.toml walking up from a directory, read,
-- its crate roots with the declared ones beside them.
ctxFor :: Facts -> World -> Set.Set String -> String -> Need Ctx
ctxFor fx w declared dir = do
  manifest <- nearestUpBy FToml fx dir "Cargo.toml"
  pkg <- maybe (pure Nothing) (\m -> fmap (package m) <$> tomlOf fx m) manifest
  pure (pkg, maybe Set.empty (crateRoots (wFiles w)) pkg `Set.union` declared)

-- | `resolve`: one site, from its file at its 0-based row.
resolveRs :: RsEnv -> Integer -> String -> Int -> String -> Need Answer
resolveRs env kind from row spec
  | kind == kindModDecl = reCtx env (parentDir from) >>= \ctx -> modRung env ctx from row spec
  | kind == kindUse = reCtx env (parentDir from) >>= \ctx -> useRungs env ctx from row spec
  | otherwise = pure (AUnresolved Unsupported)

-- | `mod_rung` (R1): an explicit `#[path]` remap wins outright, else one
-- child lookup under the convention base.
modRung :: RsEnv -> Ctx -> String -> Int -> String -> Need Answer
modRung env (_, roots) from row name = verdict <$> rsAt (reFacts env) from row
 where
  w = reWorld env
  verdict at = case lookup name (raItems at) of
    Just (Just target) -> case joinRel (pathBase roots from at) target of
      Just path | member w path -> AFile path 1
      _ -> AUnresolved OutOfScope
    _ -> case childIn w (convBase roots from at) name of
      COne path -> AFile path 1
      CBoth -> AUnresolved AmbiguousPaths
      CNone -> AUnresolved OutOfScope

-- | `conv_base`: the declarer's child directory, then one directory per
-- enclosing bodied `mod`.
convBase :: Set.Set String -> String -> RsAt -> String
convBase roots from at = foldl joinDir (childDir roots from) (raMods at)

-- | `path_base`: the file's own directory at file level, the convention
-- base inside inline modules.
pathBase :: Set.Set String -> String -> RsAt -> String
pathBase roots from at = if null (raMods at) then parentDir from else convBase roots from at

-- | `use_rungs`: the whole spec (a hand fold's cut read back off the
-- tree), its module path walked bind-free, then the binder's one hop.
useRungs :: RsEnv -> Ctx -> String -> Int -> String -> Need Answer
useRungs env ctx from row spec = do
  whole <- case usePath spec of
    Just _ -> pure (Just spec)
    Nothing -> lookup spec . raUses <$> rsAt (reFacts env) from row
  case whole >>= usePath of
    Nothing -> pure (AUnresolved OutOfScope)
    Just (global, segs) -> do
      (out, used, walked) <- useWalk env ctx (from, row) segs global
      bound env walked out used

-- | `use_path`: the module-path prefix of a spec (before its first `{`,
-- `*` or ` as `) and whether it is global; none for a fragment a hand
-- fold cut mid-path.
usePath :: String -> Maybe (Bool, [String])
usePath spec
  | "::" `isSuffixOf` prefix && cut == length spec = Nothing
  | otherwise = Just ("::" `isPrefixOf` prefix, filter (not . null) (map rustTrim (splitOnStr "::" (trimColons prefix))))
 where
  cut = minimum (length spec : [i | (i, t) <- zip [0 ..] (tails spec), any (`isPrefixOf` t) ["{", "*", " as "]])
  prefix = dropWhileEnd isRustWhite (take cut spec)
  trimColons s = maybe s trimColons (stripSuffix "::" s)

-- | `use_walk`: the bind-free walk real sites and the binder's hop
-- share, from a file at a row; the answer, the segments consumed and the
-- segments walked.
useWalk :: RsEnv -> Ctx -> (String, Int) -> [String] -> Bool -> Need (Answer, Int, [String])
useWalk env ctx@(_, roots) (from, row) segs global = case segs of
  [] -> pure (AUnresolved OutOfScope, 0, segs)
  (h : rest)
    | global -> walked rest <$> externRung env ctx h rest
    | h == "crate" -> walked rest <$> crateWalk env roots (coveringRoots from roots) rest
    | h == "self" -> do
        at <- rsAt fx from row
        pure $
          if raDepth at > 0
            then (AFile from 3, length rest, rest)
            else walked rest (walkAll w roots [from] rest 3)
    | h == "super" -> superWalk env roots from row rest
    | otherwise ->
        localModule env ctx (from, row) h rest
          >>= maybe (walked rest <$> externRung env ctx h rest) (pure . walked rest)
 where
  (w, fx) = (reWorld env, reFacts env)
  walked rest (o, u) = (o, u, rest)

-- | `crate_walk` (R2): two roots of one package covering the site are
-- settled by the first unconsumed segment — the root that defines or
-- imports it at top level; both or neither refuse.
crateWalk :: RsEnv -> Set.Set String -> [String] -> [String] -> Need (Answer, Int)
crateWalk env roots anchors segs = case walkHits w roots anchors segs of
  Left why -> pure (AUnresolved why, 0)
  Right hits
    | Set.size (Set.map fst hits) < 2 -> pure (settle hits 2)
    | otherwise -> do
        owned <- needAll [maybe (pure False) (owns (reFacts env) path) (at used) | (path, used) <- Set.toList hits]
        let owners = Set.fromList [h | (h, True) <- zip (Set.toList hits) owned]
        pure (if Set.null owners then (AUnresolved AmbiguousRoot, 0) else settle owners 2)
 where
  w = reWorld env
  at i = if i < length segs then Just (segs !! i) else Nothing

-- | `local_module` (R3 for a bare head): a module declared in the site's
-- own namespace; a bodied one keeps every segment in this file, a
-- declaration mounts through the mod rung and the rest descends. None:
-- the head is a crate name.
localModule :: RsEnv -> Ctx -> (String, Int) -> String -> [String] -> Need (Maybe (Answer, Int))
localModule env ctx@(_, roots) (from, row) h rest = do
  at <- rsAt (reFacts env) from row
  case [(bodied, declRow) | (n, bodied, declRow) <- raNs at, n == h] of
    [] -> pure Nothing
    ((True, _) : _) -> pure (Just (AFile from 3, length rest))
    ((False, declRow) : _) -> Just . descended <$> modRung env ctx from declRow h
 where
  descended mounted = case mounted of
    AFile path _ -> either (\why -> (AUnresolved why, 0)) (\(p, u) -> (AFile p 3, u)) (descend (reWorld env) roots path rest)
    other -> (other, 0)

-- | `super_walk`: the ups consume inline-module depth before any file
-- climb.
superWalk :: RsEnv -> Set.Set String -> String -> Int -> [String] -> Need (Answer, Int, [String])
superWalk env roots from row rest = do
  at <- rsAt (reFacts env) from row
  let depth = raDepth at
      anchors = if ups <= depth then [from] else climb w roots from (ups - depth)
      (o, u) = walkAll w roots anchors tl 3
  pure (o, u, tl)
 where
  w = reWorld env
  ups = 1 + length (takeWhile (== "super") rest)
  tl = drop (ups - 1) rest

-- | `extern_rung` (R4): a toolchain crate is External; a walked package
-- by its normalized name anchors at its lib root and descends; a
-- declared dependency is External; anything else is out of scope.
externRung :: RsEnv -> Ctx -> String -> [String] -> Need (Answer, Int)
externRung env (pkg, roots) name rest
  | Set.member name rsBuiltin = pure (AExternal 4, 0)
  | otherwise = verdict . filter ((== Just name) . fmap norm . cpName) <$> reMembers env
 where
  w = reWorld env
  norm = replaceChar '-' '_'
  verdict members = case members of
    [m] -> case libRoot (wFiles w) m of
      Just root -> either (\why -> (AUnresolved why, 0)) (\(p, u) -> (AFile p 4, u)) (descend w (Set.insert root roots) root rest)
      Nothing -> (AUnresolved OutOfScope, 0)
    []
      | maybe False (any ((== name) . norm) . cpDeps) pkg -> (AExternal 4, 0)
      | otherwise -> (AUnresolved OutOfScope, 0)
    _ -> (AUnresolved AmbiguousWorkspace, 0)

-- | `bound` (R5): a single-terminal walk that left segments unconsumed
-- consults the terminal's surface for one hop and answers the definition
-- file; an unbound, ambiguous or self-pointing hop keeps the file.
bound :: RsEnv -> [String] -> Answer -> Int -> Need Answer
bound env walked out used = case (out, drop used walked) of
  (AFile path rung, name : tl) -> do
    b <- bindsTo (reFacts env) path name
    target <- case b of
      Just (Named segs row) -> hop env path segs tl row
      Just (Globbed globs) -> globbed env path name tl globs
      Nothing -> pure Nothing
    pure $ case target of
      Just found | found /= path -> AVia found rung
      _ -> out
  _ -> pure out

-- | `hop`: the bound path, then the tail, walked bind-free from the
-- facade at the entry's own row; a global path walks as a crate name.
hop :: RsEnv -> String -> [String] -> [String] -> Int -> Need (Maybe String)
hop env facade segs tl row = do
  ctx <- reCtx env (parentDir facade)
  (o, _, _) <- useWalk env ctx (facade, row) (drop (fromEnum global) segs <> tl) global
  pure $ case o of
    AFile path _ -> Just path
    _ -> Nothing
 where
  global = take 1 segs == [""]

-- | `globbed`: exactly one glob whose module exports the name answers the
-- walk of its path, the name and the tail; several, or none, keep the
-- file. The globs are asked in order until a second carries the name.
globbed :: RsEnv -> String -> String -> [String] -> [([String], Int)] -> Need (Maybe String)
globbed env facade name tl globs = carrying [] globs >>= \found -> case found of
  [(segs, row)] -> hop env facade segs (name : tl) row
  _ -> pure Nothing
 where
  carrying found gs
    | length found == 2 = pure found
    | otherwise = case gs of
        [] -> pure found
        (g@(segs, row) : more) -> do
          target <- hop env facade segs [] row
          carries <- case target of
            Just m | m /= facade -> exports (reFacts env) m name
            _ -> pure False
          carrying (if carries then found <> [g] else found) more
