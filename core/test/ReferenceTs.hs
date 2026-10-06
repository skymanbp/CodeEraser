{-# LANGUAGE OverloadedStrings #-}

-- | An independently written reference for the TypeScript / TSX rungs and
-- the tsconfig chain (plan v2.33 W2-text stage E), written apart from
-- CE.Resolve.Ts, CE.Resolve.TsConfig, CE.Resolve.TsFacts and
-- CE.Resolve.Json: the cases (ReferenceTsGen) are read back from their
-- records, never from the text the core parses, so the JSONC reader is
-- on the core's side only. The core is asked the way the measuring side
-- asks it: a request without facts, then every file-system fact it names
-- under `tsWanted` answered from the case's world, until it names none.
-- Every site of the two hundred cases must get the same answer from both,
-- every chained tsconfig the same reached list, and the cases reach every
-- answer the TS rungs give.
module ReferenceTs (equivalence, parent, joinRel, ancestorsOf, within) where

import CE.Resolve.Cost (Reason (..), langTs, langTsx)
import CE.Resolve.Tables (nodeBuiltins, nodePrefixOnly)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseJSON, parseMaybe)
import Data.List (isPrefixOf, isSuffixOf, sortOn, stripPrefix)
import qualified Data.Map.Strict as M
import Data.Maybe (isJust, listToMaybe, mapMaybe)
import qualified Data.Set as Set
import ReferenceResolveGen (Ref (..), answers, askCore, originsOf, refShape, requestHeader, splitOn)
import ReferenceTsGen
import WireHarness (battery, runChecks)

equivalence :: IO Bool
equivalence = runChecks (battery titles (map disagree tsCases) shapes reachedShapes)
 where
  titles = ("resolve: 200 TS cases, 2400 sites, shipped = reference (rungs, reached chains)", "resolve: the TS cases reach every answer of the TS rungs")
  shapes = ["file 1", "file 2", "file 3", "file 4", "external 5", "OutOfScope", "ConfigDepth", "AmbiguousPaths", "AmbiguousWorkspace", "AmbiguousExports"]
  reachedShapes = Set.fromList [refShape (refSite c s) | c <- tsCases, s <- tSites c]

-- | The first disagreement of a case, spelled.
disagree :: TCase -> Maybe String
disagree c = case settle c [] (8 :: Int) of
  Nothing -> Just "no reply, or facts still wanted after eight rounds"
  Just v
    | answers v /= Just want -> Just ("sites: core " <> show (answers v) <> " reference " <> show want)
    | reachedOf v /= Just [(s, reached c s) | s <- chained c] -> Just ("reached: core " <> show (reachedOf v))
    | otherwise -> Nothing
 where
  want = map (refSite c) (tSites c)
  reachedOf :: Value -> Maybe [(String, [String])]
  reachedOf v = case v of
    Object o -> KM.lookup "tsReached" o >>= parseMaybe parseJSON
    _ -> Nothing

-- | Ask until the core wants nothing more, each round answering what it
-- named.
settle :: TCase -> [Value] -> Int -> Maybe Value
settle c known n = do
  v <- askCore (request c known)
  wanted <- case v of
    Object o -> KM.lookup "tsWanted" o >>= parseMaybe parseJSON
    _ -> Nothing
  if null (wanted :: [(Int, String, String)])
    then Just v
    else if n == 0 then Nothing else settle c (known <> map (fact c) wanted) (n - 1)

-- | One wanted fact answered from the case's world.
fact :: TCase -> (Int, String, String) -> Value
fact c (op, a, b) = toJSON $ case op of
  0 -> case textOf c a of
    Right (Just t) -> [toJSON op, toJSON a, toJSON b, toJSON (2 :: Int), toJSON t]
    Right Nothing -> [toJSON op, toJSON a, toJSON b, toJSON (1 :: Int), Null]
    Left () -> [toJSON op, toJSON a, toJSON b, toJSON (0 :: Int), Null]
  1 -> [toJSON op, toJSON a, toJSON b, flag (isFileIn c a), Null]
  _ -> [toJSON op, toJSON a, toJSON b, flag (isDirIn c a b), Null]
 where
  flag x = toJSON (if x then 1 else 0 :: Int)

-- | The tsconfigs whose reached lists the request asks for.
chained :: TCase -> [String]
chained c = Set.toList (Set.fromList (map cPath (tCfgs c) <> tUnreadable c))

