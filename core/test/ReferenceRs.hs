{-# LANGUAGE OverloadedStrings #-}

-- | An independently written reference for the Rust rungs, the Cargo.toml
-- reading and the crate-root and privacy answers (plan v2.33 W2-text
-- stage F), written apart from CE.Resolve.Rs, CE.Resolve.RsTree,
-- CE.Resolve.RsSurface and CE.Resolve.Cargo (the Cargo half is
-- ReferenceRsCargo): the cases (ReferenceRsGen)
-- are item and manifest records, and the facts the core asks for — a
-- Cargo.toml's document, a source's answers at a row, its top-level
-- surface — are written from them, the way the measuring side reads
-- them off a file. The core is asked the way the measuring side asks it:
-- a request with the walk's manifests, then every fact it names under
-- `tsWanted` answered from the case, until it names none. Every site of
-- the two hundred cases must get the same answer from both, the crate
-- roots and the kept files too, and the cases reach every answer the Rust
-- rungs give.
module ReferenceRs (equivalence) where

import CE.Resolve.Cost (Reason (..), langRs)
import CE.Resolve.Tables (kindModDecl, kindUse, rsBuiltin)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (isPrefixOf, isSuffixOf)
import qualified Data.Map.Strict as M
import Data.Maybe (isJust, listToMaybe)
import qualified Data.Set as Set
import ReferenceResolveGen (Ref (..), answers, originsOf, refShape, replyKey, requestHeader, settleWith)
import ReferenceRsCargo
import ReferenceRsGen
import ReferenceTs (joinRel, parent, within)
import WireHarness (battery, runChecks)

equivalence :: IO Bool
equivalence = runChecks (battery (agreeTitle, reachTitle) (map disagree rsCases) wanted (Set.fromList (concatMap shapesOf rsCases)))
 where
  agreeTitle = "resolve: 200 Rust cases, " <> show (length (concatMap sitesOf rsCases)) <> " sites, shipped = reference (rungs, crate roots, kept files)"
  reachTitle = "resolve: the Rust cases reach every answer of the Rust rungs"
  shapesOf c = map (refShape . refSite c) (sitesOf c)
  wanted = map ("file " <>) ["1", "2", "3", "4"] <> map ("via " <>) ["2", "3", "4"] <> ["external 4", "OutOfScope", "AmbiguousPaths", "AmbiguousRoot", "AmbiguousWorkspace"]

-- | The first disagreement of a case, spelled: the core is asked round
-- after round, each round's wanted facts answered from the case.
disagree :: RCase -> Maybe String
disagree c = maybe (Just "no reply, or facts still wanted after twelve rounds") firstOff (settleWith 12 (request c) (fact c))
 where
  firstOff v =
    listToMaybe
      [ name <> ": core " <> got <> " reference " <> expected
      | (name, same, got, expected) <-
          [ ("sites", answers v == Just want, show (answers v), show want)
          , ("crates", strs "crates" v == Just (Set.toList (crates c)), show (strs "crates" v), show (crates c))
          , ("private", strs "private" v == Just (Set.toList (kept c)), show (strs "private" v), show (kept c))
          ]
      , not same
      ]
  want = map (refSite c) (sitesOf c)
  strs :: String -> Value -> Maybe [String]
  strs = replyKey

-- | Every site: each `mod x;` and each `use` (its first line), at its
-- 1-based line.
sitesOf :: RCase -> [(Integer, String, Int, String)]
sitesOf c = [s | (p, its) <- rSources c, rw <- rowsOf its, s <- site p rw]
 where
  site p rw = case rwItem rw of
    IMod name _ _ -> [(kindModDecl, p, rwRow rw + 1, name)]
    IUse _ first _ -> [(kindUse, p, rwRow rw + 1, first)]
    _ -> []

request :: RCase -> [Value] -> Value
request c known = object (requestHeader (rFiles c) origins <> body)
 where
  ss = sitesOf c
  (origins, index) = originsOf (rFiles c) [p | (_, p, _, _) <- ss]
  row (k, p, line, spec) = toJSON (langRs, k, index M.! p, spec, line)
  walked = Set.toList (Set.fromList (map manPath (filter mWalked (rMans c))))
  body =
    [ "sites" .= map row ss
    , "rs" .= object ["packages" .= walked, "crateRoots" .= rDeclared c, "manifests" .= manifests c, "owners" .= owners c]
    , "ts" .= object ["facts" .= known]
    ]

-- the rungs

refSite :: RCase -> (Integer, String, Int, String) -> Ref
refSite c (k, from, line, spec)
  | k == kindModDecl = modRef c roots from (line - 1) spec
  | otherwise = case whole >>= usePath of
      Nothing -> RUnres OutOfScope
      Just (global, segs) -> let (o, used, walk) = walkUse c ctx (from, line - 1) segs global in bind c walk o used
 where
  ctx@(_, roots) = ctxAt c (parent from)
  whole
    | isJust (usePath spec) = Just spec
    | otherwise = listToMaybe [w | rw <- rowsAt c from, rwRow rw == line - 1, IUse _ f w <- [rwItem rw], f == spec]

rowsAt :: RCase -> String -> [Row]
rowsAt c p = maybe [] rowsOf (lookup p (rSources c))

coverAt :: RCase -> String -> Int -> [Block]
coverAt c p row = maybe [] (`covering` row) (lookup p (rSources c))

-- | R1: a `#[path]` value answers outright (against the file's own
-- directory at file level), else one child lookup under the declarer's
-- child directory, one directory per enclosing bodied module.
modRef :: RCase -> Set.Set String -> String -> Int -> String -> Ref
modRef c roots from row name = case [attr | rw <- rowsAt c from, rwRow rw == row, IMod n _ attr <- [rwItem rw], n == name] of
  (Just t : _) -> case joinRel (if null inl then parent from else conv) t of
    Just p | p `elem` rFiles c -> RFile p 1
    _ -> RUnres OutOfScope
  _ -> either RUnres (maybe (RUnres OutOfScope) (`RFile` 1)) (childAt c conv name)
 where
  inl = [n | (n, _, _) <- coverAt c from row]
  conv = foldl within (childDirOf roots from) inl

childAt :: RCase -> String -> String -> Either Reason (Maybe String)
childAt c dir name = case (plain `elem` rFiles c, modrs `elem` rFiles c) of
  (True, True) -> Left AmbiguousPaths
  (True, _) -> Right (Just plain)
  (_, True) -> Right (Just modrs)
  _ -> Right Nothing
 where
  plain = within dir (name <> ".rs")
  modrs = within dir (name <> "/mod.rs")

isModRs :: String -> Bool
isModRs f = f == "mod.rs" || "/mod.rs" `isSuffixOf` f

childDirOf :: Set.Set String -> String -> String
childDirOf roots f
  | Set.member f roots || isModRs f = parent f
  | otherwise = within (parent f) (stem (reverse (takeWhile (/= '/') (reverse f))))
 where
  stem s = maybe s stem (stripSuffix' ".rs" s)

-- | One anchor's descent to the deepest walked module, with the segments
-- consumed.
down :: RCase -> Set.Set String -> String -> [String] -> Either Reason (String, Int)
down c roots f segs = case segs of
  [] -> Right (f, 0)
  (s : rest) -> childAt c (childDirOf roots f) s >>= maybe (Right (f, 0)) (\next -> fmap (+ 1) <$> down c roots next rest)

-- | Every anchor descends; the answers fold into one.
walkFrom :: RCase -> Set.Set String -> [String] -> [String] -> Int -> (Ref, Int)
walkFrom c roots anchors segs rung = case traverse (\a -> down c roots a segs) anchors of
  Right hits -> fold hits rung
  Left why -> (RUnres why, 0)

fold :: [(String, Int)] -> Int -> (Ref, Int)
fold hits rung = case Set.toList (Set.fromList (map fst hits)) of
  [] -> (RUnres OutOfScope, 0)
  [p] -> (RFile p rung, minimum (map snd hits))
  _ -> (RUnres AmbiguousRoot, 0)

-- | The bind-free walk: the answer, the segments consumed, the segments
-- walked.
walkUse :: RCase -> (Maybe Pkg, Set.Set String) -> (String, Int) -> [String] -> Bool -> (Ref, Int, [String])
walkUse c (pkg, roots) (from, row) segs global = case segs of
  [] -> (RUnres OutOfScope, 0, segs)
  (h : rest)
    | global -> with rest (externRef c pkg roots h rest)
    | h == "crate" -> with rest (crateRef c roots (coveringOf from roots) rest)
    | h == "self" -> if null cover then with rest (walkFrom c roots [from] rest 3) else (RFile from 3, length rest, rest)
    | h == "super" ->
        let ups = 1 + length (takeWhile (== "super") rest)
            tl = drop (ups - 1) rest
            anchors = if ups <= length cover then [from] else climbUp c roots from (ups - length cover)
         in with tl (walkFrom c roots anchors tl 3)
    | otherwise -> with rest (inNamespace c (pkg, roots) (from, cover) h rest)
 where
  cover = coverAt c from row
  with r (o, u) = (o, u, r)

-- | A first segment that is neither a keyword nor global: a module the
-- namespace declares (inline: the file itself), else an extern crate.
inNamespace :: RCase -> (Maybe Pkg, Set.Set String) -> (String, [Block]) -> String -> [String] -> (Ref, Int)
inNamespace c (pkg, roots) (from, cover) h rest = case [(b, rwRow rw) | rw <- rowsAt c from, blocksOf rw == cover, Just (n, b) <- [modName (rwItem rw)], n == h] of
  ((True, _) : _) -> (RFile from 3, length rest)
  ((False, r) : _) -> case modRef c roots from r h of
    RFile p _ -> either (\why -> (RUnres why, 0)) (\(q, u) -> (RFile q 3, u)) (down c roots p rest)
    other -> (other, 0)
  [] -> externRef c pkg roots h rest

-- | R2: two covering roots settle on the one whose top level owns the
-- next segment.
crateRef :: RCase -> Set.Set String -> [String] -> [String] -> (Ref, Int)
crateRef c roots anchors segs = case mapM (\a -> down c roots a segs) anchors of
  Left why -> (RUnres why, 0)
  Right hits
    | Set.size (Set.fromList (map fst hits)) < 2 -> fold hits 2
    | otherwise -> case [h | h@(p, u) <- hits, u < length segs, ownsName c p (segs !! u)] of
        [] -> (RUnres AmbiguousRoot, 0)
        owned -> fold owned 2

-- | Itself when it is a root, else the roots in the deepest directory
-- that holds the file.
coveringOf :: String -> Set.Set String -> [String]
coveringOf from roots
  | Set.member from roots = [from]
  | otherwise = [r | (n, r) <- holding, n == maximum (map fst holding)]
 where
  holding = [(length d, r) | r <- Set.toList roots, let d = parent r, null d || (d <> "/") `isPrefixOf` from]

-- | k×super over the files owning each parent directory; a crate root
-- drops out.
climbUp :: RCase -> Set.Set String -> String -> Int -> [String]
climbUp c roots from = Set.toList . go (Set.singleton from)
 where
  go cur k
    | k == 0 = cur
    | Set.null next = next
    | otherwise = go next (k - 1)
   where
    next = Set.unions [ownersOf (up f) | f <- Set.toList cur, Set.notMember f roots]
  up f = if isModRs f then parent (parent f) else parent f
  ownersOf d = Set.fromList ([within d "mod.rs" | within d "mod.rs" `elem` rFiles c] <> [d <> ".rs" | not (null d), (d <> ".rs") `elem` rFiles c] <> [r | r <- Set.toList roots, parent r == d])

-- | R4: a toolchain crate is external; a walked package by its
-- normalized name anchors at its lib root; a declared dependency is
-- external.
externRef :: RCase -> Maybe Pkg -> Set.Set String -> String -> [String] -> (Ref, Int)
externRef c pkg roots name rest
  | Set.member name rsBuiltin = (RExt 4, 0)
  | [p] <- named = maybe (RUnres OutOfScope, 0) fromRoot (libRootOf c p)
  | not (null named) = (RUnres AmbiguousWorkspace, 0)
  | name `elem` maybe [] (map norm . pDeps) pkg = (RExt 4, 0)
  | otherwise = (RUnres OutOfScope, 0)
 where
  named = [p | m <- rMans c, mWalked m, Just p <- [pkgOf m], Just name == fmap norm (pName p)]
  fromRoot root = case down c (Set.insert root roots) root rest of
    Right (q, u) -> (RFile q 4, u)
    Left why -> (RUnres why, 0)
  norm = map (\ch -> if ch == '-' then '_' else ch)

-- R5: the re-export surface

topOf :: RCase -> String -> [Item]
topOf c p = maybe [] id (lookup p (rSources c))

pubEntries :: RCase -> String -> [(String, [String], Int)]
pubEntries c f = [(n, s, r) | (n, s, r, True) <- entriesOf (topOf c f)]

ownsName :: RCase -> String -> String -> Bool
ownsName c f n = any ((== n) . fst) (defsOf (topOf c f)) || any (\(m, _, _, _) -> m == n) (entriesOf (topOf c f))

exportsName :: RCase -> String -> String -> Bool
exportsName c f n = (n, True) `elem` defsOf (topOf c f) || any (\(m, _, _) -> m == n) (pubEntries c f)

-- | A walk that left segments unconsumed hops once through the
-- terminal's surface to the definition file.
bind :: RCase -> [String] -> Ref -> Int -> Ref
bind c walk out used = case (out, drop used walk) of
  (RFile path rung, name : tl) -> case target path name tl of
    Just found | found /= path -> RVia found rung
    _ -> out
  _ -> out
 where
  target path name tl
    | any ((== name) . fst) (defsOf (topOf c path)) = Nothing
    | otherwise = case [(s, r) | (n, s, r) <- pubEntries c path, n == name] of
        [(s, r)] -> hopTo c path s tl r
        (_ : _ : _) -> Nothing
        [] -> case [(s, r) | (n, s, r) <- pubEntries c path, n == "*", carries path name s r] of
          [(s, r)] -> hopTo c path s (name : tl) r
          _ -> Nothing
  carries path name s r = case hopTo c path s [] r of
    Just m | m /= path -> exportsName c m name
    _ -> False

hopTo :: RCase -> String -> [String] -> [String] -> Int -> Maybe String
hopTo c facade segs tl row = case walkUse c (ctxAt c (parent facade)) (facade, row) (drop (fromEnum global) segs <> tl) global of
  (RFile p _, _, _) -> Just p
  _ -> Nothing
 where
  global = take 1 segs == [""]

-- | The module-path prefix of a spec (before its first `{`, `*` or
-- ` as `) and whether it is global; none for a hand fold's fragment.
usePath :: String -> Maybe (Bool, [String])
usePath spec
  | null after && "::" `isSuffixOf` pre = Nothing
  | otherwise = Just (take 2 pre == "::", [t | t <- map trim (splitOnStr "::" (dropTrailing pre)), not (null t)])
 where
  (before, after) = cutAt spec
  cutAt s = case s of
    (ch : r) | not (any (`isPrefixOf` s) ["{", "*", " as "]) -> let (a, b) = cutAt r in (ch : a, b)
    _ -> ("", s)
  pre = reverse (dropWhile (== ' ') (reverse before))
  dropTrailing s = maybe s dropTrailing (stripSuffix' "::" s)
