-- | The Rust module tree's mechanics (moved from
-- cli/src/graph/ladder/rs_tree.rs in plan v2.33 W2-text stage F — each
-- function below is the Rust function of the same name): one
-- child-lookup throat, the descent that stops at the deepest walked
-- module, the multi-anchor walk and its fold, the crate roots covering a
-- file and the super-climb over file-level module owners. The rungs'
-- policy is CE.Resolve.Rs.
module CE.Resolve.RsTree (
  Child (..),
  Hits,
  childIn,
  childDir,
  descend,
  walkHits,
  settle,
  walkAll,
  coveringRoots,
  climb,
) where

import CE.Resolve.Answer (Answer (..))
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str (joinDir, parentDir, stripSuffix, utf8Len)
import CE.Resolve.World (World, member)
import Data.List (isPrefixOf, isSuffixOf)
import qualified Data.Set as Set

data Child = COne String | CBoth | CNone

-- | The (terminal, consumed) pairs every anchor's descent reaches.
type Hits = Set.Set (String, Int)

-- | `child_in`: dir/name.rs | dir/name/mod.rs; both walked is rustc's
-- own E0761 ambiguity.
childIn :: World -> String -> String -> Child
childIn w dir name = case (member w plain, member w modrs) of
  (True, True) -> CBoth
  (True, False) -> COne plain
  (False, True) -> COne modrs
  (False, False) -> CNone
 where
  plain = joinDir dir (name <> ".rs")
  modrs = joinDir dir (name <> "/mod.rs")

-- | `child`: the lookup at the declarer's file-level child directory.
child :: World -> Set.Set String -> String -> String -> Child
child w roots file = childIn w (childDir roots file)

-- | `child_dir`: a crate root or a mod.rs parents children in its own
-- directory, any other module file under dir/<stem>/.
childDir :: Set.Set String -> String -> String
childDir roots file
  | Set.member file roots || isModRs file = dir
  | otherwise = joinDir dir (trimRs (lastSeg file))
 where
  dir = parentDir file
  lastSeg = reverse . takeWhile (/= '/') . reverse
  -- `trim_end_matches(".rs")`: every trailing `.rs`
  trimRs s = maybe s trimRs (stripSuffix ".rs" s)

isModRs :: String -> Bool
isModRs file = file == "mod.rs" || "/mod.rs" `isSuffixOf` file

-- | `descend`: stopping early is not failure — the remaining segments
-- live inside the deepest matched file; the consumed count travels.
descend :: World -> Set.Set String -> String -> [String] -> Either Reason (String, Int)
descend w roots anchor = go anchor 0
 where
  go cur i [] = Right (cur, i)
  go cur i (seg : rest) = case child w roots cur seg of
    COne next -> go next (i + 1) rest
    CBoth -> Left AmbiguousPaths
    CNone -> Right (cur, i)

-- | `walk_hits`: every anchor's descent, the first refusal in anchor
-- order winning.
walkHits :: World -> Set.Set String -> [String] -> [String] -> Either Reason Hits
walkHits w roots anchors segs = Set.fromList <$> mapM (\a -> descend w roots a segs) anchors

-- | `settle`: one terminal answers (its least consumed count), none is
-- out of scope, several distinct terminals refuse.
settle :: Hits -> Int -> (Answer, Int)
settle hits rung = case Set.toList (Set.map fst hits) of
  [] -> (AUnresolved OutOfScope, 0)
  [_] -> let (path, used) = Set.findMin hits in (AFile path rung, used)
  _ -> (AUnresolved AmbiguousRoot, 0)

-- | `walk_all`.
walkAll :: World -> Set.Set String -> [String] -> [String] -> Int -> (Answer, Int)
walkAll w roots anchors segs rung = either (\why -> (AUnresolved why, 0)) (`settle` rung) (walkHits w roots anchors segs)

-- | `covering_roots`: itself when it is a root, else the roots whose
-- directory is the deepest prefix of the file (in bytes).
coveringRoots :: String -> Set.Set String -> [String]
coveringRoots from roots
  | Set.member from roots = [from]
  | otherwise = fst (foldl pick ([], 0) (Set.toList roots))
 where
  pick (best, bestLen) root
    | not (null dir || (dir <> "/") `isPrefixOf` from) = (best, bestLen)
    | null best || len > bestLen = ([root], len)
    | len == bestLen = (best <> [root], bestLen)
    | otherwise = (best, bestLen)
   where
    dir = parentDir root
    len = utf8Len dir

-- | `climb`: k×super — each step maps every anchor to the files owning
-- its parent directory; a crate root has no parent and drops out.
climb :: World -> Set.Set String -> String -> Int -> [String]
climb w roots from ups = Set.toList (go ups (Set.singleton from))
 where
  go 0 cur = cur
  go k cur =
    let next = Set.unions [owners w roots (up f) | f <- Set.toList cur, not (Set.member f roots)]
     in if Set.null next then next else go (k - 1) next
  up f = if isModRs f then parentDir (parentDir f) else parentDir f

-- | `owners`: the files whose child directory is `dir` — dir/mod.rs, the
-- sibling <dir>.rs and any crate root directly in dir.
owners :: World -> Set.Set String -> String -> Set.Set String
owners w roots dir =
  Set.fromList ([modrs | member w modrs] <> [sibling | not (null dir), member w sibling])
    `Set.union` Set.filter ((== dir) . parentDir) roots
 where
  modrs = joinDir dir "mod.rs"
  sibling = dir <> ".rs"
