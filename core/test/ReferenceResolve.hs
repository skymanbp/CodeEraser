{-# LANGUAGE OverloadedStrings #-}

-- | An independently written reference for the resolve family (plan
-- v2.33 wave W2a; on text since W2-text; R since its stage B): the five
-- ladders written apart from the shipped modules, over the case as it
-- holds its configuration (module paths, replace pairs, roots,
-- dependency names, package names and Collate lists) — the shipped core
-- reads the same configuration from the request's go.mod and DESCRIPTION
-- texts and pyproject document, so its readers are on the path. Every
-- site of the two hundred seeded cases must get the same answer from
-- both, target spelled the same, and every case the same R package code
-- (the Java rungs have their own reference, ReferenceJava). The C ladder
-- is read without compile databases
-- here (the measuring side's ladder batteries drive the database rungs
-- end to end through the shipped core).
module ReferenceResolve (equivalence, refAnswer) where

import CE.Resolve (respond)
import CE.Resolve.Cost (Reason (..), langC, langCpp, langGo, langLua, langPy, langR)
import CE.Resolve.Tables (goStd, kindLibrary, kindLoad, kindRequire, kindSource, luaStdlib, pyStdlib)
import Data.Aeson (Value (..), decodeStrict, encode)
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseMaybe, parseJSON)
import qualified Data.ByteString.Lazy as BL
import Data.List (intercalate, isInfixOf, isPrefixOf, isSuffixOf, sortOn)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe, listToMaybe, mapMaybe)
import qualified Data.Set as Set
import ReferenceResolveGen
import WireHarness (runChecks)

-- | Every case's every site: the shipped core against the reference,
-- and the cases reach every rung and refusal the four ladders give.
equivalence :: IO Bool
equivalence =
  runChecks
    ( ("resolve: 200 cases, 2040 sites, shipped = reference", null bad)
        : ("resolve: 200 cases, R package code shipped = reference", all samePackages cases)
        : ("resolve: the cases reach every rung and refusal of the five ladders", null missing)
        : take 5 bad
          <> [("  never reached: " <> show m, False) | m <- missing]
    )
 where
  bad = [("  case " <> show k <> ": " <> show d, False) | (k, c) <- zip [1 :: Int ..] cases, Just d <- [disagree c]]
  reached = Set.fromList [(lang, refShape (refAnswer c s)) | c <- cases, s@(lang, _, _, _) <- cSites c]
  missing = filter (`Set.notMember` reached) required
  required =
    [(langPy, s) | s <- ["file 1", "file 2", "file 3", "external 4", "OutOfScope", "AmbiguousRoot"]]
      <> [(langLua, s) | s <- ["file 1", "file 2", "external 3", "OutOfScope", "Unsupported", "AmbiguousRoot"]]
      <> [(langGo, s) | s <- ["package 1", "package 2", "external 3", "OutOfScope", "AmbiguousWorkspace"]]
      <> [(langC, s) | s <- ["file 1", "file 2", "file 4", "external 5", "OutOfScope", "Empty", "AmbiguousRoot"]]
      <> [(langR, s) | s <- ["file 1", "package 2", "external 3", "OutOfScope", "Unsupported", "AmbiguousRoot", "AmbiguousWorkspace"]]

-- | The first disagreeing site of a case, if any.
disagree :: Case -> Maybe (String, String, Maybe Ref, Ref)
disagree c = listToMaybe [(from, spec, got, want) | ((_, _, from, spec), got, want) <- zip3 (cSites c) shipped wanted, got /= Just want]
 where
  shipped = maybe (repeat Nothing) (map Just) (shippedReply c >>= answers) <> repeat Nothing
  wanted = map (refAnswer c) (cSites c)

shippedReply :: Case -> Maybe Value
shippedReply c = either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode (request c))))

-- | The reply's `packages` against the reference expansion.
samePackages :: Case -> Bool
samePackages c = (got :: Maybe [(String, [String])]) == Just (refPackages c)
 where
  got = case shippedReply c of
    Just (Object o) -> parseMaybe parseJSON =<< KM.lookup "packages" o
    _ -> Nothing

-- | The reference ladders, one site.
refAnswer :: Case -> (Integer, Integer, String, String) -> Ref
refAnswer c (lang, kind, from, spec)
  | lang == langPy = py c from spec
  | lang == langLua = lua c kind from spec
  | lang == langGo = go c from spec
  | lang `elem` [langC, langCpp] = cFamily c from spec
  | lang == langR = rLang c kind from spec
  | otherwise = RUnres Unsupported

walked :: Case -> Set.Set String
walked = Set.fromList . cFiles

