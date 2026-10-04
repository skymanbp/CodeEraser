-- | What the compile databases say about each C-family file (plan v2.30
-- step 5b, item 14; moved from cli/src/graph/ladder/c_index.rs and
-- c_search.rs in plan v2.33 wave W2a, on text since W2-text — each
-- function below is the Rust function of the same name). The measuring
-- side finds the databases clangd would find; this module seats the
-- files their entries name (CE.Resolve.CompDb read them), covers the rest
-- with the nearest `compile_flags.txt`, and — since a database lists
-- translation units and a header has no entry — walks the include closure
-- of each chain: a header is compiled with the chain of every translation
-- unit whose includes reach it, so its own `#include` lines are answered
-- under those chains (clangd guesses a nearby unit's command instead,
-- which the register's no-guess rule forbids). Under the MSVC dialect the
-- quoted form also asks the directories of the files on the include
-- stack, which grows with the links a pass adds, so an MSVC chain's pass
-- repeats until it adds nothing.
module CE.Resolve.CIndex (
  Env (..),
  Found (..),
  Index (..),
  index,
  chainsOf,
  stack,
  along,
  declared,
  forcedArcs,
  form,
) where

import CE.Lang (languages)
import CE.Lang.Spec (Language (..))
import CE.Resolve.CompDb (Entry (..), isAbsolute, relativize)
import CE.Resolve.Flags (Chain (..), Search (..))
import CE.Resolve.Str
import CE.Resolve.World
import Control.Applicative ((<|>))
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | What the index reads besides the databases: the tree, the declared
-- `[graph.search_roots] c`, each walked C-family file's include
-- specifiers, the root's text (an absolute forced include is placed
-- against it).
data Env = Env {eWorld :: World, eRoots :: [String], eIncludes :: M.Map String [String], eBase :: String}

-- | One database a probe found, read: a JSON one's entries (by its path,
-- read once however many probes find it), or a flags file's directory
-- and chain (Nothing when unreadable).
data Found = FJson String String [Entry] | FFlags String (Maybe Chain)

data Seat = Seat {sFile :: String, sDir :: Maybe String, sChain :: Int, sUnit :: Bool}

data Index = Index
  { ixChains :: [Chain]
  , ixSeats :: [Seat]
  , ixUnits :: M.Map String (Set.Set Int)
  , ixParents :: [M.Map String (Set.Set String)]
  }

-- | `c_search::form`: a spec's name and whether it is the `<…>` form.
form :: String -> (String, Bool)
form ('<' : rest) | Just inner <- stripSuffix ">" rest = (inner, True)
form spec = (spec, False)

-- | `index`: gather, then close every chain.
index :: Env -> [Found] -> Index
index env founds = ix {ixParents = [close env ix c | c <- [0 .. length (ixChains ix) - 1]]}
 where
  ix = gather env founds

-- | `gather`: JSON seats in found order, flags files by directory, then
-- the cover; a chain's id is its first sight.
gather :: Env -> [Found] -> Index
gather env founds = cover env flags jsonDirs (Index chains seats units []) ids
 where
  (chains, seats, units, ids, jsonDirs, flags, _) = foldl step ([], [], M.empty, M.empty, Set.empty, M.empty, Set.empty) founds
  step (cs, ss, us, is, jd, fl, readSet) found = case found of
    FFlags dir (Just c) -> (cs, ss, us, is, jd, M.insert dir c fl, readSet)
    FFlags _ Nothing -> (cs, ss, us, is, jd, fl, readSet)
    FJson dir rel entries
      | Set.member rel readSet -> (cs, ss, us, is, Set.insert dir jd, fl, readSet)
      | otherwise ->
          let (cs', ss', us', is') = foldl (seatEntry env) (cs, ss, us, is) entries
           in (cs', ss', us', is', Set.insert dir jd, fl, Set.insert rel readSet)

-- | One JSON entry seated when its unit is a walked file.
seatEntry :: Env -> ([Chain], [Seat], M.Map String (Set.Set Int), M.Map Chain Int) -> Entry -> ([Chain], [Seat], M.Map String (Set.Set Int), M.Map Chain Int)
seatEntry env acc@(cs, ss, us, is) e
  | member (eWorld env) (eUnit e) =
      let (c, cs', is') = chainId (eChain e) cs is
       in (cs', ss <> [Seat (eUnit e) (eDir e) c True], M.insertWith Set.union (eUnit e) (Set.singleton c) us, is')
  | otherwise = acc

