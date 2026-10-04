{-# LANGUAGE OverloadedStrings #-}

-- | resolve.request handler (plan v2.33 wave W2a; design booklet
-- docs/reference/algorithm-track.md §6 row W2): the reference ladders'
-- search for Python, Lua, Go and C / C++. The measuring side reads the
-- tree, detects the sites, reads the configuration files and lowers
-- every string the search needs to segment ids; this family decides
-- which candidate locations a site tries, in which rung order, what
-- counts as a hit, the ambiguity and refusal rules and the External
-- classification. A reply row per site, `[rung, outcome, target,
-- reason]`, is exactly what the measuring side's outcome carried, so
-- the edge store and every face downstream read the same answers; the
-- C-family's forced includes (`-include x.h`) ride beside them as
-- `[unit, header]` arcs.
module CE.Resolve (respond) where

import CE.Resolve.Answer
import CE.Resolve.C (resolveC)
import CE.Resolve.CIndex (Index, forcedArcs, index)
import CE.Resolve.Contract (offence, overCap)
import CE.Resolve.Cost
import CE.Resolve.Go (Mod, mods, resolveGo)
import CE.Resolve.Lua (resolveLua)
import CE.Resolve.Py (resolvePy)
import CE.Resolve.Request
import CE.Resolve.World (World, world)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import Data.Bits ((.&.))
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | decode → cap (every table together) → contract → judge.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "resolve" rqId overCap offence (degraded proto) (judged proto)

-- | Each site through its language's ladder, in request order; the
-- forced-include arcs when the request carries a compile database.
judged :: String -> ResolveReq -> B8.ByteString
judged proto rq = reply proto rq (map (answerRow . site w rq ix gomods) (rqSites rq)) arcs False
 where
  w = world rq
  ix = index w (rqC rq)
  gomods = mods (rqGo rq)
  arcs = [[toInteger u, toInteger h] | (u, h) <- forcedArcs w ix]

-- | One site: its language's ladder over its tokens.
site :: World -> ResolveReq -> Index -> [Mod] -> [Integer] -> Answer
site w rq ix gomods row = case row of
  lang : kind : from : form : toks
    | lang == langPy -> resolvePy w (rqPy rq) (int from) (int form) (dotted toks)
    | lang == langLua -> resolveLua w (rqLua rq) kind (int from) form toks
    | lang == langGo -> resolveGo w gomods (int from) form (map int toks)
    | otherwise -> resolveC w ix (int from) (form .&. formSystem /= 0) (map int toks)
  _ -> AUnresolved Unsupported -- unreachable behind the contract
 where
  int = fromInteger

-- | A Python spec's dotted segments, each its slash pieces.
dotted :: [Integer] -> [[Int]]
dotted [] = []
dotted toks = case break (== sepDot) toks of
  (a, _ : rest) -> map fromInteger a : dotted rest
  (a, []) -> [map fromInteger a]

-- | Over-cap: a complete degraded reply with empty tables.
degraded :: String -> ResolveReq -> B8.ByteString
degraded proto rq = reply proto rq [] [] True

-- | The resolve.result object.
reply :: String -> ResolveReq -> [[Integer]] -> [[Integer]] -> Bool -> B8.ByteString
reply proto rq results arcs isDegraded =
  BL.toStrict . encode . object $
    -- in the order the encoder writes them (aeson's key map is sorted)
    [ "counts" .= counts
    , "degraded" .= isDegraded
    , "forced" .= arcs
    , "id" .= rqId rq
    , "proto" .= proto
    ]
      <> ["reason" .= ("resolve_too_large" :: String) | isDegraded]
      <> ["results" .= results, "type" .= ("resolve.result" :: String)]
 where
  counts =
    object
      [ "files" .= length (rqFiles rq)
      , "resolved" .= length [() | (_ : o : _) <- results, o /= outUnresolved]
      , "sites" .= length (rqSites rq)
      ]
