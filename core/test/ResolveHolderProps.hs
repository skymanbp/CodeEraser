{-# LANGUAGE OverloadedStrings #-}

-- | The section holder's `also` rows (lane v235_licence, part 4C;
-- CE.Resolve.Answer `holderOf`): a site whose answer is a confirmed
-- section of ANOTHER file also references that file, at the answer's
-- rung, the answer itself untouched. One request mixes a Python import,
-- Markdown links (to another file, an unconfirmed fragment, a page's own
-- section by path and by bare fragment), an unused and a used reference
-- definition and an HTML `id` link: the answers stand, the rows come in
-- (row, target) order, and the three "none" cases draw none. Over the
-- seeded Markdown and HTML cases every row is exactly the holder of a
-- confirmed section of another file. The Python legs stand apart
-- (ResolvePyAlsoProps), unchanged.
module ResolveHolderProps (battery) where

import CE.Resolve.Cost (langHtml, langMd, langPy)
import CE.Resolve.Tables (kindHref, kindImportFrom, kindLink, kindRefDef, kindRefLink)
import Data.Aeson (Value (Null), object, toJSON, (.=))
import Data.Maybe (isJust)
import ReferenceHtmlGen (HCase (..), hCases, htmlRequest)
import ReferenceMdGen (MCase (..), mdCases, mdRequest)
import ReferenceResolveGen
import WireHarness (runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "holder also: the mixed request's answers stand, the sections included"
    , "holder also: rows in (row, target) order: the Python name, the confirmed sections of other files at their rungs"
    , "holder also: 200 seeded Markdown cases, every row the holder of a confirmed section of another file"
    , "holder also: 200 seeded HTML cases, every row the holder of a confirmed section of another file"
    ]
    [ fmap fst mixed == Just mixedAnswers
    , fmap snd mixed == Just mixedAlso
    , all (holdersOnly mSites mdRequest) mdCases
    , all (holdersOnly hSites htmlRequest) hCases
    ]
 where
  holdersOnly sites req c = maybe False (uncurry (holders (sites c))) (askBoth (req c))

-- | A case's answers and `also` rows, as the core gives them.
askBoth :: Value -> Maybe ([Ref], [(Int, String, Int)])
askBoth req = do
  v <- askCore req
  (,) <$> answers v <*> replyKey "also" v

-- | The rows a case's answers must draw: one per confirmed section of a
-- file other than the site's own, at the answer's rung.
holders :: [(Integer, String, String)] -> [Ref] -> [(Int, String, Int)] -> Bool
holders sites got rows = rows == [(i, t, r) | (i, (_, from, _), RSection t slug r) <- zip3 [0 ..] sites got, isJust slug, t /= from]

-- | The mixed request's answers and `also` rows.
mixed :: Maybe ([Ref], [(Int, String, Int)])
mixed = askBoth mixedRequest

mixedFiles :: [String]
mixedFiles = ["README.md", "c.md", "index.html", "main.py", "me.md", "page2.html", "pkg/__init__.py", "pkg/sub.py"]

-- | The sites `(lang, kind, from, spec)`: 0 a Python `from pkg import
-- sub`; 1 a link into another file's heading; 2 its fragment no heading
-- confirms; 3 a page's own section by path; 4 by bare fragment; 5 an
-- unused definition; 6 an HTML link to another page's `id`; 7 a used
-- reference link.
mixedSites :: [(Integer, Integer, String, String)]
mixedSites =
  [ (langPy, kindImportFrom, "main.py", "pkg")
  , (langMd, kindLink, "README.md", "c.md#sec")
  , (langMd, kindLink, "README.md", "c.md#nosuch")
  , (langMd, kindLink, "me.md", "me.md#part")
  , (langMd, kindLink, "me.md", "#part")
  , (langMd, kindRefDef, "README.md", "me.md#part")
  , (langHtml, kindHref, "index.html", "page2.html#s1")
  , (langMd, kindRefLink, "README.md", "used")
  ]

mixedRequest :: Value
mixedRequest =
  object $
    requestHeader mixedFiles []
      <> [ "sites" .= [[toJSON l, toJSON k, toJSON (ix from), toJSON spec] | (l, k, from, spec) <- mixedSites]
         , "py" .= object ["names" .= [(0 :: Int, ["sub" :: String])]]
         , "md" .= object ["slugs" .= slugs, "refs" .= refs]
         , "html" .= object ["docs" .= [page "index.html" [], page "page2.html" ["s1"]]]
         ]
 where
  ix from = length (takeWhile (/= from) mixedFiles)
  slugs = [("c.md", ["sec"]), ("me.md", ["part"]) :: (String, [String])]
  refs = [("README.md", [("unused", "me.md#part"), ("used", "c.md#sec")], ["used"]) :: (String, [(String, String)], [String])]
  page p ids = toJSON [toJSON (p :: String), Null, Null, Null, Null, toJSON ([] :: [Value]), toJSON (ids :: [String])]

mixedAnswers :: [Ref]
mixedAnswers =
  [ RFile "pkg/__init__.py" 2
  , RSection "c.md" (Just "sec") 2
  , RSection "c.md" Nothing 2
  , RSection "me.md" (Just "part") 2
  , RSection "me.md" (Just "part") 4
  , RInert "me.md" 3
  , RSection "page2.html" (Just "s1") 2
  , RSection "c.md" (Just "sec") 3
  ]

mixedAlso :: [(Int, String, Int)]
mixedAlso = [(0, "pkg/sub.py", 2), (1, "c.md", 2), (6, "page2.html", 2), (7, "c.md", 3)]