-- | `chain_id`: one chain's id, structural equality deciding.
chainId :: Chain -> [Chain] -> M.Map Chain Int -> (Int, [Chain], M.Map Chain Int)
chainId c cs is = case M.lookup c is of
  Just i -> (i, cs, is)
  Nothing -> (length cs, cs <> [c], M.insert c (length cs) is)

-- | `cover`: every walked C-family file with no seat whose nearest
-- database directory holds a flags file (a JSON one first stops it),
-- seated with that file's chain, in path order.
cover :: Env -> M.Map String Chain -> Set.Set String -> Index -> M.Map Chain Int -> Index
cover env flags jsonDirs ix ids0 = ix {ixChains = cs, ixSeats = ixSeats ix <> seats, ixUnits = us}
 where
  covered = [(f, d) | f <- Set.toList (wFiles (eWorld env)), isC f, not (M.member f (ixUnits ix)), Just d <- [nearest f]]
  (cs, seats, us, _) = foldl seatOne (ixChains ix, [], ixUnits ix, ids0) covered
  seatOne (cs0, ss, us0, is) (f, d) =
    let (c, cs', is') = chainId (flags M.! d) cs0 is
     in (cs', ss <> [Seat f (Just d) c (isUnit f)], M.insertWith Set.union f (Set.singleton c) us0, is')
  nearest f = firstJust [stop d | d <- ancestors (parentDir f)]
  stop d
    | Set.member d jsonDirs = Just Nothing
    | M.member d flags = Just (Just d)
    | otherwise = Nothing
  firstJust xs = case [x | Just x <- xs] of
    (x : _) -> x
    [] -> Nothing

-- | `compdb_find::is_c`: the path's extension (`Path::extension`: after
-- the basename's last `.`, a non-empty stem before it) is a C or C++ one.
isC :: String -> Bool
isC path = maybe False (`elem` cExts) (extension (baseName path))
 where
  cExts = concat [lgExts l | l <- languages, lgName l `elem` ["c", "cpp"]]
  extension base
    | base == ".." = Nothing
    | otherwise = case break (== '.') (reverse base) of
        (ext, '.' : stem) | not (null stem) -> Just (reverse ext)
        _ -> Nothing

-- | `compdb_find::is_unit`: `.c` / `.cc` / `.cpp` / `.cxx` after the
-- path's last `.`.
isUnit :: String -> Bool
isUnit path = case break (== '.') (reverse path) of
  (ext, '.' : _) -> reverse ext `elem` ["c", "cc", "cpp", "cxx"]
  _ -> False

