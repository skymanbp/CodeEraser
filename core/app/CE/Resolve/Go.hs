-- | Go rungs (design §4 row 4; moved from cli/src/graph/ladder/go.rs in
-- plan v2.33 wave W2a). Go resolves at PACKAGE granularity: an import
-- path names a directory, the node identity is (pkg_dir, ""), and
-- collapsing to one file would be a guess — so a directory only counts
-- as an importable package while it directly holds an in-scope non-
-- _test.go file (a test-only directory is not a target).
--   R1 the module-prefix walk: the LONGEST in-scope go.mod module path
--      owning the spec wins — longest is how nested modules own their
--      subtrees; two modules declaring one path is ambiguous_workspace.
--   R2 the importer's nearest module's replace directives (the longest
--      old path wins, the first of equals): a filesystem target (./ or
--      ../) maps into the corpus, a module target is rewritten and
--      retried against the module set.
--   R3 External: the stdlib table (machine-generated, importable set
--      only), or a dotted first segment matching no local module — heads
--      without a dot are reserved for the standard library, so a dotless
--      head outside the table is out_of_scope, never External.
-- //go:build-constrained files still count toward package existence.
--
-- Paths arrive as their slash pieces with empty pieces kept, so a
-- module path owns a spec exactly when its pieces are a prefix of the
-- spec's — the text's `module` or `module/…` — and a spec one slash
-- longer than its module (`module/`) has the module's own directory.
module CE.Resolve.Go (Mod (..), Rep (..), mods, resolveGo) where

import CE.Resolve.Answer
import CE.Resolve.Cost
import CE.Resolve.Request (GoFacts (..))
import CE.Resolve.Vocab (Word' (..), goStd)
import CE.Resolve.World
import Data.Bits ((.&.))
import qualified Data.IntSet as IS
import Data.List (isPrefixOf, maximumBy, minimumBy)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import qualified Data.Set as Set

-- | One go.mod that declares a module: its directory, its module path,
-- its replace directives in file order.
data Mod = Mod {mDir :: [Int], mPath :: [Int], mReps :: [Rep]}

-- | One replace directive: old path, new path or directory, whether new
-- is a filesystem path, whether new's first piece holds a dot.
data Rep = Rep {rOld :: [Int], rNew :: [Int], rIsPath :: Bool, rDotted :: Bool}

-- | The facts as modules, in the configs' order.
mods :: GoFacts -> [Mod]
mods facts = zipWith modAt [0 :: Integer ..] (goMods facts)
 where
  modAt k (n : rest) =
    Mod (ints (take (fromInteger n) rest)) (ints (drop (fromInteger n) rest)) (reps k)
  modAt _ [] = Mod [] [] []
  reps k = [rep form row | (m : form : row) <- goReplaces facts, m == k]
  rep form (n : rest) =
    Rep (ints (take (fromInteger n) rest)) (ints (drop (fromInteger n) rest)) (bit form formPath) (bit form formDotted)
  rep _ [] = Rep [] [] False False
  bit form b = form .&. b /= 0
  ints = map fromInteger

-- | One site: the importing file, its form, its pieces.
resolveGo :: World -> [Mod] -> Int -> Integer -> [Int] -> Answer
resolveGo w ms from form spec =
  firstOf [moduleRung w ms spec, replaceRung w ms from spec] (externalRung w (form .&. formDotted /= 0) spec)

-- | The remainder of a spec its module path owns, a trailing empty
-- piece (`module/`) read as none.
strip :: World -> [Int] -> [Int] -> Maybe [Int]
strip w spec m
  | m `isPrefixOf` spec = Just (case drop (length m) spec of [e] | isWord w WEmpty e -> []; rest -> rest)
  | otherwise = Nothing

-- | R1: the longest module prefix owns the import; the remainder names
-- a package directory under the module's own directory. The measuring
-- side's loop over the modules in order: a longer module replaces the
-- set, an equal one joins it (a module's length is its piece count,
-- which orders the modules owning one spec as their byte lengths do).
moduleRung :: World -> [Mod] -> [Int] -> Maybe Answer
moduleRung w ms spec = case Set.toList (snd (foldl step (0, Set.empty) ms)) of
  [] -> Nothing
  [d] -> Just (package w d 1)
  _ -> Just (AUnresolved AmbiguousWorkspace)
 where
  step (bestLen, dirs) m = case strip w spec (mPath m) of
    Nothing -> (bestLen, dirs)
    Just rest
      | len > bestLen -> (len, Set.singleton dir)
      | len == bestLen -> (bestLen, Set.insert dir dirs)
      | otherwise -> (bestLen, dirs)
     where
      len = length (mPath m)
      dir = mDir m <> rest

-- | R2: the importer's module's replace directives, the shortest
-- remainder (the longest old) wins, the first of equals.
replaceRung :: World -> [Mod] -> Int -> [Int] -> Maybe Answer
replaceRung w ms from spec = do
  owner <- deepest [m | m <- ms, mDir m `isPrefixOf` parentOf w from]
  let cands = [(rest, r) | r <- mReps owner, Just rest <- [strip w spec (rOld r)]]
  (rest, r) <- if null cands then Nothing else Just (minimumBy (comparing (length . fst)) cands)
  if rIsPath r
    then do
      base <- joinRel w (mDir owner) (map K (rNew r)) >>= traverse known
      Just (package w (base <> rest) 2)
    else
      let rewritten = rNew r <> rest
       in Just (maybe (externalRung w (rDotted r) rewritten) (withRung 2) (moduleRung w ms rewritten))
 where
  deepest [] = Nothing
  deepest xs = Just (maximumBy (comparing (length . mDir)) xs)

-- | A directory is an importable package while it directly holds an
-- in-scope non-test .go file.
package :: World -> [Int] -> Int -> Answer
package w dir rung = case M.lookup dir (wDirAt w) of
  Just d | IS.member d (wGoDirs w) -> APackage d rung
  _ -> AUnresolved OutOfScope

-- | R3: the stdlib table, or a dotted head with no local match.
externalRung :: World -> Bool -> [Int] -> Answer
externalRung w dotted spec
  | dotted || spec `elem` map (spelled w) goStd = AExternal 3
  | otherwise = AUnresolved OutOfScope
