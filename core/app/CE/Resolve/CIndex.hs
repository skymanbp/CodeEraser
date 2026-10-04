-- | What the compile databases say about each C-family file (plan
-- v2.30 step 5b, item 14; moved from cli/src/graph/ladder/c_index.rs
-- and c_search.rs in plan v2.33 wave W2a). The measuring side finds the
-- databases clangd would find and reads each entry's argv into a chain
-- of searched places; this module seats the files, covers the rest
-- with the nearest `compile_flags.txt`, and — since a database lists
-- translation units and a header has no entry — walks the include
-- closure of each chain: a header is compiled with the chain of every
-- translation unit whose includes reach it, so its own `#include`
-- lines are answered under those chains (clangd guesses a nearby
-- unit's command instead, which the register's no-guess rule forbids).
--
-- The closure's work order is the measuring side's: the seats' files
-- in path order (file ids ascend in path order), the last popped
-- first, each file's include lines in order, each line's targets in
-- path order. Under the MSVC dialect the quoted form also asks the
-- directories of the files on the include stack, which grows with the
-- links a pass adds, so an MSVC chain's pass repeats until it adds
-- nothing.
module CE.Resolve.CIndex (
  Search (..),
  Chain (..),
  Index (..),
  index,
  chainsOf,
  stackOf,
  along,
  forcedArcs,
  ancestors,
) where

import CE.Resolve.Cost
import CE.Resolve.Request (CFacts (..))
import CE.Resolve.Vocab (Affix (..), Word' (..))
import CE.Resolve.World
import Control.Applicative ((<|>))
import Data.Array (Array, listArray, (!))
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M
import Data.Maybe (listToMaybe)
import qualified Data.Set as Set

-- | One searched place: a directory joined with the spelling, or a
-- framework directory (`A/B.h` as `A.framework/Headers/B.h`).
data Search = SDir [Int] | SFw [Int]

-- | A forced include: a spelling to search, a placed in-tree path, or a
-- path placed outside the tree.
data Forced = FRel [Pc] | FPlaced [Int] | FOutside

data Chain = Chain
  { chMsvc :: Bool
  , chOwn :: Bool
  , chQuote :: [Search]
  , chBracket :: [Search]
  , chSystem :: [Search]
  , chForced :: [Forced]
  }

-- | A seat: a file, the entry's working directory (Nothing = outside
-- the tree), its chain, whether a translation unit seats it.
data Seat = Seat {sFile :: Int, sDir :: Maybe [Int], sChain :: Int, sUnit :: Bool}

data Index = Index
  { ixChains :: Array Int Chain
  , ixSeats :: [Seat]
  , ixUnits :: IM.IntMap IS.IntSet
  , ixParents :: Array Int (IM.IntMap IS.IntSet)
  , ixRoots :: [[Int]]
  , ixIncludes :: IM.IntMap [(Bool, [Pc])]
  }

-- | The index of one request: chains, seats, cover, closure.
index :: World -> CFacts -> Index
index w facts = ix
 where
  ix = Index chains seats units parents roots includes
  n = length (cChains facts)
  chains = listArray (0, n - 1) (zipWith (chainAt facts) [0 ..] (cChains facts))
  json = [Seat (i f) (if placed == 1 then Just (ints d) else Nothing) (i c) True | (f : c : placed : d) <- cSeats facts]
  seats = json <> cover w facts (IS.fromList (map sFile json))
  units = IM.fromListWith IS.union [(sFile s, IS.singleton (sChain s)) | s <- seats]
  parents = listArray (0, n - 1) [closeChain w ix c | c <- [0 .. n - 1]]
  roots = map ints (cRoots facts)
  includes = IM.fromListWith (flip (<>)) [(i f, [(s == formSystem, map (K . i) p)]) | (f : s : p) <- cIncludes facts]
  i = fromInteger
  ints = map fromInteger