request :: TCase -> [Value] -> Value
request c known =
  object $
    requestHeader (tFiles c) origins
      <> [ "sites" .= [[toJSON (lang from), toJSON (0 :: Int), toJSON (index M.! from), toJSON spec] | (from, spec) <- tSites c]
         , "ts" .= object ["packages" .= Set.toList (Set.fromList [pPath p | p <- tPkgs c, pWalked p]), "facts" .= known, "chains" .= chained c]
         ]
 where
  (origins, index) = originsOf (tFiles c) (map fst (tSites c))
  lang f = if ".tsx" `isSuffixOf` f then langTsx else langTs

-- path steps, written here

parent :: String -> String
parent p = maybe "" (\i -> take i p) (listToMaybe (reverse [i | (i, ch) <- zip [0 ..] p, ch == '/']))

-- | A spec joined onto a directory, `.` and `..` read; Nothing past the
-- root.
joinRel :: String -> String -> Maybe String
joinRel d s = fmap (intercalateSlash . reverse) (foldl step (Just (reverse (filter (not . null) (splitOn '/' d)))) (splitOn '/' s))
 where
  step acc seg = acc >>= \ps -> case seg of
    "" -> Just ps
    "." -> Just ps
    ".." -> case ps of
      (_ : rest) -> Just rest
      [] -> Nothing
    _ -> Just (seg : ps)
  intercalateSlash = foldr1Safe
  foldr1Safe [] = ""
  foldr1Safe xs = foldr1 (\x y -> x <> "/" <> y) xs

ancestorsOf :: String -> [String]
ancestorsOf d = d : if null d then [] else ancestorsOf (parent d)

within :: String -> String -> String
within d n = if null d then n else d <> "/" <> n

-- the chain

data Opts = Opts {oBase :: Maybe String, oPaths :: [(String, [String])], oAnchor :: String}

