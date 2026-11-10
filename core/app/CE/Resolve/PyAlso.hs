-- | The Python import's other references (lane v235_pyimport, proto
-- 9.1.0): what CPython executes beside the module a site's ladder
-- answers (CE.Resolve.Py), as the reply's `also` rows — each an edge of
-- the site's own kind, never a ledger row. Three rules, each from
-- importlib's own semantics:
--
-- * ancestors: importing a dotted module executes every parent
--   package first, so the answer's frame (the source root it was found
--   under, or the directory a relative import climbed to) adds each
--   strict prefix of the dotted path that is a package there — its
--   `__init__.py` in the corpus; an absent one is a namespace package
--   and adds nothing;
-- * names: `from M import n` imports `M.n` as a submodule when it is
--   one, so each name is tried as `M.n` under every frame — package
--   before module (`moduleAt`), its own ancestors with it; a miss is
--   an attribute and adds nothing; two roots that both hold `M.n` both
--   draw, never an ambiguity refusal (the safe direction);
-- * a star import: `from pkg import *` with no names of its own, whose
--   answer is a package `__init__.py` binding `__all__` (`py.all`, lane
--   v235_graphtrust), imports each name its literal bindings and edits
--   list as `from pkg import n` would — the names rule above
--   (`starNames`); when any binding or edit is one no literal reads,
--   the names it adds are unknown: the literal names still draw, and the
--   site carries one more ledger row (`opaque`, CE.Resolve);
-- * the importer's package: a relative import proves its file is a
--   package member, whose package chain CPython has executed — each
--   `__init__.py` from the importer's directory upward while contiguous.
--
-- The primary answer is untouched (computed apart, in CE.Resolve); a
-- row never names the site's own file or its primary target, one row
-- per target at its lowest rung, in target order.
module CE.Resolve.PyAlso (alsoPy, starNames) where

import CE.Resolve.Answer (Answer (..))
import CE.Resolve.Py (PyProject, moduleAt, packages, relativeFrame, sourceRoots)
import CE.Resolve.Str (ancestors, joinDir, parentDir)
import CE.Resolve.World (World, member)
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M

-- | A place a dotted path is read under: the base directory, the dotted
-- path, the rung an answer there carries.
type Frame = (String, String, Int)

-- | One Python site's `also` rows — the site `(file, spec, names)` and
-- its answer — as `(target, rung)` in target order.
alsoPy :: World -> Maybe PyProject -> (String, String, [String]) -> Answer -> [(String, Int)]
alsoPy w py (from, spec, names) primary =
  M.toList (M.fromListWith min [(t, r) | (t, r) <- found, t /= from, Just t /= target])
 where
  frames = framesOf py from spec
  target = case primary of
    AFile p _ -> Just p
    _ -> Nothing
  found = ancestorsOfPrimary <> concatMap named frames <> chain
  ancestorsOfPrimary = case primary of
    AFile p r -> [(a, r) | (base, dotted, _) <- frames, let pre = packages w base dotted, Just p `elem` (moduleAt w base dotted : map Just pre), a <- pre]
    _ -> []
  -- a hit brings its own parents under its frame; a miss is an
  -- attribute and brings nothing
  named (base, dotted, rung) =
    concat
      [ case moduleAt w base dn of
          Just t -> (t, rung) : [(p, rung) | p <- packages w base dn]
          Nothing -> []
      | n <- names
      , let dn = if null dotted then n else dotted <> "." <> n
      ]
  chain
    | "." `isPrefixOf` spec = [(p, 1) | p <- takeWhile (member w) [joinDir d "__init__.py" | d <- ancestors (parentDir from)]]
    | otherwise = []

-- | A site with no names of its own, by its answer: the literal names
-- of the `__all__` of the package `__init__.py` it answers and whether
-- that `__all__` is opaque; ([], False) for any other answer.
starNames :: M.Map String ([String], Bool) -> Answer -> ([String], Bool)
starNames alls a = case a of
  AFile p _ -> M.findWithDefault ([], False) p alls
  _ -> ([], False)

-- | The frames a spec is read under: each source root for an absolute
-- spec, the climbed directory for a relative one (none above the root).
framesOf :: Maybe PyProject -> String -> String -> [Frame]
framesOf py from spec
  | "." `isPrefixOf` spec = [(dir, rest, 1) | Just (dir, rest) <- [relativeFrame from spec]]
  | otherwise = [(root, spec, 2) | root <- sourceRoots py]
