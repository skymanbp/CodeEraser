-- | What one HTML document says about itself, derived for the HTML rungs
-- (plan v2.33 W2-text stage H; once cli/src/graph/ladder/html_head.rs
-- `read` after its walk, `self_url`, `served_from`, `split_origin`,
-- `decode_refs` and `decoded`). The measuring side walks the document's
-- syntax tree and sends what it wrote (`html.docs`: its `<html lang>`,
-- first `<base href>`, first canonical link, first `og:url`, its
-- hreflang alternates in document order, each value as written and
-- non-empty, and its `id` set); here the base is decoded and the host
-- and tree root of the URL that names the page are derived: the
-- canonical (or, without one, the og:url) when it names the page — a
-- canonical pointing elsewhere is the page saying it has no URL of its
-- own — else the alternate in the page's own language (`<html lang>`'s
-- primary subtag; every alternate when none is declared) whose root is
-- shortest. Nothing here consults the tree.
module CE.Resolve.HtmlHead (Doc (..), Page (..), page, servedFrom, splitOrigin, decodeRefs) where

import CE.Resolve.Str (splitOnceStr, stripSuffix, utf8Len)
import CE.Resolve.Url (percentDecode)
import Data.Char (chr, isAsciiLower, isAsciiUpper, isDigit, isHexDigit, digitToInt, toLower)
import Data.List (isPrefixOf, isSuffixOf, minimumBy)
import Data.Maybe (mapMaybe)
import Data.Ord (comparing)

-- | One document's facts as written: `<html lang>`, the first
-- `<base href>`, the first canonical href, the first `og:url`, every
-- `(hreflang, href)` alternate, the `id` values in document order.
data Doc = Doc
  { dLang :: Maybe String
  , dBase :: Maybe String
  , dCanonical :: Maybe String
  , dOgUrl :: Maybe String
  , dAlternates :: [(String, String)]
  , dIds :: [String]
  }

-- | What the rungs read of a page: its base decoded, the host of its own
-- served URL and the tree directory that host serves.
data Page = Page {pBase :: Maybe String, pHost :: Maybe String, pRoot :: Maybe String}

-- | `read`'s derivation for the document at tree path `from`.
page :: String -> Doc -> Page
page from d = Page (decodeRefs <$> dBase d) (fst <$> own) (snd <$> own)
 where
  own = selfUrl from (dLang d) (maybe (dOgUrl d) Just (dCanonical d)) (dAlternates d)

-- | `self_url`: the declared URL's (host, root) — no fallback when it
-- names another page — else the matching alternates' with the shortest
-- root in UTF-8 bytes, the first such (`min_by_key`).
selfUrl :: String -> Maybe String -> Maybe String -> [(String, String)] -> Maybe (String, String)
selfUrl from lang declared alternates = case declared of
  Just url -> servedFrom from (decodeRefs url)
  Nothing -> case mapMaybe (servedFrom from . decodeRefs . snd) (filter mine alternates) of
    [] -> Nothing
    hits -> Just (minimumBy (comparing (utf8Len . snd)) hits)
 where
  mine (hreflang, _) = maybe True (\l -> primary l == primary hreflang) lang
  primary = map asciiLower . takeWhile (/= '-')

-- | `served_from`: the (host, root) a URL yields when its path — query
-- and fragment cut, every leading `/` trimmed, percent-decoded — names
-- the page at `from`: the tree paths it could serve (a directory URL its
-- index page, a last segment with a `.` itself, any other that page or
-- `name.html`), the first that is `from` (root "") or ends it at a
-- directory boundary (the root the head before that `/`).
servedFrom :: String -> String -> Maybe (String, String)
servedFrom from url = do
  (host, path) <- splitOrigin url
  let p = percentDecode (dropWhile (== '/') (takeWhile (`notElem` ("?#" :: String)) path))
      candidates
        | null p || "/" `isSuffixOf` p = [p <> "index.html", p <> "index.htm"]
        | '.' `elem` reverse (takeWhile (/= '/') (reverse p)) = [p]
        | otherwise = [p <> t | t <- ["/index.html", "/index.htm", ".html", ".htm"]]
      rootOf c
        | from == c = Just ""
        | otherwise = stripSuffix c from >>= stripSuffix "/"
  case mapMaybe rootOf candidates of
    (root : _) -> Just (host, root)
    [] -> Nothing

-- | `split_origin`: an absolute (`scheme://`, the scheme non-empty ASCII
-- letters, digits, `+`, `-` or `.`) or protocol-relative (`//`) URL as
-- its host — the authority up to the first `/`, `?` or `#`, after its
-- last `@`, ASCII-lowercased — and the rest (path, query, fragment);
-- none for anything else.
splitOrigin :: String -> Maybe (String, String)
splitOrigin url = do
  rest <- if "//" `isPrefixOf` url then Just (drop 2 url) else schemed
  let (authority, path) = break (`elem` ("/?#" :: String)) rest
      host = reverse (takeWhile (/= '@') (reverse authority))
  Just (map asciiLower host, path)
 where
  schemed = case splitOnceStr "://" url of
    Just (scheme, rest) | not (null scheme) && all schemeChar scheme -> Just rest
    _ -> Nothing
  schemeChar c = isAsciiUpper c || isAsciiLower c || isDigit c || c `elem` ("+-." :: String)

-- | `decode_refs`: each `&` that opens a reference this reading knows —
-- up to the first `;` after it — is that character; any other `&` stays
-- as written and the scan goes on after it.
decodeRefs :: String -> String
decodeRefs s = case break (== '&') s of
  (before, '&' : rest) -> before <> maybe ('&' : decodeRefs rest) (\(c, after) -> c : decodeRefs after) (decoded rest)
  (before, _) -> before

-- | `decoded`: the reference body before the first `;` — `amp`, `lt`,
-- `gt`, `quot`, `apos`, or `#` and a decimal or `x` / `X` and a
-- hexadecimal number (Rust's `u32` reading: an optional leading `+`, at
-- least one ASCII digit, no overflow past u32) naming a scalar value
-- other than U+0000 — with the text after the `;`.
decoded :: String -> Maybe (Char, String)
decoded rest = case break (== ';') rest of
  (name, ';' : after) -> (\c -> (c, after)) <$> named name
  _ -> Nothing
 where
  named name = case name of
    "amp" -> Just '&'
    "lt" -> Just '<'
    "gt" -> Just '>'
    "quot" -> Just '"'
    "apos" -> Just '\''
    '#' : digits -> numeric digits >>= scalar
    _ -> Nothing
  numeric digits = case digits of
    x : hex | x `elem` ("xX" :: String) -> number 16 isHexDigit hex
    _ -> number 10 isDigit digits
  number base ok ds = case dropPlus ds of
    body@(_ : _) | all (\c -> c < '\128' && ok c) body -> let v = foldl (\a c -> a * base + toInteger (digitToInt c)) 0 body in if v > 4294967295 then Nothing else Just v
    _ -> Nothing
  dropPlus ('+' : ds) = ds
  dropPlus ds = ds
  scalar v
    | v == 0 || v > 0x10FFFF || (v >= 0xD800 && v <= 0xDFFF) = Nothing
    | otherwise = Just (chr (fromInteger v))

asciiLower :: Char -> Char
asciiLower c = if isAsciiUpper c then toLower c else c