-- | `roots::join_rel`: the directory's non-empty pieces, then the
-- spec's pieces read — "" and "." stay, ".." climbs, above the root is
-- no path.
joinRel :: String -> String -> Maybe String
joinRel dir spec = intercalate "/" . reverse <$> foldl step (Just (reverse (filter (not . null) (splitOn '/' dir)))) (splitOn '/' spec)
 where
  step acc seg = acc >>= \st -> case seg of
    "" -> Just st
    "." -> Just st
    ".." -> if null st then Nothing else Just (drop 1 st)
    s -> Just (s : st)

parentDir :: String -> String
parentDir p = maybe "" (\i -> take i p) (lastSlash p)
 where
  lastSlash s = listToMaybe (reverse [i | (i, ch) <- zip [0 ..] s, ch == '/'])

joinDir :: String -> String -> String
joinDir d n = if null d then n else d <> "/" <> n

oneOf :: [String] -> Int -> Maybe Ref
oneOf hits rung = case Set.toList (Set.fromList hits) of
  [] -> Nothing
  [p] -> Just (RFile p rung)
  _ -> Just (RUnres AmbiguousRoot)

-- Python ---------------------------------------------------------------

py :: Case -> String -> String -> Ref
py c from spec
  | "." `isPrefixOf` spec = relative
  | Just a <- absolute moduleAt = a
  | Just a <- absolute initPrefix = rung3 a
  | otherwise = stdlibOrDeps
 where
  files = walked c
  absolute rule = (\a -> case a of RFile p _ -> RFile p 2; x -> x) <$> oneOf (mapMaybe (\r -> rule r spec) ("" : "src" : cPyRoots c)) 2
  rung3 a = case a of RFile p _ -> RFile p 3; x -> x
  relative =
    let dots = length (takeWhile (== '.') spec)
        rest = drop dots spec
        climb d = if null d then Nothing else Just (parentDir d)
     in case foldl (\d _ -> d >>= climb) (Just (parentDir from)) [2 .. dots] of
          Nothing -> RUnres OutOfScope
          Just dir -> fromMaybe (RUnres OutOfScope) (fmap (`RFile` 1) (moduleAt dir rest) `orElse` fmap (`RFile` 3) (initPrefix dir rest))
  moduleAt root dotted = do
    b <- if null dotted then Just root else joinRel root (map (\ch -> if ch == '.' then '/' else ch) dotted)
    let pkg = if null b then "__init__.py" else b <> "/__init__.py"
    listToMaybe [p | p <- [pkg, b <> ".py"], Set.member p files]
  initPrefix root dotted =
    let segs = filter (not . null) (splitOn '.' dotted)
     in listToMaybe [h | k <- reverse [1 .. length segs - 1], Just h <- [moduleAt root (intercalate "." (take k segs))], "/__init__.py" `isSuffixOf` h]
  stdlibOrDeps =
    let top = takeWhile (/= '.') spec
     in if top `elem` cPyDeps c || top == "__future__" || [top] `elem` map pure (Set.toList pyStdlib) then RExt 4 else RUnres OutOfScope

orElse :: Maybe a -> Maybe a -> Maybe a
orElse a b = maybe b Just a

-- Lua ------------------------------------------------------------------

lua :: Case -> Integer -> String -> String -> Ref
lua c kind from spec
  | kind == kindRequire && splitOn '.' spec `elem` map (splitOn '.') (Set.toList luaStdlib) = RExt 3
  | kind == kindRequire = luaModule
  | kind == kindLoad = besideOrRoot
  | otherwise = RUnres Unsupported
 where
  files = walked c
  name = map (\ch -> if ch == '.' then '/' else ch) spec
  standard = [".lua", "/init.lua"]
  add m (d, sufs) = M.insertWith (\new old -> sortOn (/= ".lua") (old <> [s | s <- new, s `notElem` old])) d (sortOn (/= ".lua") (dedup sufs)) m
  dedup = foldl (\acc s -> if s `elem` acc then acc else acc <> [s]) []
  searched =
    foldl add M.empty ([(d, standard) | d <- ["", "src", "lua"] <> cLuaRoots c] <> [(d, [s]) | (d, s) <- cLuaTemplates c] <> [(parentDir from, standard)])
  luaModule
    | any null (splitOn '/' name) = RUnres OutOfScope
    | otherwise = fromMaybe (RUnres OutOfScope) (oneOf [p | (d, sufs) <- M.toList searched, Just p <- [listToMaybe [q | s <- sufs, Just q <- [joinRel d (name <> s)], Set.member q files]]] 1)
  besideOrRoot = maybe (RUnres OutOfScope) (`RFile` 2) (scriptPath c from spec)

