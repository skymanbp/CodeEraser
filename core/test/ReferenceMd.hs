-- | An independently written reference for the Markdown rungs (plan
-- v2.33 W2-text stage G), written apart from CE.Resolve.Md and
-- CE.Resolve.Url: the path join is a stack walk, the directory test a
-- scan of every file, the first-wins table an association list read by
-- `lookup` (its first entry wins), the percent-decoding a byte walk whose
-- UTF-8 check measures each code point against its shortest form, the
-- surrogates and the top of the code space. The label fold shares the
-- core's lowercase (CE.Resolve.Lower): the tests subrepo holds that
-- against the measuring side's standard library over every scalar value.
-- Every site of the two hundred cases must get the same answer from
-- both, and the cases reach every answer the Markdown rungs give.
module ReferenceMd (equivalence) where

import CE.Resolve (respond)
import CE.Resolve.Chars (isRustWhite)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Lower (rustLower)
import CE.Resolve.Tables (kindImage, kindLink, kindRefDef, kindRefLink, kindUrl)
import Data.Aeson (decodeStrict, encode)
import Data.Bits (shiftL, shiftR, (.&.), (.|.))
import qualified Data.ByteString.Lazy as BL
import Data.Char (chr, digitToInt, isAsciiLower, isAsciiUpper, isDigit, isHexDigit, ord)
import Data.List (isPrefixOf, unfoldr)
import qualified Data.Set as Set
import ReferenceMdGen
import ReferenceResolveGen (Ref (..), answers, refShape)
import WireHarness (battery, runChecks)

equivalence :: IO Bool
equivalence =
  runChecks
    ( battery
        ("resolve: 200 Markdown cases, 3000 sites, shipped = reference", "resolve: the Markdown cases reach every answer of the Markdown rungs")
        (map disagree mdCases)
        ["file 1", "file 3", "file 4", "package 1", "package 3", "external 3", "external 5", "section 2", "section 2 slug", "section 3 slug", "section 4 slug", "inert 3", "OutOfScope"]
        (Set.fromList [refShape (refSite c s) | c <- mdCases, s <- mSites c])
    )

disagree :: MCase -> Maybe String
disagree c = case either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode (mdRequest c)))) >>= answers of
  Nothing -> Just "no reply"
  Just got
    | got /= want -> Just ("core " <> show got <> " reference " <> show want)
    | otherwise -> Nothing
 where
  want = map (refSite c) (mSites c)

refSite :: MCase -> (Integer, String, String) -> Ref
refSite c (kind, from, spec)
  | kind == kindUrl = RExt 5
  | kind == kindRefLink = maybe (RUnres OutOfScope) (relabel . chain c from) (lookup (normal spec) defs)
  | kind == kindRefDef = defined
  | kind == kindLink || kind == kindImage = chain c from spec
  | otherwise = RUnres Unsupported
 where
  (defs, used) = tableOf c from
  defined
    | any (\(l, t) -> t == spec && l `elem` used) (firsts defs) = relabel (chain c from spec)
    | otherwise = inert (relabel (chain c from spec))
  inert a = case a of
    RFile p r -> RInert p r
    RSection p _ r -> RInert p r
    RPkg d r -> RInert d r
    other -> other

-- | The file's definitions, folded, in text order, and its used labels
-- folded.
tableOf :: MCase -> String -> ([(String, String)], [String])
tableOf c from = case [(ds, us) | (f, ds, us) <- mRefs c, f == from] of
  ((ds, us) : _) -> ([(normal l, t) | (l, t) <- ds], map normal us)
  [] -> ([], [])

-- | Each label's first entry.
firsts :: [(String, String)] -> [(String, String)]
firsts = go []
 where
  go _ [] = []
  go seen ((l, t) : rest)
    | l `elem` seen = go seen rest
    | otherwise = (l, t) : go (l : seen) rest

normal :: String -> String
normal = rustLower . unwords' . unfoldr word
 where
  word s = case dropWhile isRustWhite s of
    "" -> Nothing
    t -> Just (break isRustWhite t)
  unwords' ws = concat (zipWith (<>) ("" : repeat " ") ws)

relabel :: Ref -> Ref
relabel a = case a of
  RFile p _ -> RFile p 3
  RPkg d _ -> RPkg d 3
  RSection p s _ -> RSection p s 3
  RExt _ -> RExt 3
  other -> other

