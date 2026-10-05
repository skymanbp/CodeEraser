{-# LANGUAGE OverloadedStrings #-}

-- | resolve.request handler (plan v2.33 wave W2a; on text since
-- W2-text, proto 9.0.0; design booklet docs/reference/algorithm-track.md
-- §3, §6): the reference ladders for Python, Lua, Go, C / C++, R, Java
-- and Haskell, and the configuration readers they read — go.mod, R's
-- DESCRIPTION, the .cabal files, the
-- root `pyproject.toml`'s keys, the compile databases with their
-- response files and flag files — and every walked Java file's header as
-- the walk read it. The measuring side walks the tree, detects the
-- sites, reads the files and sends what it read as text; this family
-- decides which candidate locations a site tries, in which rung order,
-- what counts as a hit, the ambiguity and refusal rules and the
-- External classification. A reply row per site, `[rung, outcome,
-- target, reason]`, is exactly what the measuring side's outcome
-- carried; the C family's forced includes (`-include x.h`) ride beside
-- them as `[unit, header]` path pairs. A response file the expansion
-- names and the request did not carry is `wanted` (the measuring side
-- reads it and asks again; the results wait for it); `responses` names,
-- per JSON database, every response path its expansion read — the
-- resolve key's input; `packages` names each carried DESCRIPTION's
-- package directory with its code (`[dir, [file]]`, the declared targets
-- an R package node reaches); `mains` names the carried cabals' declared
-- executable and test mains that are walked, and `private` the files of
-- `hs.owners` their owning cabal keeps private (the mounts table's bit
-- 1).
module CE.Resolve (respond) where

import CE.Resolve.Answer
import qualified CE.Resolve.Cabal as Cabal
import CE.Resolve.C (resolveC)
import CE.Resolve.CIndex (Env (..), Found (..), forcedArcs, index)
import CE.Resolve.CompDb (Expanded (..), parseDb, parseFlags)
import CE.Resolve.Contract (offence, overCap)
import CE.Resolve.Cost
import CE.Resolve.Description (Description (..), packageCode, readDescription)
import CE.Resolve.Go (GoMod (..), parseGoMod, resolveGo)
import CE.Resolve.Hs (resolveHs)
import CE.Resolve.Inspect (inspected)
import CE.Resolve.Java (resolveJava)
import CE.Resolve.JavaPick (javaEnv)
import CE.Resolve.Lua (resolveLua, searched)
import CE.Resolve.Py (pyproject, resolvePy)
import CE.Resolve.R (resolveR)
import CE.Resolve.Request
import CE.Resolve.Str (parentDir)
import CE.Resolve.World (pathOf, world)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe, isJust)
import qualified Data.Set as Set

-- | decode → cap → contract → judge.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "resolve" rqId overCap offence (degraded proto) (judged proto)

-- | Each site through its language's ladder, in request order; the
-- forced-include arcs; the response files read and wanted.
judged :: String -> ResolveReq -> B8.ByteString
judged proto rq = reply proto rq resolved body False
 where
  w = world rq
  c = rqC rq
  base = cRoot c
  texts = M.fromList (cResponses c)
  parsed = M.fromList [(rel, maybe ([], mempty) (parseDb base texts) rows) | (rel, rows) <- cJson c]
  expanded = mconcat (map snd (M.elems parsed))
  flagText = M.fromList (cFlags c)
  founds =
    [ if probe == 2
        then FFlags dir (snd . parseFlags base rel <$> (M.lookup rel flagText >>= id))
        else FJson dir rel (maybe [] fst (M.lookup rel parsed))
    | Db dir probe rel <- cDbs c
    ]
  env = Env w (searchRoots "c" rq) (M.fromList (cIncludes c)) base
  ix = index env founds
  py = pyproject (rqPyproject rq)
  luaDirs = searched (searchRoots "lua" rq) (rqLuaTemplates rq)
  gomods = [m | (rel, text) <- rqGoMods rq, let m = parseGoMod rel text, isJust (gmModule m)]
  descs = [d | (rel, text) <- rqDescriptions rq, Just d <- [readDescription (parentDir rel) text]]
  packages = M.fromList [(dDir d, packageCode w d) | d <- descs]
  java = javaEnv w (rqJavaHeaders rq) (searchRoots "java" rq)
  cabals = M.fromList [(rel, Cabal.parse rel text) | (rel, text) <- rqHsCabals rq]
  site s
    | sLang s == langPy = resolvePy w py from (sSpec s)
    | sLang s == langLua = resolveLua w luaDirs (sKind s) from (sSpec s)
    | sLang s == langGo = resolveGo w gomods from (sSpec s)
    | sLang s == langR = resolveR (w, searchRoots "r" rq, descs) (sKind s) from (sSpec s)
    | sLang s == langJava = resolveJava java (sKind s) from (fromMaybe 0 (sLine s)) (sSpec s)
    | sLang s == langHs = resolveHs w (M.elems cabals) from (sSpec s)
    | otherwise = resolveC env ix from (sSpec s)
   where
    from = pathOf w (sFrom s)
  wanted = xWanted expanded
  complete = Set.null wanted
  answers = if complete then map site (rqSites rq) else []
  resolved = length [() | a <- answers, not (unresolved a)]
  unresolved a = case a of
    AUnresolved _ -> True
    _ -> False
  body =
    [ "results" .= map answerRow answers
    , "forced" .= (if complete then map (\(u, h) -> [u, h]) (forcedArcs env ix) else [])
    , "wanted" .= Set.toList wanted
    , "responses" .= [(rel, Set.toList (xResponses x)) | (rel, (_, x)) <- M.toList parsed]
    , "packages" .= M.toList packages
    , "mains" .= Set.toList (Set.unions (map (Cabal.mainTargets w) (M.elems cabals)))
    , "private" .= [f | (f, owner) <- rqHsOwners rq, maybe False (`Cabal.keepsPrivate` f) (M.lookup owner cabals)]
    ]
      <> ["inspected" .= v | Just v <- [inspected <$> rqInspect rq]]

-- | Over-cap: a complete degraded reply with empty tables.
degraded :: String -> ResolveReq -> B8.ByteString
degraded proto rq = reply proto rq 0 [k .= ([] :: [Value]) | k <- ["results", "forced", "wanted", "responses", "packages", "mains", "private"]] True

-- | The resolve.result object (aeson writes the keys sorted).
reply :: String -> ResolveReq -> Int -> [Pair] -> Bool -> B8.ByteString
reply proto rq resolved body isDegraded =
  BL.toStrict . encode . object $
    [ "counts" .= counts
    , "degraded" .= isDegraded
    , "id" .= rqId rq
    , "proto" .= proto
    , "type" .= ("resolve.result" :: String)
    ]
      <> body
      <> ["reason" .= ("resolve_too_large" :: String) | isDegraded]
 where
  counts =
    object
      [ "files" .= length (rqFiles rq)
      , "resolved" .= resolved
      , "sites" .= length (rqSites rq)
      ]
