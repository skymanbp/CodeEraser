{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceHtml (plan v2.33 W2-text stage H)
-- and the request a case turns into. A case is a tree of pages, Markdown
-- documents and other files with walked assets, each page's facts as
-- the measuring side's walk would send them (its `<html lang>`, a
-- `<base href>`, a canonical link or an `og:url`, hreflang alternates,
-- ids — duplicated, non-ASCII, escaped), each Markdown document's anchor
-- set, a declared `html` search root in some cases (declared empty in
-- one in twenty), and fifteen sites of the four HTML kinds with targets
-- relative, root-relative, absolute on the own host and others, odd,
-- escaped, behind a query, with fragments, and empty. No RNG: an LCG
-- over the case number (the Reference.hs posture).
module ReferenceHtmlGen (HCase (..), hCases, htmlReach, htmlRequest) where

import CE.Resolve.Cost (langHtml)
import CE.Resolve.HtmlHead (Doc (..))
import CE.Resolve.Tables (kindAction, kindHref)
import Data.Aeson (Value, object, (.=))
import Data.List (stripPrefix)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import ReferenceResolveGen (rands, siteRequest)

data HCase = HCase
  { hFiles :: [String]
  , hAssets :: [String]
  , hPages :: [(String, Doc)]
  , hSlugs :: [(String, [String])]
  , hRoots :: Maybe [String]
  , hSites :: [(Integer, String, String)]
  }

-- | Two hundred cases, fifteen sites each.
hCases :: [HCase]
hCases = map hCase [1 .. 200]

hCase :: Int -> HCase
hCase k = HCase files assets pages slugs roots sites
 where
  pick i xs = xs !! (rands k i `mod` length xs)
  maybePick i xs = if rands k i `mod` 3 == 0 then Nothing else Just (pick (i + 1) xs)
  files = Set.toList (Set.fromList [joined (pick (10 + i) dirs) (pick (30 + i) bases) | i <- [0 .. 11 :: Int]])
  htmls = filter (hasExt ["html", "htm"]) files
  assets = Set.toList (Set.fromList [pick (50 + i) assetNames | i <- [0 .. rands k 49 `mod` 3]])
  unwalked = "zz/u.html"
  pages = [(f, page n f) | (n, f) <- zip [0 ..] (htmls <> [unwalked])]
  page n f =
    Doc
      (maybePick (100 + 13 * n) langs)
      (if rands k (102 + 13 * n) `mod` 4 == 0 then Just (pick (103 + 13 * n) baseHrefs) else Nothing)
      (maybePick (104 + 13 * n) (ownly f))
      (maybePick (106 + 13 * n) (ownly f))
      [(pick (108 + 13 * n + j) langs, pick (109 + 13 * n + j) (ownly f)) | j <- [0 .. rands k (111 + 13 * n) `mod` 3 - 1]]
      [pick (112 + 13 * n + j) idWords | j <- [0 .. rands k (113 + 13 * n) `mod` 4]]
  slugs = [(f, [pick (400 + 7 * n + j) idWords | j <- [0 .. rands k (401 + 7 * n) `mod` 3]]) | (n, f) <- zip [0 ..] (filter (hasExt ["md", "markdown"]) files)]
  roots = case rands k 90 `mod` 20 of
    0 -> Just []
    1 -> Just ["", "site"]
    r | r < 6 -> Just [pick (91 + j) rootDirs | j <- [0 .. r `mod` 2]]
    _ -> Nothing
  froms = [pick (200 + i) (htmls <> [unwalked]) | i <- [0 .. 14]]
  sites = zipWith site [0 ..] froms
  site i from =
    let s j = 300 + 11 * i + j
        (o, pre) = (pick (s 1) origins, pick (s 2) prefixes)
        spec = o <> (if null o || take 1 pre == "/" then pre else '/' : pre) <> pick (s 3) targets <> pick (s 4) queries <> pick (s 5) frags
     in (pick (s 0) kinds, from, if rands k (s 6) `mod` 12 == 0 then "" else spec)

