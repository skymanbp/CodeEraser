-- | An independently written reference for the HTML rungs (plan v2.33
-- W2-text stage H), written apart from CE.Resolve.Html and
-- CE.Resolve.HtmlHead: a page's own URL is chosen by sorting the
-- candidates (a stable sort, so the first shortest root wins), the
-- character references decode by a scan that reads a reference's number
-- with `span`, a rung's hits are a de-duplicated list, the directory and
-- file tests scan every walked path, and the path join and the
-- percent-decoding are ReferenceMd's. Every site of the two hundred
-- cases must get the same answer from both, and the cases reach every
-- answer the HTML rungs give.
module ReferenceHtml (equivalence) where

import CE.Resolve.Cost (Reason (..))
import CE.Resolve.HtmlHead (Doc (..))
import CE.Resolve.Tables (kindAction, kindHref)
import Data.Char (chr, digitToInt, isAsciiLower, isAsciiUpper, isDigit, isHexDigit, ord, toLower)
import Data.List (isPrefixOf, isSuffixOf, nub, sortOn, stripPrefix)
import Data.Maybe (listToMaybe, mapMaybe)
import ReferenceHtmlGen
import ReferenceMd (decode, dirOf, pieces, schemed, walk)
import ReferenceResolveGen (Ref (..), refEquivalence)

equivalence :: IO Bool
equivalence = refEquivalence "HTML" hCases (htmlRequest, hSites, refSite) htmlReach

-- | What a page says of itself: (base, host, root).
type Said = (Maybe String, Maybe String, Maybe String)

said :: HCase -> String -> Said
said c from = case lookup from (hPages c) of
  Nothing -> (Nothing, Nothing, Nothing)
  Just g -> (unescape <$> dBase g, fst <$> own g, snd <$> own g)
 where
  own g = case (dCanonical g, dOgUrl g) of
    (Just u, _) -> serves from (unescape u)
    (_, Just u) -> serves from (unescape u)
    _ -> listToMaybe (sortOn (bytes . snd) (mapMaybe (serves from . unescape . snd) (filter (sameLang (dLang g) . fst) (dAlternates g))))
  sameLang Nothing _ = True
  sameLang (Just l) h = subtag l == subtag h
  subtag = map lowerAscii . takeWhile (/= '-')
  bytes = sum . map (\ch -> length (filter (<= ord ch) [0, 0x80, 0x800, 0x10000]))

lowerAscii :: Char -> Char
lowerAscii ch = if isAsciiUpper ch then toLower ch else ch

-- | The (host, root) a URL names the page `from` by.
serves :: String -> String -> Maybe (String, String)
serves from url = do
  (host, rest) <- origin url
  let p = decode (dropWhile (== '/') (fst (break (`elem` ("?#" :: String)) rest)))
      lastSeg = reverse (takeWhile (/= '/') (reverse p))
      tries
        | null p || "/" `isSuffixOf` p = map (p <>) ["index.html", "index.htm"]
        | '.' `elem` lastSeg = [p]
        | otherwise = map (p <>) ["/index.html", "/index.htm", ".html", ".htm"]
      hit t
        | t == from = Just ""
        | ('/' : t) `isSuffixOf` from = Just (take (length from - length t - 1) from)
        | otherwise = Nothing
  root <- listToMaybe (mapMaybe hit tries)
  Just (host, root)

-- | A URL's (host, the rest) when it carries an origin.
origin :: String -> Maybe (String, String)
origin url = do
  afterSlashes <- case stripPrefix "//" url of
    Just r -> Just r
    Nothing -> case [(take i url, r) | i <- [0 .. length url], Just r <- [stripPrefix "://" (drop i url)]] of
      ((sc, r) : _) | not (null sc) && all okScheme sc -> Just r
      _ -> Nothing
  let (auth, rest) = span (`notElem` ("/?#" :: String)) afterSlashes
  Just (map lowerAscii (lastAt auth), rest)
 where
  lastAt a = case break (== '@') a of
    (_, '@' : more) -> lastAt more
    (h, _) -> h
  okScheme ch = isAsciiLower ch || isAsciiUpper ch || isDigit ch || ch `elem` ("+-." :: String)

-- | The character references decoded.
unescape :: String -> String
unescape [] = []
unescape ('&' : more) = case break (== ';') more of
  (name, ';' : after) | Just ch <- ref name -> ch : unescape after
  _ -> '&' : unescape more
unescape (ch : more) = ch : unescape more