-- | `close`: one chain's parent links — passes until one adds nothing
-- (MSVC), or one pass (GNU).
close :: Env -> Index -> Int -> M.Map String (Set.Set String)
close env ix c = go M.empty
 where
  ch = ixChains ix !! c
  go ps = case pass env ix c ps of
    (ps', True) | chMsvc ch -> go ps'
    (ps', _) -> ps'

-- | `pass`: forced includes first, then a LIFO worklist from the seats'
-- files (the largest path popped first), each popped file's include
-- lines in order, each line's targets in path order. Whether a parent
-- link was added.
pass :: Env -> Index -> Int -> M.Map String (Set.Set String) -> (M.Map String (Set.Set String), Bool)
pass env ix c ps0 = (ps, grew)
 where
  ch = ixChains ix !! c
  seats = [(sFile s, sDir s) | s <- ixSeats ix, sChain s == c]
  seen0 = Set.fromList (map fst seats)
  forcedLinks = [(f, h) | (f, d) <- seats, spec <- chForced ch, Just h <- [forced env d spec ch]]
  start = foldl link (ps0, seen0, Set.toAscList seen0, False) forcedLinks
  (ps, _, _, grew) = drain start
  drain st@(_, _, [], _) = st
  drain (p, seen, work, g) =
    let x = last work
        st = stack ch p x
        ts = concat [Set.toList (targets env x spec ch st) | spec <- M.findWithDefault [] x (eIncludes env)]
     in drain (foldl link (p, seen, init work, g) [(x, t) | t <- ts])

-- | `link`: one parent link; a file reached for the first time joins the
-- work (pushed on the end, popped next).
link :: (M.Map String (Set.Set String), Set.Set String, [String], Bool) -> (String, String) -> (M.Map String (Set.Set String), Set.Set String, [String], Bool)
link (ps, seen, work, grew) (from, to) =
  ( M.insertWith Set.union to (Set.singleton from) ps
  , Set.insert to seen
  , if Set.member to seen then work else work <> [to]
  , grew || not present
  )
 where
  present = maybe False (Set.member from) (M.lookup to ps)

-- | `chains_of`: the chains a file compiles under — its own seats, else
-- the chains whose closure reaches it.
chainsOf :: Index -> String -> Set.Set Int
chainsOf ix f = case M.lookup f (ixUnits ix) of
  Just own -> own
  Nothing -> Set.fromList [c | (c, ps) <- zip [0 ..] (ixParents ix), M.member f ps]

-- | `stack`: the directories of the files on the include paths from a
-- chain's units down to a file (the MSVC dialect; empty otherwise).
stack :: Chain -> M.Map String (Set.Set String) -> String -> Set.Set String
stack ch ps file
  | not (chMsvc ch) = Set.empty
  | otherwise = Set.map parentDir (up Set.empty [file])
 where
  up seen [] = seen
  up seen (f : rest) =
    let next = [p | p <- Set.toList (M.findWithDefault Set.empty f ps), not (Set.member p seen)]
     in up (foldr Set.insert seen next) (reverse next <> rest)

-- | `paths::declared`: the walked files a name names under each declared
-- `c` root.
declared :: Env -> String -> Set.Set String
declared env name = Set.fromList [p | dir <- eRoots env, Just p <- [inScope (eWorld env) dir name]]

-- | `c_search::under`: the walked file one searched place yields; a
-- framework directory answers `A/B.h` as `A.framework/Headers/B.h`, then
-- `PrivateHeaders`, and a name with no `/` is no framework header.
under :: World -> Search -> String -> Maybe String
under w (SDir dir) name = inScope w dir name
under w (SFramework dir) name = do
  (fw, rest) <- splitOnce '/' name
  case [p | sub <- ["Headers", "PrivateHeaders"], Just p <- [inScope w dir (fw <> ".framework/" <> sub <> "/" <> rest)]] of
    (p : _) -> Just p
    [] -> Nothing

-- | `c_search::along`: the hits of the first non-empty step along one
-- chain — the quoted form asks the include stack's directories then the
-- quote class; both forms then the bracket and system classes, first hit.
along :: World -> Chain -> Bool -> String -> Set.Set String -> Set.Set String
along w ch system name st
  | not system, not (Set.null stacked) = stacked
  | not system, Just h <- firstHit (chQuote ch) = Set.singleton h
  | otherwise = maybe Set.empty Set.singleton (firstHit (chBracket ch <> chSystem ch))
 where
  stacked = Set.fromList [p | d <- Set.toList st, Just p <- [inScope w d name]]
  firstHit places = case [h | s <- places, Just h <- [under w s name]] of
    (h : _) -> Just h
    [] -> Nothing

-- | `c_search::forced`: an absolute forced include placed against the
-- root; else the entry's directory, then the quoted chain's first hit
-- in path order.
forced :: Env -> Maybe String -> String -> Chain -> Maybe String
forced env dir spec ch
  | isAbsolute spec = case relativize (eBase env) "" spec of
      Just p | member w p -> Just p
      _ -> Nothing
  | otherwise = (dir >>= \d -> inScope w d spec) <|> Set.lookupMin (along w ch False spec Set.empty)
 where
  w = eWorld env

-- | `c_search::targets`: every target one include line reaches under one
-- chain — the own directory (quoted, unless `-I-`), the declared roots,
-- then the chain.
targets :: Env -> String -> String -> Chain -> Set.Set String -> Set.Set String
targets env from spec ch st
  | null name = Set.empty
  | not system && chOwnDir ch, Just p <- inScope w (parentDir from) name = Set.singleton p
  | not (Set.null decl) = decl
  | otherwise = along w ch system name st
 where
  (name, system) = form spec
  decl = declared env name
  w = eWorld env

-- | `forced_arcs`: every unit seat's forced includes, resolved — the
-- (unit, header) arcs a build declares outside the source text.
forcedArcs :: Env -> Index -> [(String, String)]
forcedArcs env ix =
  Set.toAscList . Set.fromList $
    [ (sFile s, h)
    | s <- ixSeats ix
    , sUnit s
    , let ch = ixChains ix !! sChain s
    , spec <- chForced ch
    , Just h <- [forced env (sDir s) spec ch]
    ]
