-- | An independently written reference for the Java rungs (plan v2.33
-- W2-text stage C), written apart from CE.Resolve.Java and its helpers:
-- the tree held as sets — the walked headers' files by package, the
-- source set of a path read by pattern — the rungs as folds over them,
-- the inheritance walk as one fold over (seen, hits). The cases
-- (ReferenceJavaGen) only write annotations of four forms (`@T`, `@a.T`,
-- `@T(1)`, an argument string holding a parenthesis) and ASCII spaces,
-- so the reference reads those and no more; the measuring side's
-- differential gate (tests subrepo unit/dedup/ladder_diff/java.rs)
-- drives the full lexer against the frozen Rust rungs. Every site of
-- the two hundred cases must get the same answer from both, and the
-- cases reach every rung and refusal the Java rungs give.
module ReferenceJava (equivalence) where

import CE.Resolve (respond)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Tables (javaLang, javaPackages, kindImport, kindImportStar, kindTypeRef)
import Data.Aeson (decodeStrict, encode)
import qualified Data.ByteString.Lazy as BL
import Data.List (intercalate, isPrefixOf, stripPrefix)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe, isJust, listToMaybe, mapMaybe)
import qualified Data.Set as Set
import ReferenceJavaGen
import ReferenceResolveGen (Ref (..), answers, refShape, splitOn)
import WireHarness (battery, runChecks)

equivalence :: IO Bool
equivalence =
  runChecks
    ( battery
        ("resolve: 200 Java cases, 2400 sites, shipped = reference", "resolve: the Java cases reach every rung and refusal of the Java rungs")
        (map (fmap show . disagree) javaCases)
        ["file 1", "file 2", "file 3", "package 2", "external 4", "OutOfScope", "Unsupported", "AmbiguousRoot", "AmbiguousPaths", "OwnUnit"]
        (Set.fromList [refShape (refSite c s) | c <- javaCases, s <- jSites c])
    )

disagree :: JCase -> Maybe (JSite, Maybe Ref, Ref)
disagree c = listToMaybe [(s, got, want) | (s, got, want) <- zip3 (jSites c) shipped (map (refSite c) (jSites c)), got /= Just want]
 where
  reply = either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode (javaRequest c))))
  shipped = maybe [] (map Just) (reply >>= answers) <> repeat Nothing

-- | One site.
refSite :: JCase -> JSite -> Ref
refSite c (kind, from, written, line)
  | any null segs || (static && length segs < 2) = RUnres OutOfScope
  | otherwise = notOwn (fromMaybe (jdkRef segs) answer)
 where
  (static, rest) = maybe (False, written) ((,) True) (stripPrefix "static " written)
  bare = filter (/= ' ') (stripNotes rest)
  star = kind == kindImportStar
  name = fromMaybe bare (if kind == kindImport || star then completed c (from, line) (star, static) bare else Nothing)
  segs = splitOn '.' name
  answer
    | kind == kindImport = if static then fq c from (take (length segs - 1) segs) 2 2 else fq c from segs 1 2
    | star = if static then fq c from segs 2 2 else maybe (fq c from segs 2 2) Just (pkgDir c from name)
    | kind == kindTypeRef = Just (typeRef c from segs line)
    | otherwise = Just (RUnres Unsupported)
  notOwn r = case r of
    RFile p _ | p == from -> RUnres OwnUnit
    _ -> r

-- | The four annotation forms the cases write, dropped.
stripNotes :: String -> String
stripNotes s = case break (== '@') s of
  (pre, '@' : post) -> pre <> stripNotes (afterNote post)
  (pre, _) -> pre
 where
  afterNote p = case dropWhile (== ' ') (dropWhile (`elem` nameChars) (dropWhile (== ' ') p)) of
    '(' : more -> close (1 :: Int) more
    other -> other
  nameChars = ['a' .. 'z'] <> ['A' .. 'Z'] <> ['0' .. '9'] <> "_$."
  close 0 t = t
  close d t = case t of
    [] -> []
    '"' : more -> close d (drop 1 (dropWhile (/= '"') more))
    '(' : more -> close (d + 1) more
    ')' : more -> close (d - 1) more
    _ : more -> close d more

