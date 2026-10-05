-- | The inheritance rung of a simple Java type name (plan v2.33 W2-text
-- stage C; moved from cli/src/graph/ladder/java_inherit.rs, each
-- function named after the Rust one it replaces): a member type the
-- class enclosing the site inherits from a supertype another walked file
-- declares. JLS 6.4.1 puts the members a class declares or inherits in
-- scope over its whole body, innermost class first and ahead of the
-- unit's imports, so the site's enclosing types are asked innermost
-- first; each one's supertypes, as its header writes them, are resolved
-- to the declaring file in THAT type's own unit — a type of the same
-- unit, else what the unit's imports, package and star imports name
-- (CE.Resolve.JavaAt `declared`), else a fully qualified name over the
-- package index — and the nearest supertype level declaring a member
-- type of the name answers (JLS 8.5); two supertypes each declaring one
-- are ambiguous_paths. A supertype no walked file declares contributes
-- nothing.
module CE.Resolve.JavaInherit (inherited) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.JavaAt (classFiles, declared)
import CE.Resolve.JavaHeader
import CE.Resolve.JavaPick (JEnv (..), declaredOne, settle)
import CE.Resolve.Str (splitOn)
import Control.Monad (foldM)
import Data.List (find, intercalate)
import Data.Maybe (isJust, listToMaybe)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | `inherited`: the first enclosing type (innermost first) whose
-- supertype chains declare a member type `name`.
inherited :: JEnv -> String -> String -> JHeader -> Int -> Maybe Answer
inherited env from name h line =
  listToMaybe
    [ a
    | decl <- enclosing (hTypes h) line
    , let hits = fst (inSupers env from decl name Set.empty)
    , not (Set.null hits)
    , Just a <- [settle env (Set.toList hits) 3 AmbiguousPaths]
    ]

-- | `enclosing`: the types enclosing `line`, innermost first.
enclosing :: [JType] -> Int -> [JType]
enclosing types line = concat [enclosing (tMembers t) line <> [t] | t <- types, tFirst t <= line, line <= tLast t]

-- | `in_supers`: the files whose declaration of a supertype of `decl` —
-- its name read in `file`'s own unit — holds a member type `name`; each
-- chain answers at its nearest such level, a chain met twice (a
-- diamond, a cycle) is read once (the `seen` set threads through every
-- chain, as the Rust one did).
inSupers :: JEnv -> String -> JType -> String -> Set.Set String -> (Set.Set String, Set.Set String)
inSupers env file decl name seen0 = foldl step (Set.empty, seen0) (tSupers decl)
 where
  step (hits, seen) written = case declaredType env file written of
    Nothing -> (hits, seen)
    Just (sfile, sdecl)
      | Set.member key seen -> (hits, seen)
      | any ((== name) . tName) (tMembers sdecl) -> (Set.insert sfile hits, seen')
      | otherwise -> let (more, seen'') = inSupers env sfile sdecl name seen' in (Set.union hits more, seen'')
     where
      key = sfile <> "\0" <> tName sdecl <> "\0" <> show (tFirst sdecl)
      seen' = Set.insert key seen

-- | `declared_type`: the walked file and declaration a supertype name
-- written in `file` names — its head a type of the unit, else what the
-- unit names around it, else the longest prefix the package index
-- holds; the segments after the head descend member types.
declaredType :: JEnv -> String -> String -> Maybe (String, JType)
declaredType env file written = do
  unit <- M.lookup file (jHeaders env)
  (path, at) <- headOf unit
  target <- M.lookup path (jHeaders env)
  decl <- oneDeclared (hTypes target) (segs !! at)
  found <- foldM (\d seg -> find ((== seg) . tName) (tMembers d)) decl (drop (at + 1) segs)
  pure (path, found)
 where
  segs = splitOn '.' written
  first = takeWhile (/= '.') written
  headOf unit
    | isJust (oneDeclared (hTypes unit) first) = Just (file, 0)
    | Just (AFile path _) <- declared env file first unit = Just (path, 0)
    | otherwise = qualifiedFile env file segs

-- | `one_declared`: the unit's top-level type of the name, else its one
-- member type of the name at any depth.
oneDeclared :: [JType] -> String -> Maybe JType
oneDeclared types name = case find ((== name) . tName) types of
  Just top -> Just top
  Nothing -> case nested types of
    [one] -> Just one
    _ -> Nothing
 where
  nested ts = concat [[t | tName t == name] <> nested (tMembers t) | t <- ts]

-- | `qualified_file`: the longest prefix `p.C` whose package holds a
-- file declaring `C` — that file (the declared roots' one among
-- several; several none of which they hold answer nothing) and the
-- index of `C` in the segments.
qualifiedFile :: JEnv -> String -> [String] -> Maybe (String, Int)
qualifiedFile env from segs = go [length segs, length segs - 1 .. 2]
 where
  go [] = Nothing
  go (k : ks) = case classFiles env from (intercalate "." (take (k - 1) segs)) (segs !! (k - 1)) of
    [] -> go ks
    files -> either (const Nothing) (\f -> Just (f, k - 1)) (declaredOne env (Set.fromList files))
