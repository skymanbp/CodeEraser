-- | The resolve.request boundary contract (plan v2.33 wave W2a; on text
-- since W2-text): what a well-formed request IS — the walked paths in
-- strictly ascending (path) order, the origins too and none of them
-- walked, every site in a language this family resolves with its file in
-- range and — a Java site — its line, the Java headers in strictly
-- ascending (path) order, every database probe one of the three, every
-- include list a walked file's, the cabals in strictly ascending (path)
-- order and every owner row a walked file's in that order, owned by a
-- carried cabal, the package.json paths and the chained tsconfig paths
-- in strictly ascending order, every TS fact a question of its kind
-- asked once with an answer in its range (a text with a text answer
-- only) — and the cap the request's text counts against. The
-- first offender is refused by name ("<table> <i>: <why>"); a request
-- that passes is the one CE.Resolve.World indexes without a check of its
-- own.
module CE.Resolve.Contract (offence, overCap, requestSize) where

import CE.Resolve.Cost
import CE.Resolve.JavaHeader
import CE.Resolve.Request
import Data.Aeson (Value, encode)
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | One per row plus every character of every string, the decoded
-- documents at their encoded length.
requestSize :: ResolveReq -> Integer
requestSize rq =
  sum
    [ texts (rqFiles rq)
    , texts (rqOrigins rq)
    , sum [1 + chars (sSpec s) | s <- rqSites rq]
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
    , sum [1 + chars a + chars b + maybe 0 chars t | (_, a, b, _, t) <- rqTsFacts rq]
    ]
 where
  c = rqC rq
  chars = toInteger . length
  texts = sum . map ((+ 1) . chars)
  doc :: Value -> Integer
  doc = toInteger . BL.length . encode
  header h = chars (hPackage h) + sum [1 + chars (iName i) | i <- hImports h] + sum (map typeSize (hTypes h))
  typeSize t = 1 + chars (tName t) + texts (tSupers t) + sum (map typeSize (tMembers t))

overCap :: ResolveReq -> Bool
overCap rq = requestSize rq > resolveCap

-- | The first offender, table by table.
offence :: ResolveReq -> Maybe String
offence rq =
  asum
    [ ascending "file" (rqFiles rq)
    , ascending "origin" (rqOrigins rq)
    , asum [Just ("origin " <> show i <> ": a walked file") | (i, o) <- zip [0 :: Int ..] (rqOrigins rq), Set.member o walked]
    , asum (zipWith site [0 :: Int ..] (rqSites rq))
    , ascending "java.header" (map fst (rqJavaHeaders rq))
    , asum [Just ("c.db " <> show i <> ": probe out of range") | (i, Db _ p _) <- zip [0 :: Int ..] (cDbs (rqC rq)), p < 0 || p > 2]
    , asum [Just ("c.include " <> show i <> ": not a walked file") | (i, (p, _)) <- zip [0 :: Int ..] (cIncludes (rqC rq)), not (Set.member p walked)]
    , ascending "hs.cabal" (map fst (rqHsCabals rq))
    , ascending "hs.owner" (map fst (rqHsOwners rq))
    , asum [Just ("hs.owner " <> show i <> ": its file is not walked or its cabal not carried") | (i, (f, c)) <- zip [0 :: Int ..] (rqHsOwners rq), not (Set.member f walked && Set.member c cabals)]
    , ascending "ts.package" (rqTsPackages rq)
    , ascending "ts.chain" (rqTsChains rq)
    , asum (zipWith tsFact [0 :: Int ..] (rqTsFacts rq))
    , asum [Just ("ts.fact " <> show i <> ": asked twice") | (i, (seen, k)) <- zip [0 :: Int ..] (zip (scanl (flip Set.insert) Set.empty keys) keys), Set.member k seen]
    ]
 where
  walked = Set.fromList (rqFiles rq)
  cabals = Set.fromList (map fst (rqHsCabals rq))
  keys = [(op, a, b) | (op, a, b, _, _) <- rqTsFacts rq]
  tsFact i (op, _, b, st, t)
    | op < 0 || op > 2 || (op /= 2 && b /= "") = Just ("ts.fact " <> show i <> ": no such question")
    | st < 0 || st > (if op == 0 then 2 else 1) || (st == 2 && op == 0) /= (t /= Nothing) = Just ("ts.fact " <> show i <> ": answer out of range")
    | otherwise = Nothing
  paths = toInteger (length (rqFiles rq) + length (rqOrigins rq))
  site i s
    | sLang s `notElem` resolvedLangs = Just ("site " <> show i <> ": language this family does not resolve")
    | sKind s < 0 || toInteger (sFrom s) < 0 || toInteger (sFrom s) >= paths = Just ("site " <> show i <> ": kind or file out of range")
    | sLang s == langJava && sLine s == Nothing = Just ("site " <> show i <> ": a Java site without its line")
    | otherwise = Nothing

-- | A path table in strictly ascending order: no path twice.
ascending :: String -> [String] -> Maybe String
ascending name xs = asum [Just (name <> " " <> show i <> ": not after the path before it") | (i, (a, b)) <- zip [1 :: Int ..] (zip xs (drop 1 xs)), a >= b]