chain :: MCase -> String -> String -> Ref
chain c from spec
  | "//" `isPrefixOf` spec || schemed spec = RExt 5
  | "/" `isPrefixOf` spec = RUnres OutOfScope
  | null path = case frag of
      Just f@(_ : _) -> RSection from (Just f) 4
      _ -> RFile from 4
  | otherwise = maybe (RUnres OutOfScope) located (walk (dirOf from) (decode path))
 where
  (path, rest) = span (/= '#') spec
  frag = if null rest then Nothing else Just (drop 1 rest)
  located t
    | t `elem` mFiles c && isMdName t && maybe False (not . null) frag = anchored t (maybe "" id frag)
    | t `elem` mFiles c || t `elem` mAssets c = RFile t 1
    | any ((if null t then "" else t <> "/") `isPrefixOf`) (mFiles c) = RPkg t 1
    | otherwise = RUnres OutOfScope
  anchored t f = RSection t (if length (filter (== decode f) (concat [ss | (p, ss) <- mSlugs c, p == t])) == 1 then Just (decode f) else Nothing) 2

-- | The scheme head: letters, digits, `+ - .` after a letter, then `:`,
-- before any `/` or `#`.
schemed :: String -> Bool
schemed s = case span (\x -> isAsciiLower x || isAsciiUpper x || isDigit x || x `elem` ("+-." :: String)) s of
  (h : _, ':' : _) -> isAsciiLower h || isAsciiUpper h
  _ -> False

dirOf :: String -> [String]
dirOf p = init' (pieces p)
 where
  init' xs = take (length xs - 1) xs

pieces :: String -> [String]
pieces s = case break (== '/') s of
  (a, _ : more) -> a : pieces more
  (a, []) -> [a]

-- | The directory's segments, then each of the target's: `..` pops (none
-- left: out of the tree), empty and `.` stay.
walk :: [String] -> String -> Maybe String
walk dir target = go (reverse (filter (not . null) dir)) (pieces target)
 where
  go stack [] = Just (concat (zipWith (<>) ("" : repeat "/") (reverse stack)))
  go stack (seg : segs)
    | seg == "" || seg == "." = go stack segs
    | seg == ".." = case stack of
        (_ : below) -> go below segs
        [] -> Nothing
    | otherwise = go (seg : stack) segs

decode :: String -> String
decode s
  | '%' `notElem` s = s
  | otherwise = maybe s id (chars (bytes (concatMap encode1 s)))
 where
  bytes ('%' : a : b : more) | isHexDigit a && isHexDigit b = chr (16 * digitToInt a + digitToInt b) : bytes more
  bytes ('%' : '+' : b : more) | isHexDigit b = chr (digitToInt b) : bytes more
  bytes (x : more) = x : bytes more
  bytes [] = []

-- | A character's UTF-8 bytes, each as a character below 256.
encode1 :: Char -> String
encode1 ch = map chr (go (ord ch))
 where
  go n
    | n < 0x80 = [n]
    | n < 0x800 = [0xC0 .|. shiftR n 6, cont n]
    | n < 0x10000 = [0xE0 .|. shiftR n 12, cont (shiftR n 6), cont n]
    | otherwise = [0xF0 .|. shiftR n 18, cont (shiftR n 12), cont (shiftR n 6), cont n]
  cont n = 0x80 .|. (n .&. 0x3F)

-- | Bytes (characters below 256) as UTF-8, or none: each sequence's
-- code point must need its length, stay off the surrogates and under
-- U+110000.
chars :: String -> Maybe String
chars [] = Just []
chars (b : more)
  | ord b < 0x80 = (b :) <$> chars more
  | otherwise = do
      n <- lookup (ord b `shiftR` 3) [(x, 1) | x <- [0x18 .. 0x1B]] `orElse` lookup (ord b `shiftR` 4) [(0xE, 2)] `orElse` lookup (ord b `shiftR` 3) [(0x1E, 3)]
      let (tailBytes, rest) = splitAt n more
      if length tailBytes /= n || any (\t -> ord t .&. 0xC0 /= 0x80) tailBytes then Nothing else Just ()
      let cp = foldl (\acc t -> shiftL acc 6 .|. (ord t .&. 0x3F)) (ord b .&. (0xFF `shiftR` (n + 2))) tailBytes
          floor' = [0x80, 0x800, 0x10000] !! (n - 1)
      if cp < floor' || (cp >= 0xD800 && cp <= 0xDFFF) || cp > 0x10FFFF then Nothing else (chr cp :) <$> chars rest
 where
  orElse (Just x) _ = Just x
  orElse Nothing y = y