-- | A path a script loads: beside the loading file, then under the root.
scriptPath :: Case -> String -> String -> Maybe String
scriptPath c from spec
  | take 1 spec `elem` ["/", "~"] || ':' `elem` spec = Nothing
  | otherwise = listToMaybe [p | d <- [parentDir from, ""], Just p <- [joinRel d spec], Set.member p (walked c)]

-- R --------------------------------------------------------------------

rLang :: Case -> Integer -> String -> String -> Ref
rLang c kind from spec
  | kind == kindSource && "://" `isInfixOf` spec = RExt 3
  | kind == kindSource = fromMaybe (RUnres OutOfScope) (fmap (`RFile` 1) (scriptPath c from spec) `orElse` oneOf [p | d <- cRRoots c, Just p <- [joinRel d spec], Set.member p (walked c)] 1)
  | kind == kindLibrary = case [d | (d, Just p, _) <- cDescs c, p == spec] of
      [] -> RExt 3
      [d] -> RPkg d 2
      _ -> RUnres AmbiguousWorkspace
  | otherwise = RUnres Unsupported

-- | Each package's code: the Collate files under its `R/`, else every
-- walked `.R` / `.r` file (a stem before the dot) directly in it.
refPackages :: Case -> [(String, [String])]
refPackages c = [(d, code d cs) | (d, Just _, cs) <- cDescs c]
 where
  code d cs =
    let dir = joinDir d "R"
     in if null cs
          then [f | f <- cFiles c, parentDir f == dir, any (\e -> e `isSuffixOf` f && length (drop (length dir) f) > 3) [".R", ".r"]]
          else Set.toList (Set.fromList [p | n <- cs, Just p <- [joinRel dir n], Set.member p (walked c)])

-- Go -------------------------------------------------------------------

go :: Case -> String -> String -> Ref
go c from spec = fromMaybe (external spec) (moduleRung spec `orElse` replaceRung)
 where
  mods = cGoMods c
  strip s m
    | s == m = Just ""
    | (m <> "/") `isPrefixOf` s = Just (drop (length m + 1) s)
    | otherwise = Nothing
  moduleRung s =
    let found = [(length m, if null rest then d else joinDir d rest) | (d, m, _) <- mods, Just rest <- [strip s m]]
        best = maximum (map fst found)
     in case Set.toList (Set.fromList [d | (n, d) <- found, n == best]) of
          [] -> Nothing
          [d] -> Just (package d 1)
          _ -> Just (RUnres AmbiguousWorkspace)
  owner = listToMaybe (reverse (sortOn (\(d, _, _) -> length d) [m | m@(d, _, _) <- mods, null d || (d <> "/") `isPrefixOf` from]))
  replaceRung = do
    (dir, _, reps) <- owner
    (rest, new) <- listToMaybe (sortOn (length . fst) [(rest, new) | (old, new) <- reps, Just rest <- [strip spec old]])
    if "./" `isPrefixOf` new || "../" `isPrefixOf` new
      then (\b -> package (if null rest then b else joinDir b rest) 2) <$> joinRel dir new
      else
        let rewritten = if null rest then new else new <> "/" <> rest
         in Just (maybe (external rewritten) (rung2) (moduleRung rewritten))
  rung2 a = case a of RPkg d _ -> RPkg d 2; x -> x
  package d rung
    | any importable (cFiles c) = RPkg d rung
    | otherwise = RUnres OutOfScope
   where
    prefix = if null d then "" else d <> "/"
    importable f = prefix `isPrefixOf` f && let r = drop (length prefix) f in ".go" `isSuffixOf` r && '/' `notElem` r && not ("_test.go" `isSuffixOf` r)
  external s = if '.' `elem` takeWhile (/= '/') s || splitOn '/' s `elem` map (splitOn '/') (Set.toList goStd) then RExt 3 else RUnres OutOfScope

-- C / C++ (no compile databases) ---------------------------------------

cFamily :: Case -> String -> String -> Ref
cFamily c from spec
  | null name = RUnres Empty
  | otherwise = fromMaybe fallback (beside `orElse` declared `orElse` includeRung)
 where
  (name, system) = maybe (spec, False) (\n -> (n, True)) (angled spec)
  files = walked c
  inScope d = (\p -> if Set.member p files then Just p else Nothing) =<< joinRel d name
  beside = if system then Nothing else (`RFile` 1) <$> inScope (parentDir from)
  declared = oneOf (mapMaybe inScope (cCRoots c)) 2
  includeRung = oneOf (mapMaybe (inScope . (`joinDir` "include")) (ancestors (parentDir from))) 4
  ancestors d = d : if null d then [] else ancestors (parentDir d)
  fallback = if system then RExt 5 else RUnres OutOfScope