-- | The extends walk from one config: what it visited (the config first,
-- then each target's chain, the last entry first) and whether it held.
walk :: TCase -> [String] -> String -> ([(String, Cfg)], Bool)
walk c branch cur
  | cur `elem` branch = ([], False)
  | otherwise = case lookupCfg c cur of
      Nothing -> ([], False)
      Just x -> let (rest, ok) = go (reverse (entries (cExtends x))) in ((cur, x) : rest, ok)
 where
  entries e = case e of
    Nothing -> []
    Just (One t) -> [t]
    Just (Many ts) -> ts
  go [] = ([], True)
  go (t : ts) = case t of
    Left _ -> ([], False)
    Right s
      | take 1 s /= "." -> go ts
      | otherwise -> case joinRel (parent cur) (s <> (if ".json" `isSuffixOf` s then "" else ".json")) of
          Nothing -> ([], False)
          Just next -> case walk c (cur : branch) next of
            (v, True) -> let (v2, ok) = go ts in (v <> v2, ok)
            (v, False) -> (v, False)

lookupCfg :: TCase -> String -> Maybe Cfg
lookupCfg c p = listToMaybe [x | x <- tCfgs c, cPath x == p]

reached :: TCase -> String -> [String]
reached c s = map fst (fst (walk c [] s))

-- | The nearest tsconfig's options: Nothing none, Just Nothing refused.
optionsAt :: TCase -> String -> Maybe (Maybe Opts)
optionsAt c d = case [p | a <- ancestorsOf d, let p = within a "tsconfig.json", isJust (lookupCfg c p) || p `elem` tUnreadable c] of
  [] -> Nothing
  (start : _) -> Just $ case walk c [] start of
    (_, False) -> Nothing
    (visited, True) -> Just (fold visited)
 where
  fold visited =
    let base = listToMaybe (mapMaybe (\(p, x) -> cBase x >>= joinRel (parent p)) visited)
        paths = listToMaybe [(parent p, [(pat, maybe [] (mapMaybe (either (const Nothing) Just)) ts) | (pat, ts) <- cPaths x]) | (p, x) <- visited, not (null (cPaths x))]
     in Opts base (maybe [] snd paths) (maybe (maybe "" fst paths) id (base <* paths))

-- the rungs

firstHit :: TCase -> String -> Maybe String
firstHit c b = listToMaybe [p | p <- b : [b <> "." <> e | e <- exts] <> [b <> "/index." <> e | e <- exts], p `elem` tFiles c]
 where
  exts = ["ts", "tsx", "d.ts", "mts", "cts"]

hitsOf :: TCase -> [(String, String)] -> [String]
hitsOf c cands = Set.toList (Set.fromList (mapMaybe (\(a, t) -> joinRel a t >>= firstHit c) cands))

refSite :: TCase -> (String, String) -> Ref
refSite c (from, spec)
  | "./" `isPrefixOf` spec || "../" `isPrefixOf` spec = relativeRef
  | otherwise = case optionsAt c dir of
      Just Nothing -> RUnres ConfigDepth
      Just (Just o) | Just a <- viaPaths o -> a
      _ -> afterPaths
 where
  dir = parent from
  relativeRef = case (joinRel dir spec >>= firstHit c, esm) of
    (Just p, _) -> RFile p 1
    (_, Just p) -> RFile p 2
    _ -> RUnres OutOfScope
  esm = listToMaybe [t | (js, ts) <- [(".js", ".ts"), (".mjs", ".mts"), (".cjs", ".cts")], js `isSuffixOf` spec, Just b <- [joinRel dir (take (length spec - length js) spec)], let t = b <> ts, t `elem` tFiles c, maybe False (not . isFileIn c) (joinRel dir spec)]
  viaPaths o = case hitsOf c [(oAnchor o, replaceFirst t cap) | (pat, ts) <- oPaths o, Just cap <- [capture pat], t <- ts] of
    [p] -> Just (RFile p 3)
    (_ : _ : _) -> Just (RUnres AmbiguousPaths)
    [] -> (\p -> RFile p 3) <$> (oBase o >>= \b -> joinRel b spec >>= firstHit c)
  capture pat = case break (== '*') pat of
    (pre, '*' : post) -> stripPrefix pre spec >>= \rest -> reverse <$> stripPrefix (reverse post) (reverse rest)
    _ -> if pat == spec then Just "" else Nothing
  replaceFirst t cap = case break (== '*') t of
    (a, '*' : b) -> a <> cap <> b
    _ -> t
  afterPaths
    | builtin = RExt 5
    | "node:" `isPrefixOf` spec = RUnres OutOfScope
    | otherwise = case [p | p <- tPkgs c, pWalked p, pName p == Just name, pPath p `notElem` tUnreadable c] of
        [] -> if declared || vendored then RExt 5 else RUnres OutOfScope
        [m] -> member m
        _ -> RUnres AmbiguousWorkspace
  builtin = maybe (Set.member spec nodeBuiltins) (\n -> Set.member n nodeBuiltins || Set.member n nodePrefixOnly) (stripPrefix "node:" spec)
  (name, sub) = case splitOn '/' spec of
    (s : n : rest) | "@" `isPrefixOf` s -> (s <> "/" <> n, slashed rest)
    (n : rest) -> (n, slashed rest)
    [] -> (spec, "")
  slashed = foldr (\x y -> if null y then x else x <> "/" <> y) ""
  declared = case [p | a <- ancestorsOf dir, let p = within a "package.json", isFileIn c p] of
    (p : _) -> maybe False ((name `elem`) . pDeps) (listToMaybe [x | x <- tPkgs c, pPath x == p, p `notElem` tUnreadable c])
    [] -> False
  vendored = any (\a -> isDirIn c a name) (ancestorsOf dir)
  member m = case hitsOf c [(parent (pPath m), leaf) | leaf <- maybe [] (entryLeaves sub) (pExports m)] of
    [p] -> RFile p 4
    [] -> RUnres OutOfScope
    _ -> RUnres AmbiguousExports

-- | The string leaves of a subpath's exports entry, a pattern's capture
-- put in place of every `*`.
entryLeaves :: String -> Exp -> [String]
entryLeaves sub e = case e of
  EObj ms | any (("." `isPrefixOf`) . fst) ms -> case M.lookup key ms' of
    Just x -> leaves "" x
    Nothing -> case sortOn (\(rank, _, _) -> rank) [((length pre, length pat), x, cap) | (pat, x) <- M.toList ms', (pre, '*' : post) <- [break (== '*') pat], '*' `notElem` post, Just rest <- [stripPrefix pre key], Just cap <- [reverse <$> stripPrefix (reverse post) (reverse rest)], not (null cap)] of
      [] -> []
      ranked -> let (_, x, cap) = last ranked in leaves cap x
   where
    ms' = M.fromList ms
  other -> if key == "." then leaves "" other else []
 where
  key = if null sub then "." else "./" <> sub
  leaves cap x = case x of
    EStr s -> [concatMap (\ch -> if ch == '*' then cap else [ch]) s]
    EObj ms -> concatMap (leaves cap . snd) ms
    _ -> []