-- | The case's request: files then origins by index, the sites, the
-- assets, the Markdown anchor sets, the pages and the declared roots.
htmlRequest :: HCase -> Value
htmlRequest c = siteRequest langHtml (hFiles c) (hSites c) keys
 where
  keys seen =
    [ "assets" .= hAssets c
    , "md" .= object ["slugs" .= hSlugs c]
    , "html" .= object ["docs" .= M.elems (M.fromList [(p, (p, dLang g, dBase g, dCanonical g, dOgUrl g, dAlternates g, dIds g)) | (p, g) <- hPages c, p `elem` hFiles c || p `elem` seen])]
    ]
      <> ["config" .= object ["searchRoots" .= M.fromList [("html" :: String, r)]] | Just r <- [hRoots c]]

-- | Every answer the HTML cases must reach.
htmlReach :: [String]
htmlReach = ["file 1", "file 2", "file 3", "file 4", "external 5", "section 2", "section 2 slug", "section 4 slug", "Empty", "OutOfScope", "AmbiguousRoot"]

-- | A page's URL list: its own URLs weighted twice over the fixed list,
-- so a page that names itself (and so has a host and a root) is common.
ownly :: String -> [String]
ownly f = own f <> own f <> urls

-- | URLs that serve a page itself: its path under the tree root or under
-- `site/`, and the directory form of an index page.
own :: String -> [String]
own f = concat [("https://a.example/" <> p) : ["https://a.example/" <> d | Just d <- [indexDir p]] | r <- ["", "site/"], Just p <- [stripPrefix r f]]
 where
  indexDir p = case break (== '/') (reverse p) of
    (n, rest) | reverse n `elem` ["index.html", "index.htm"] -> Just (reverse rest)
    _ -> Nothing

-- | The kinds: a page reference, a form, two fetch kinds (any code but
-- `href` and `action` reads an empty value as nothing fetched).
kinds :: [Integer]
kinds = [kindHref, kindHref, kindAction, kindHref + 1, kindHref + 2]

joined :: String -> String -> String
joined d b = if null d then b else d <> "/" <> b

hasExt :: [String] -> String -> Bool
hasExt exts p = case break (== '.') (takeWhile (/= '/') (reverse p)) of
  (ext, '.' : _ : _) -> reverse ext `elem` exts
  _ -> False

dirs, bases, assetNames, langs, urls, baseHrefs, idWords, rootDirs, origins, prefixes, targets, queries, frags :: [String]
dirs = ["", "site", "site", "site/how", "site/zh", "docs"]
bases = ["index.html", "index.html", "a.html", "b.htm", "index.htm", "README.md", "g.md", "c.py", "\233.html", "a b.html"]
assetNames = ["img/a.png", "site/s.css", "docs/x.svg", "site/how/i.png"]
langs = ["en", "zh-Hans", "zh", "EN", ""]
urls = ["https://a.example/", "https://a.example/how/", "https://A.Example/how/index.html", "//a.example/zh/", "https://a.example/a", "https://a.example/a.html", "https://b.example/", "https://a.example/site/how/", "https://a.example/%73ite/", "https://a.example/a&amp;b/", "/how/", "https://a.example:8080/", "https://a.example/how/?v=1#x"]
baseHrefs = ["/site/", "../", "./", "https://a.example/site/", "https://b.example/", "mailto:x", "/nope/", "docs/", "a&amp;b/", "//a.example/"]
idWords = ["top", "intro", "top", "\20013\25991", "a b", "x%20y", "&amp;", ""]
rootDirs = ["", "site", "docs", "site/how"]
origins = ["", "", "", "", "https://a.example", "https://A.EXAMPLE", "//a.example", "https://b.example", "mailto:", "https://a.example:8080", "https://placeholder@a.example"]
prefixes = ["", "", "/", "/site/", "../", "./", "site/", "how/"]
targets = ["a.html", "index.html", "", "how/", "README.md", "g.md", "b.htm", "img/a.png", "s.css", "c.py", "zh", "x.html", "%C3%A9.html", "a%20b.html", "a&amp;b", "&#47;how/", ".."]
queries = ["", "", "", "?q", "?a=1&amp;b=2"]
frags = ["", "", "", "#top", "#intro", "#", "#\20013\25991", "#%E4%B8%AD%E6%96%87", "#a%20b", "#a b", "#x%20y", "#&amp;", "#nope"]