ref :: String -> Maybe Char
ref name = case lookup name [("amp", '&'), ("lt", '<'), ("gt", '>'), ("quot", '"'), ("apos", '\'')] of
  Just ch -> Just ch
  Nothing -> case name of
    '#' : x : hex | x == 'x' || x == 'X' -> numeral 16 isHexDigit hex
    '#' : dec -> numeral 10 isDigit dec
    _ -> Nothing
 where
  numeral base ok s = case span (\d -> d < '\128' && ok d) (maybe s id (stripPrefix "+" s)) of
    (ds@(_ : _), "") ->
      let v = foldl (\acc d -> acc * base + toInteger (digitToInt d)) (0 :: Integer) ds
       in if v < 1 || v > 0x10FFFF || (v >= 0xD800 && v < 0xE000) then Nothing else Just (chr (fromInteger v))
    _ -> Nothing

-- | One site's reference answer.
refSite :: HCase -> (Integer, String, String) -> Ref
refSite c (kind, from, spec)
  | null spec = if kind == kindHref || kind == kindAction then RFile from 4 else RUnres Empty
  | Just (host, rest) <- origin s = if Just host == pageHost then place c from (if null rest then "/" else rest) else RExt 5
  | "//" `isPrefixOf` s || schemed s = RExt 5
  | otherwise = place c from s
 where
  s = unescape spec
  (_, pageHost, _) = said c from

place :: HCase -> String -> String -> Ref
place c from r = case path of
  "" -> if maybe False (not . null) frag then RSection from frag 4 else RFile from 4
  '/' : rel -> either id fragOn (underRoot c from rel servesHere)
  _ -> either id fragOn (besideBase c from path)
 where
  (path, frag) = case break (== '#') r of
    (p, '#' : f) -> (takeWhile (/= '?') p, Just f)
    (p, _) -> (takeWhile (/= '?') p, Nothing)
  servesHere t = served c t
  fragOn (t, n) = case frag of
    Just f@(_ : _)
      | hasExtR ["md", "markdown"] t -> RSection t (only (decode f) (concat [ss | (p, ss) <- hSlugs c, p == t])) 2
      | hasExtR ["html", "htm"] t -> RSection t (only (decode f) (concat [dIds g | (p, g) <- hPages c, p == t])) 2
    _ -> RFile t n
  only x xs = if length (filter (== x) xs) == 1 then Just x else Nothing

hasExtR :: [String] -> String -> Bool
hasExtR exts p = case break (== '.') (reverse (last (pieces p))) of
  (e, '.' : _ : _) -> reverse e `elem` exts
  _ -> False

-- | The walked file or index page a tree path serves.
served :: HCase -> String -> Maybe String
served c t
  | t `elem` hFiles c || t `elem` hAssets c = Just t
  | otherwise = listToMaybe [i | name <- ["index.html", "index.htm"], let i = if null t then name else t <> "/" <> name, i `elem` hFiles c]

-- | A directory walked files or assets live under.
holds :: HCase -> String -> Maybe String
holds c d
  | null d || any ((d <> "/") `isPrefixOf`) (hFiles c <> hAssets c) = Just d
  | otherwise = Nothing

-- | R1: beside the page, or beside its base's directory.
besideBase :: HCase -> String -> String -> Either Ref (String, Int)
besideBase c from p = do
  dir <- case base of
    Nothing -> Right (dirOf from)
    Just b -> pieces <$> baseDirectory c from b
  t <- maybe (Left (RUnres OutOfScope)) Right (walk dir (decode p))
  maybe (Left (RUnres OutOfScope)) (\x -> Right (x, 1)) (served c t)
 where
  (base, _, _) = said c from

baseDirectory :: HCase -> String -> String -> Either Ref String
baseDirectory c from b = case origin b of
  Just (h, rest)
    | Just h /= pageHost -> Left (RExt 5)
    | otherwise -> fst <$> underRoot c from (parentOf (dropWhile (== '/') rest)) (holds c)
  Nothing
    | "//" `isPrefixOf` b || schemed b -> Left (RExt 5)
    | '/' : rel <- b -> fst <$> underRoot c from (parentOf rel) (holds c)
    | otherwise -> maybe (Left (RUnres OutOfScope)) Right (walk (dirOf from) (parentOf b) >>= holds c)
 where
  (_, pageHost, _) = said c from
  parentOf x = case dirOf x of
    [] -> ""
    segs -> foldr1 (\a z -> a <> "/" <> z) segs

-- | R2 / R3 under the declared roots, the page's root, or its ancestors.
underRoot :: HCase -> String -> String -> (String -> Maybe String) -> Either Ref (String, Int)
underRoot c from rel hit = settle (nub (mapMaybe (\d -> walk (pieces d) (decode rel) >>= hit) dirs))
 where
  (_, _, pageRoot) = said c from
  (dirs, rung) = case (hRoots c, pageRoot) of
    (Just ds, _) -> (ds, 2)
    (Nothing, Just r) -> ([r], 2)
    _ -> ("" : scanl1 (\a z -> a <> "/" <> z) (dirOf from), 3)
  settle hits = case hits of
    [one] -> Right (one, rung)
    [] -> Left (RUnres OutOfScope)
    _ -> Left (RUnres AmbiguousRoot)
