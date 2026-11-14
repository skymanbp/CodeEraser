-- | The resolve.request boundary contract (plan v2.33 wave W2a; on text
-- since W2-text): what a well-formed request IS — the walked paths in
-- strictly ascending (path) order, the origins too and none of them
-- walked, every site in a language this family resolves with its file in
-- range and — a Java site — its line, the Java headers in strictly
-- ascending (path) order, every database probe one of the three, every
-- include list a walked file's, the cabals in strictly ascending (path)
-- order and every owner row a walked file's in that order, owned by a
-- carried cabal, the package.json paths and the chained tsconfig paths
-- in strictly ascending order, every fact a question of its kind asked
-- once with an answer in its range (a text with a text answer only, a
-- TOML document an object, a Rust row's or surface's answer one that
-- reads), the Rust package, crate-root and manifest paths in strictly
-- ascending order, every Rust owner row a walked file's in that order and
-- every Rust site with its line, the assets and the Markdown anchor sets
-- and reference tables in strictly ascending (path) order, every anchor
-- set a walked Markdown file's and every Markdown reference site's file
-- with its table, the HTML documents in strictly ascending (path) order,
-- each an HTML file's of the request, and every HTML site's file with its
-- document, the `py.names` rows in strictly ascending site order, each a
-- Python `import_from` site's with no empty name (proto 9.1.0) — and the
-- cap the request's text counts against. The
-- first offender is refused by name ("<table> <i>: <why>"); a request
-- that passes is the one CE.Resolve.World indexes without a check of its
-- own.
module CE.Resolve.Contract (offence, overCap, requestSize) where

import CE.Resolve.Cost
import CE.Resolve.JavaHeader
import CE.Resolve.Request
import CE.Resolve.RsSurface (readsAsAt, readsAsSurface)
import CE.Resolve.Tables (kindImportFrom, kindRefDef, kindRefLink)
import CE.Resolve.World (ofLangs)
import CE.Wire (ascendingOn, label)
import Data.Array (listArray, (!))
import Data.Aeson (Value (..), encode)
import qualified Data.ByteString.Lazy as BL
import Control.Applicative ((<|>))
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import Data.Char (isDigit)
import Data.List (isSuffixOf)
import Data.Maybe (isJust, listToMaybe)
import qualified Data.Aeson.Key as K

-- | One per row plus every character of every string, the decoded
-- documents at their encoded length.
requestSize :: ResolveReq -> Integer
requestSize rq =
  sum
    [ texts (rqFiles rq)
    , texts (rqOrigins rq)
    , sum [1 + chars (sSpec s) | s <- rqSites rq]
    , sum [1 + texts ns | (_, ns) <- rqPyNames rq]
    , sum [2 + chars p + texts ns | (p, ns, _) <- rqPyAll rq]
    , sum [1 + texts ds | ds <- M.elems (rqSearch rq)]
    , maybe 0 doc (rqPyproject rq)
    , sum [1 + chars d + chars s | (d, s) <- rqLuaTemplates rq]
    , sum [1 + chars p + chars t | (p, t) <- rqGoMods rq <> rqDescriptions rq]
    , chars (cRoot c)
    , sum [1 + chars d + chars r | Db d _ r <- cDbs c]
    , sum [1 + chars r + maybe 0 (sum . map doc) rows | (r, rows) <- cJson c]
    , sum [1 + chars r + maybe 0 chars t | (r, t) <- cFlags c <> cResponses c]
    , sum [1 + chars p + texts is | (p, is) <- cIncludes c]
    , sum [1 + chars p + header h | (p, h) <- rqJavaHeaders rq]
    , sum [1 + chars p + chars t | (p, t) <- rqHsCabals rq <> rqHsOwners rq]
    , texts (rqTsPackages rq <> rqTsChains rq)
    , sum [1 + chars a + chars b + maybe 0 payload t | (_, a, b, _, t) <- rqTsFacts rq]
    , texts (rqRsPackages rq <> rqRsCrateRoots rq <> rqRsManifests rq)
    , sum [1 + chars f + chars m | (f, m) <- rqRsOwners rq]
    , mdSize rq
    ]
 where
  c = rqC rq
  chars = toInteger . length
  texts = sum . map ((+ 1) . chars)
  doc :: Value -> Integer
  doc = toInteger . BL.length . encode
  -- a text at its characters, a document at its encoded length
  payload v = case v of
    String s -> chars (K.toString (K.fromText s))
    _ -> doc v
  header h = chars (hPackage h) + sum [1 + chars (iName i) | i <- hImports h] + sum (map typeSize (hTypes h))
  typeSize t = 1 + chars (tName t) + texts (tSupers t) + sum (map typeSize (tMembers t))