-- | One chain from its rows.
chainAt :: CFacts -> Integer -> [Integer] -> Chain
chainAt facts c row =
  Chain (flag 0) (flag 1) (places 0) (places 1) (places 2) [forced how p | (c' : how : p) <- cForced facts, c' == c]
 where
  flag k = take 1 (drop k row) == [1]
  places cls = [place kind d | (c' : cls' : kind : d) <- cSearches facts, c' == c, cls' == cls]
  place kind d = (if kind == 1 then SFw else SDir) (map fromInteger d)
  forced how p
    | how == forcedPlaced = FPlaced (map fromInteger p)
    | how == forcedOutside = FOutside
    | otherwise = FRel (map (K . fromInteger) p)

-- | The files a compile_flags.txt covers: every C-family file with no
-- entry whose nearest database directory holds one — clangd's rule, the
-- nearest ancestor holding any database decides, a JSON one first.
cover :: World -> CFacts -> IS.IntSet -> [Seat]
cover w facts seated =
  [ Seat f (Just d) (flags M.! d) (unit f)
  | f <- [0 .. wFileCount w - 1]
  , wWalked w ! f
  , wFileLang w ! f `elem` [langC, langCpp]
  , not (IS.member f seated)
  , Just d <- [nearest (ancestors (parentOf w f))]
  ]
 where
  jsonDirs = Set.fromList (map (map fromInteger) (cJsonDirs facts))
  flags = M.fromList [(map fromInteger d, fromInteger c) | (c : d) <- cFlags facts]
  nearest (d : ds)
    | Set.member d jsonDirs = Nothing
    | M.member d flags = Just d
    | otherwise = nearest ds
  nearest [] = Nothing
  unit f = any (\a -> endsWith w a (wFileBase w ! f)) [AC, ACc, ACpp, ACxx]

-- | A directory and every ancestor of it up to the root, nearest first.
ancestors :: [Int] -> [[Int]]
ancestors [] = [[]]
ancestors d = d : ancestors (init d)

-- | The closure of one chain: passes until one adds nothing (MSVC), or
-- one pass (GNU).
closeChain :: World -> Index -> Int -> IM.IntMap IS.IntSet
closeChain w ix c = go IM.empty
 where
  go ps = case pass w ix c ps of
    (ps', True) | chMsvc (ixChains ix ! c) -> go ps'
    (ps', _) -> ps'

-- | One closure pass from the chain's seats: forced includes first,
-- then every include line of every reached file. Whether it added a
-- parent link.
pass :: World -> Index -> Int -> IM.IntMap IS.IntSet -> (IM.IntMap IS.IntSet, Bool)
pass w ix c ps0 = (ps, grew)
 where
  chain = ixChains ix ! c
  seats = [(sFile s, sDir s) | s <- ixSeats ix, sChain s == c]
  seen0 = IS.fromList (map fst seats)
  start = (ps0, seen0, reverse (IS.toAscList seen0), False)
  forcedLinks = [(f, h) | (f, d) <- seats, fz <- chForced chain, Just h <- [forcedTarget w chain d fz]]
  (ps, _, _, grew) = drain (foldl link start forcedLinks)
  drain st@(_, _, [], _) = st
  drain (p, seen, x : rest, g) = drain (foldl link (p, seen, rest, g) [(x, t) | t <- targetsOf p x])
  targetsOf p x =
    let stack = stackOf w chain p x
     in concat [targets w ix chain x inc stack | inc <- IM.findWithDefault [] x (ixIncludes ix)]

-- | One parent link; a file reached for the first time joins the work.
link :: (IM.IntMap IS.IntSet, IS.IntSet, [Int], Bool) -> (Int, Int) -> (IM.IntMap IS.IntSet, IS.IntSet, [Int], Bool)
link (ps, seen, work, grew) (from, to) =
  ( IM.insertWith IS.union to (IS.singleton from) ps
  , IS.insert to seen
  , if IS.member to seen then work else to : work
  , grew || not present
  )
 where
  present = maybe False (IS.member from) (IM.lookup to ps)

-- | Every target one include line of `from` reaches under one chain:
-- the file's own directory (quoted form, unless `-I-` inhibits it), the
-- declared roots, then the chain; the first step with a hit answers.
targets :: World -> Index -> Chain -> Int -> (Bool, [Pc]) -> Set.Set [Int] -> [Int]
targets w ix chain from (system, name) stack
  | name == [K (word w WEmpty)] = []
  | not system && chOwn chain, Just p <- inScope w (parentOf w from) name = [p]
  | not (null declared) = declared
  | otherwise = along w chain system name stack
 where
  declared = hits [inScope w r name | r <- ixRoots ix]

-- | The hits of the first non-empty step along one chain: the quoted
-- form asks the include stack's directories (cl's second step) then
-- the `-iquote` class; both forms then the `-I` class and the system
-- class, first hit.
along :: World -> Chain -> Bool -> [Pc] -> Set.Set [Int] -> [Int]
along w chain system name stack
  | not system, not (null stacked) = stacked
  | not system, Just h <- firstHit (chQuote chain) = [h]
  | otherwise = maybe [] pure (firstHit (chBracket chain <> chSystem chain))
 where
  stacked = hits [inScope w d name | d <- Set.toList stack]
  firstHit places = listToMaybe [h | s <- places, Just h <- [under w s name]]

-- | The walked file one searched place yields for a name; a framework
-- directory answers `A/B.h` as `A.framework/Headers/B.h`, then
-- `PrivateHeaders`, and a name with no `/` is no framework header.
under :: World -> Search -> [Pc] -> Maybe Int
under w (SDir d) name = inScope w d name
under w (SFw d) (fw : rest@(_ : _)) =
  listToMaybe [h | sub <- [WHeaders, WPrivateHeaders], Just h <- [inScope w d (glue w (affix w AFramework) fw : K (word w sub) : rest)]]
under _ (SFw _) _ = Nothing

-- | A forced include's target: a placed path, else the entry's
-- directory, then the quoted chain's first hit in path order.
forcedTarget :: World -> Chain -> Maybe [Int] -> Forced -> Maybe Int
forcedTarget w _ _ (FPlaced p) = fileAt w (map K p)
forcedTarget _ _ _ FOutside = Nothing
forcedTarget w chain dir (FRel spec) =
  (dir >>= \d -> inScope w d spec) <|> listToMaybe (along w chain False spec Set.empty)

-- | The chains a file compiles under: its own seats, else the chains
-- whose closure reaches it.
chainsOf :: Index -> Int -> [Int]
chainsOf ix f = case IM.lookup f (ixUnits ix) of
  Just own -> IS.toList own
  Nothing -> [c | (c, ps) <- zip [0 ..] (elemsOf (ixParents ix)), IM.member f ps]
 where
  elemsOf a = foldr (:) [] a

-- | The directories of the files on the include paths from a chain's
-- units down to a file (the MSVC dialect; empty otherwise).
stackOf :: World -> Chain -> IM.IntMap IS.IntSet -> Int -> Set.Set [Int]
stackOf w chain ps file
  | not (chMsvc chain) = Set.empty
  | otherwise = Set.fromList (map (parentOf w) (IS.toList (up IS.empty [file])))
 where
  up seen [] = seen
  up seen (f : rest) =
    let next = [p | p <- IS.toList (IM.findWithDefault IS.empty f ps), not (IS.member p seen)]
     in up (foldr IS.insert seen next) (next <> rest)

-- | The forced includes of every translation unit, resolved: the
-- (unit, header) arcs a build declares outside the source text.
forcedArcs :: World -> Index -> [(Int, Int)]
forcedArcs w ix =
  Set.toAscList . Set.fromList $
    [ (sFile s, h)
    | s <- ixSeats ix
    , sUnit s
    , let chain = ixChains ix ! sChain s
    , fz <- chForced chain
    , Just h <- [forcedTarget w chain (sDir s) fz]
    ]
