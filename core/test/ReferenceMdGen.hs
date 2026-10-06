{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceMd (plan v2.33 W2-text stage G) and
-- the request a case turns into. A case is a tree of Markdown and other
-- files with walked assets, each walked Markdown file's anchor set (as
-- the measuring side would have slugged it: duplicates, non-ASCII, a
-- space, a percent sign), and per file of a reference site its
-- definitions in text order and the labels its reference links write —
-- labels across case, whitespace, a final sigma, a dotted capital I, a
-- titlecase digraph — and sites of the five kinds with targets plain,
-- relative, odd, percent-escaped (well-formed or not) and fragments.
-- No RNG: an LCG over the case number (the Reference.hs posture).
module ReferenceMdGen (MCase (..), mdCases, mdRequest, isMdName) where

import CE.Resolve.Cost (langMd)
import CE.Resolve.Tables (kindImage, kindLink, kindRefDef, kindRefLink, kindUrl)
import Data.Aeson (Value, object, (.=))
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import ReferenceResolveGen (originsOf, rands, requestHeader)

data MCase = MCase
  { mFiles :: [String]
  , mAssets :: [String]
  , mSlugs :: [(String, [String])]
  , mRefs :: [(String, [(String, String)], [String])]
  , mSites :: [(Integer, String, String)]
  }

-- | Two hundred cases, fifteen sites each (three per kind).
mdCases :: [MCase]
mdCases = map mdCase [1 .. 200]

mdCase :: Int -> MCase
mdCase k = MCase files assets slugs refs sites
 where
  pick i xs = xs !! (rands k i `mod` length xs)
  files = Set.toList (Set.fromList [joined (pick (10 + i) dirs) (pick (30 + i) bases) | i <- [0 .. 9 :: Int]])
  mds = filter isMdName files
  assets = Set.toList (Set.fromList [pick (50 + i) assetNames | i <- [0 .. rands k 49 `mod` 3]])
  slugs = [(f, [pick (100 + 7 * n + j) slugWords | j <- [0 .. rands k (60 + n) `mod` 4]]) | (n, f) <- zip [0 ..] mds]
  froms = [pick (200 + i) (mds <> ["zz/u.md"]) | i <- [0 .. 14]]
  sites = zipWith site [0 ..] froms
  site i from =
    let kind = kinds !! (i `mod` 5)
        s j = 300 + 11 * i + j
     in (kind, from, specOf kind (pick (s 0) labels) (pick (s 1) prefixes <> pick (s 2) targets <> pick (s 3) frags))
  specOf kind label target
    | kind == kindRefLink = label
    | otherwise = target
  tabled = Set.toList (Set.fromList [f | (kind, f, _) <- sites, kind `elem` [kindRefLink, kindRefDef]])
  refs = [(f, defs n, [pick (600 + 9 * n + j) labels | j <- [0 .. rands k (590 + n) `mod` 4]]) | (n, f) <- zip [0 ..] tabled]
  defs n = [(pick (700 + 9 * n + j) labels, pick (800 + 9 * n + j) targets) | j <- [0 .. rands k (690 + n) `mod` 6]]

kinds :: [Integer]
kinds = [kindLink, kindImage, kindRefLink, kindRefDef, kindUrl]

joined :: String -> String -> String
joined d b = if null d then b else d <> "/" <> b

-- | A file name of the Markdown extensions, by its last dot after a
-- non-empty stem.
isMdName :: String -> Bool
isMdName p = case break (== '.') (reverse (lastSegment p)) of
  (ext, '.' : _ : _) -> reverse ext `elem` ["md", "markdown"]
  _ -> False
 where
  lastSegment = reverse . takeWhile (/= '/') . reverse

dirs, bases, assetNames, slugWords, labels, prefixes, targets, frags :: [String]
dirs = ["", "", "docs", "docs/sub", "a"]
bases = ["README.md", "a.md", "b.md", "x.markdown", "c.py", "A.MD", "\233.md", ".md", "a.b.md"]
assetNames = ["img/logo.png", "docs/x.svg", "s.css", "a.md.png"]
slugWords = ["intro", "intro-1", "faq", "faq", "\948\961\972\956\959\962", "x y", "a%b", "\233t\233"]
labels = ["Guide", "guide", " GUIDE  ", "\927\916\927\931", "\959\948\959\962", "\959\948\959\963", "\304", "i\775", "Foo\tBar", "foo bar", "missing", "\453", "\454", "A'\931", "\931"]
prefixes = ["", "", "", "../", "docs/", "./", "a/../"]
targets =
  [ "a.md", "README.md", "b.md", "docs/", "docs", "x.markdown", "", "#", "#intro", "https:x", "//h/x", "/abs.md", "a%2Emd"
  , "%2e%2e/b.md", "%C3%A9.md", "%ZZ.md", "%+f", "img/logo.png", "../img/logo.png", "..", ".", "c.py", "nowhere/"
  , "%ED%A0%80", "a1+.-:x", "1a:x", "docs/sub", "A.MD", "s.css"
  ]
frags = ["", "", "", "#intro", "#faq", "#x%20y", "#", "#\948\961\972\956\959\962", "#%CE%B4%CF%81%CF%8C%CE%BC%CE%BF%CF%82", "#nope", "#a%b", "#a%25b", "#%C3%A9t%C3%A9"]

-- | The case's request: files then origins by index, the sites, the
-- assets and the Markdown facts.
mdRequest :: MCase -> Value
mdRequest c =
  object $
    requestHeader (mFiles c) origins
      <> [ "sites" .= [(langMd, kind, M.findWithDefault 0 from ix, spec) | (kind, from, spec) <- mSites c]
         , "assets" .= mAssets c
         , "md" .= object ["slugs" .= mSlugs c, "refs" .= mRefs c]
         ]
 where
  (origins, ix) = originsOf (mFiles c) [f | (_, f, _) <- mSites c]