-- | The header's import on the site's line the cut spec begins.
completed :: JCase -> (String, Int) -> (Bool, Bool) -> String -> Maybe String
completed c (from, line) (star, static) bare = do
  (_, ims, _) <- M.lookup from (jHeads c)
  let onLine = [n | (n, s, st, l) <- ims, l == line, s == star, st == static]
      begun = [n | n <- onLine, reverse (dropWhile (== '.') (reverse bare)) `isPrefixOf` n]
  if bare `elem` onLine then Nothing else case begun of
    [n] -> Just n
    _ -> Nothing

-- | The walked files with a header, by package.
byPackage :: JCase -> M.Map String (Set.Set String)
byPackage c = M.fromListWith Set.union [(pkg, Set.singleton f) | (f, (pkg, _, _)) <- M.toList (jHeads c), f `elem` jFiles c]

-- | `<module>/src/<set>/java` holding the path, and the set.
layout :: String -> Maybe (String, String)
layout = go [] . splitOn '/'
 where
  go acc ("src" : set : "java" : _ : _) = Just (intercalate "/" (reverse acc <> ["src", set, "java"]), set)
  go acc (x : xs) = go (x : acc) xs
  go _ [] = Nothing

seeable :: String -> String -> Bool
seeable from f = fmap snd (layout from) /= Just "main" || maybe True ((== "main") . snd) (layout f)

-- | The package's files `from` sees that declare the class.
holding :: JCase -> String -> String -> String -> [String]
holding c from pkg cls = filter ok (Set.toList (M.findWithDefault Set.empty pkg (byPackage c)))
 where
  ok f = seeable from f && case M.lookup f (jHeads c) of
    Just (_, _, tys@(_ : _)) -> cls `elem` [n | Ty n _ _ _ _ <- tys]
    _ -> reverse (takeWhile (/= '/') (reverse f)) == cls <> ".java"

underRoots :: JCase -> [String] -> [String]
underRoots c = filter (\p -> any (\d -> null d || p == d || (d <> "/") `isPrefixOf` p) (jRoots c))

-- | One of the set, or the one the roots hold.
oneOf :: JCase -> [String] -> Maybe String
oneOf c xs = case Set.toList (Set.fromList xs) of
  [one] -> Just one
  many -> case underRoots c many of
    [one] -> Just one
    _ -> Nothing

choose :: JCase -> [String] -> Int -> Reason -> Maybe Ref
choose _ [] _ _ = Nothing
choose c hits g why = Just (maybe (RUnres why) (`RFile` g) (oneOf c hits))

-- | The longest `p.C` prefix whose package holds a file declaring `C`.
fq :: JCase -> String -> [String] -> Int -> Int -> Maybe Ref
fq c from segs whole part = listToMaybe (mapMaybe at (reverse [2 .. length segs]))
 where
  at k = choose c (holding c from (intercalate "." (take (k - 1) segs)) (segs !! (k - 1))) (if k == length segs then whole else part) AmbiguousRoot

pkgDir :: JCase -> String -> String -> Maybe Ref
pkgDir c from pkg = case Set.toList (Set.fromList [dirOf f | f <- Set.toList (M.findWithDefault Set.empty pkg (byPackage c)), seeable from f]) of
  [] -> Nothing
  dirs -> Just (maybe (RUnres AmbiguousRoot) (`RPkg` 2) (mine dirs `orElse` oneOf c dirs))
 where
  mine dirs = case layout from of
    Just (root, _) | [d] <- filter ((root <> "/") `isPrefixOf`) dirs -> Just d
    _ -> Nothing
  orElse a b = maybe b Just a

dirOf :: String -> String
dirOf f = reverse (drop 1 (dropWhile (/= '/') (reverse f)))

exported :: [String] -> Bool
exported segs = any (\k -> intercalate "." (take k segs) `Set.member` javaPackages) [1 .. length segs]

jdkRef :: [String] -> Ref
jdkRef segs = if exported segs then RExt 4 else RUnres OutOfScope