-- | The assets, the Markdown facts and the HTML documents: one per row,
-- one per character.
mdSize :: ResolveReq -> Integer
mdSize rq = strings (rqAssets rq) + sum (map slugs (rqMdSlugs rq)) + sum (map refs (rqMdRefs rq)) + sum (map page (rqHtmlDocs rq))
 where
  count = toInteger . length
  strings = foldr (\x n -> n + 1 + count x) 0
  slugs (p, ss) = 1 + count p + strings ss
  refs (p, ds, us) = 1 + count p + strings us + sum [1 + count l + count t | (l, t) <- ds]
  page (p, l, b, c, o, alts, ids) = 1 + count p + strings [x | Just x <- [l, b, c, o]] + strings ids + sum [1 + count h + count t | (h, t) <- alts]

overCap :: ResolveReq -> Bool
overCap rq = requestSize rq > resolveCap

-- | The first offender, table by table.
offence :: ResolveReq -> Maybe String
offence rq =
  asum
    [ ascendingOn "file" id (rqFiles rq)
    , ascendingOn "origin" id (rqOrigins rq)
    , asum [Just ("origin " <> show i <> ": a walked file") | (i, o) <- zip [0 :: Int ..] (rqOrigins rq), Set.member o walked]
    , asum (zipWith site [0 :: Int ..] (rqSites rq)) <|> pyNamesOffence rq <|> pyAllOffence rq walked
    , ascendingOn "java.header" id (map fst (rqJavaHeaders rq))
    , asum [Just ("c.db " <> show i <> ": probe out of range") | (i, Db _ p _) <- zip [0 :: Int ..] (cDbs (rqC rq)), p < 0 || p > 2]
    , asum [Just ("c.include " <> show i <> ": not a walked file") | (i, (p, _)) <- zip [0 :: Int ..] (cIncludes (rqC rq)), not (Set.member p walked)]
    , ascendingOn "hs.cabal" id (map fst (rqHsCabals rq))
    , ascendingOn "hs.owner" id (map fst (rqHsOwners rq))
    , asum [Just ("hs.owner " <> show i <> ": its file is not walked or its cabal not carried") | (i, (f, c)) <- zip [0 :: Int ..] (rqHsOwners rq), not (Set.member f walked && Set.member c cabals)]
    , ascendingOn "ts.package" id (rqTsPackages rq)
    , ascendingOn "ts.chain" id (rqTsChains rq)
    , asum (zipWith tsFact [0 :: Int ..] (rqTsFacts rq))
    , asum [Just ("ts.fact " <> show i <> ": asked twice") | (i, (seen, k)) <- zip [0 :: Int ..] (zip (scanl (flip Set.insert) Set.empty keys) keys), Set.member k seen]
    , ascendingOn "rs.package" id (rqRsPackages rq)
    , ascendingOn "rs.crateRoot" id (rqRsCrateRoots rq)
    , ascendingOn "rs.manifest" id (rqRsManifests rq)
    , ascendingOn "rs.owner" id (map fst (rqRsOwners rq))
    , asum [Just ("rs.owner " <> show i <> ": its file is not walked") | (i, (f, _)) <- zip [0 :: Int ..] (rqRsOwners rq), not (Set.member f walked)]
    , mdOffence rq walked
    ]
 where
  walked = Set.fromList (rqFiles rq)
  cabals = Set.fromList (map fst (rqHsCabals rq))
  keys = [(op, a, b) | (op, a, b, _, _) <- rqTsFacts rq]
  tsFact i (op, _, b, st, t)
    | op < 0 || op > 5 || not (question op b) = Just ("ts.fact " <> show i <> ": no such question")
    | st `notElem` states op || (st == 2) /= isJust t || not (maybe True (readsAs op) t) = Just ("ts.fact " <> show i <> ": answer out of range")
    | otherwise = Nothing
  -- the second argument: a directory fact's name, a row fact's row
  question op b = case op of
    2 -> True
    4 -> not (null b) && all isDigit b && length b <= 9
    _ -> b == ""
  states op = if op `elem` [1, 2] then [0, 1] else if op >= 4 then [0, 2] else [0, 1, 2]
  readsAs op v = case (op, v) of
    (0, String _) -> True
    (3, Object _) -> True
    (4, _) -> readsAsAt v
    (5, _) -> readsAsSurface v
    _ -> False
  paths = toInteger (length (rqFiles rq) + length (rqOrigins rq))
  site i s
    | sLang s `notElem` resolvedLangs = Just ("site " <> show i <> ": language this family does not resolve")
    | sKind s < 0 || toInteger (sFrom s) < 0 || toInteger (sFrom s) >= paths = Just ("site " <> show i <> ": kind or file out of range")
    | sLang s == langJava && sLine s == Nothing = Just ("site " <> show i <> ": a Java site without its line")
    | sLang s == langRs && sLine s == Nothing = Just ("site " <> show i <> ": a Rust site without its line")
    | otherwise = Nothing

