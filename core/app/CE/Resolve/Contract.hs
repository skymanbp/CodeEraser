-- | The resolve.request boundary contract (plan v2.33 wave W2a; on text
-- since W2-text): what a well-formed request IS — the walked paths in
-- strictly ascending (path) order, the origins too and none of them
-- walked, every site in a language this family resolves with its file in
-- range, every database probe one of the three, every include list a
-- walked file's — and the cap the request's text counts against. The
-- first offender is refused by name ("<table> <i>: <why>"); a request
-- that passes is the one CE.Resolve.World indexes without a check of its
-- own.
module CE.Resolve.Contract (offence, overCap, requestSize) where

import CE.Resolve.Cost
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
    ]
 where
  c = rqC rq
  chars = toInteger . length
  texts = sum . map ((+ 1) . chars)
  doc :: Value -> Integer
  doc = toInteger . BL.length . encode

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
    , asum [Just ("c.db " <> show i <> ": probe out of range") | (i, Db _ p _) <- zip [0 :: Int ..] (cDbs (rqC rq)), p < 0 || p > 2]
    , asum [Just ("c.include " <> show i <> ": not a walked file") | (i, (p, _)) <- zip [0 :: Int ..] (cIncludes (rqC rq)), not (Set.member p walked)]
    ]
 where
  walked = Set.fromList (rqFiles rq)
  paths = toInteger (length (rqFiles rq) + length (rqOrigins rq))
  site i s
    | sLang s `notElem` resolvedLangs = Just ("site " <> show i <> ": language this family does not resolve")
    | sKind s < 0 || toInteger (sFrom s) < 0 || toInteger (sFrom s) >= paths = Just ("site " <> show i <> ": kind or file out of range")
    | otherwise = Nothing

-- | A path table in strictly ascending order: no path twice.
ascending :: String -> [String] -> Maybe String
ascending name xs = asum [Just (name <> " " <> show i <> ": not after the path before it") | (i, (a, b)) <- zip [1 :: Int ..] (zip xs (drop 1 xs)), a >= b]