typeRef :: JCase -> String -> [String] -> Int -> Ref
typeRef c from segs line = case M.lookup from (jHeads c) of
  Nothing -> RUnres OutOfScope
  Just h@(_, ims, tys) -> case maybe (unitView c from first h) Just (inherited c from first tys line) of
    Just r -> r
    Nothing -> fromMaybe (outside ims) (fq c from segs 3 3)
 where
  first = head' segs
  outside ims
    | first `Set.member` javaLang = RExt 4
    | length segs > 1 = jdkRef segs
    | otherwise =
        let out = [n | (n, star, st, _) <- ims, star, not st, not (M.member n (byPackage c))]
         in if not (null out) && all (`Set.member` javaPackages) out then RExt 4 else RUnres OutOfScope

head' :: [String] -> String
head' = concat . take 1

-- | A unit's own view of a simple name: its single import (a type one
-- first), its package (beside it first), its star-imported packages.
unitView :: JCase -> String -> String -> (String, [(String, Bool, Bool, Int)], [Ty]) -> Maybe Ref
unitView c from name (pkg, ims, _) = case [(n, st) | st <- [False, True], (n, False, st', _) <- ims, st' == st, lastSeg n == name] of
  (n, st) : _ ->
    let ss = splitOn '.' n
     in Just (fromMaybe (jdkRef ss) (fq c from (if st then take (length ss - 1) ss else ss) 3 3))
  [] ->
    let own = holding c from pkg name
        beside = (if null (dirOf from) then "" else dirOf from <> "/") <> name <> ".java"
     in if beside `elem` own
          then Just (RFile beside 3)
          else maybe (choose c (concat [holding c from p name | (p, True, False, _) <- ims]) 3 AmbiguousPaths) Just (choose c own 3 AmbiguousRoot)
 where
  lastSeg = reverse . takeWhile (/= '.') . reverse

-- | The member type `name` the classes enclosing `line` inherit.
inherited :: JCase -> String -> String -> [Ty] -> Int -> Maybe Ref
inherited c from name tys line = listToMaybe [r | t <- around tys, let hits = snd (supers from t (Set.empty, Set.empty)), not (Set.null hits), Just r <- [choose c (Set.toList hits) 3 AmbiguousPaths]]
 where
  around ts = concat [around ms <> [t] | t@(Ty _ _ ms a b) <- ts, a <= line, line <= b]
  supers file (Ty _ ss _ _ _) acc = foldl (visit file) acc ss
  visit file (seen, hits) w = case typeAt c file w of
    Nothing -> (seen, hits)
    Just (sf, sd@(Ty sn _ sms sa _))
      | (sf, sn, sa) `Set.member` seen -> (seen, hits)
      | name `elem` [n | Ty n _ _ _ _ <- sms] -> (Set.insert (sf, sn, sa) seen, Set.insert sf hits)
      | otherwise -> supers sf sd (Set.insert (sf, sn, sa) seen, hits)

-- | The file and declaration a supertype written in `file` names.
typeAt :: JCase -> String -> String -> Maybe (String, Ty)
typeAt c file w = do
  unit@(_, _, utys) <- M.lookup file (jHeads c)
  let segs = splitOn '.' w
      hd = head' segs
  (path, at) <-
    if isJust (oneDecl utys hd)
      then Just (file, 0)
      else case unitView c file hd unit of
        Just (RFile p _) -> Just (p, 0)
        _ -> qualified segs
  (_, _, ptys) <- M.lookup path (jHeads c)
  d <- oneDecl ptys (segs !! at)
  descend d (drop (at + 1) segs) >>= \t -> Just (path, t)
 where
  qualified segs = case [(fs, k - 1) | k <- reverse [2 .. length segs], let fs = holding c file (intercalate "." (take (k - 1) segs)) (segs !! (k - 1)), not (null fs)] of
    [] -> Nothing
    (fs, at) : _ -> (\f -> (f, at)) <$> oneOf c fs
  descend t [] = Just t
  descend (Ty _ _ ms _ _) (s : rest) = listToMaybe [m | m@(Ty n _ _ _ _) <- ms, n == s] >>= \m -> descend m rest

-- | A unit's top-level type of the name, else its one member type of the
-- name at any depth.
oneDecl :: [Ty] -> String -> Maybe Ty
oneDecl tys n = case [t | t@(Ty m _ _ _ _) <- tys, m == n] of
  t : _ -> Just t
  [] -> case deep tys of
    [one] -> Just one
    _ -> Nothing
 where
  deep ts = concat [[t | m == n] <> deep ms | t@(Ty m _ ms _ _) <- ts]