-- | `py.all` (lane v235_graphtrust) in strictly ascending path order,
-- each row a walked package `__init__.py` with no empty name.
pyAllOffence :: ResolveReq -> Set.Set String -> Maybe String
pyAllOffence rq walked =
  ascendingOn "py.all" id [p | (p, _, _) <- rqPyAll rq]
    <|> listToMaybe [label "py.all" i <> why | (i, (p, ns, _)) <- zip [0 :: Int ..] (rqPyAll rq), Just why <- [row p ns]]
 where
  row p ns
    | Set.notMember p walked || not (p == "__init__.py" || "/__init__.py" `isSuffixOf` p) = Just "not a walked __init__.py"
    | any null ns = Just "an empty name"
    | otherwise = Nothing

-- | `py.names` in strictly ascending site order, each row a Python
-- `import_from` site's with at least one name and no empty one (the
-- sites already in range: the site checks come first).
pyNamesOffence :: ResolveReq -> Maybe String
pyNamesOffence rq =
  asum
    [ asum [Just (label "py.names" i <> "not after the site before it") | (i, (a, b)) <- zip [1 :: Int ..] (zip ixs (drop 1 ixs)), a >= b]
    , listToMaybe [label "py.names" i <> why | (i, (at, ns)) <- zip [0 :: Int ..] (rqPyNames rq), Just why <- [row at ns]]
    ]
 where
  ixs = map fst (rqPyNames rq)
  sites = listArray (0, length (rqSites rq) - 1) (rqSites rq)
  row at ns
    | at < 0 || at >= length (rqSites rq) = Just "site out of range"
    | sLang (sites ! at) /= langPy || sKind (sites ! at) /= kindImportFrom = Just "not a Python import_from site"
    | null ns || any null ns = Just "no names, or an empty one"
    | otherwise = Nothing

-- | The assets, the anchor sets (each a walked Markdown file's) and the
-- reference tables in path order, and every Markdown reference site's
-- file with its table (the sites already in range: the site checks come
-- first).
mdOffence :: ResolveReq -> Set.Set String -> Maybe String
mdOffence rq walked =
  asum
    [ ascendingOn "asset" id (rqAssets rq)
    , ascendingOn "md.slugs" id (map fst (rqMdSlugs rq))
    , stray "md.slugs" "not a walked Markdown file" "markdown" walked (map fst (rqMdSlugs rq))
    , ascendingOn "md.refs" id [f | (f, _, _) <- rqMdRefs rq]
    , bare "a Markdown reference site without its file's table" (\s -> sLang s == langMd && sKind s `elem` [kindRefLink, kindRefDef]) tabled
    , ascendingOn "html.docs" id (map docPath (rqHtmlDocs rq))
    , stray "html.docs" "not an HTML file of the request" "html" known (map docPath (rqHtmlDocs rq))
    , bare "an HTML site without its file's document" ((== langHtml) . sLang) paged
    ]
 where
  -- a fact table's file that is not one of `among` or not of `lang`
  stray key why lang among fs = listToMaybe [label key i <> why | (i, f) <- zip [0 :: Int ..] fs, Set.notMember f among || not (ofLangs [lang] f)]
  -- a site `wants` facts for whose file is not in `held`
  bare why wants held = listToMaybe [label "site" i <> why | (i, s) <- zip [0 :: Int ..] (rqSites rq), wants s, Set.notMember (path ! sFrom s) held]
  tabled = Set.fromList [f | (f, _, _) <- rqMdRefs rq]
  docPath (p, _, _, _, _, _, _) = p
  paged = Set.fromList (map docPath (rqHtmlDocs rq))
  known = walked <> Set.fromList (rqOrigins rq)
  path = listArray (0, length (rqFiles rq) + length (rqOrigins rq) - 1) (rqFiles rq <> rqOrigins rq)
