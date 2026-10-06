{-# LANGUAGE OverloadedStrings #-}

-- | resolve.request handler (plan v2.33 wave W2a; on text since
-- W2-text, proto 9.0.0; design booklet docs/reference/algorithm-track.md
-- §3, §6): the reference ladders for Python, TypeScript / TSX, Rust, Lua,
-- Go, C / C++, R, Java, Haskell and Markdown, and the configuration
-- readers they
-- read — the tsconfig chains and package.json files, the Cargo.toml
-- files, go.mod, R's DESCRIPTION, the .cabal files, the
-- root `pyproject.toml`'s keys, the compile databases with their
-- response files and flag files — and every walked Java file's header as
-- the walk read it. The measuring side walks the tree, detects the
-- sites, reads the files and sends what it read as text; this family
-- decides which candidate locations a site tries, in which rung order,
-- what counts as a hit, the ambiguity and refusal rules and the
-- External classification. A reply row per site, `[rung, outcome,
-- target, reason]`, is exactly what the measuring side's outcome
-- carried, a Markdown section's slug (null: an anchor not confirmed)
-- beside it in `sections` as `[row, slug]`; the C family's forced
-- includes (`-include x.h`) ride beside them as `[unit, header]` path
-- pairs. A response file the expansion
-- names and the request did not carry is `wanted` (the measuring side
-- reads it and asks again; the results wait for it), and so is a
-- file-system or syntax-tree fact a TS or Rust rung needs and the request
-- lacks, under `tsWanted` (CE.Resolve.TsFacts); `tsReached` names, per
-- tsconfig of
-- `ts.chains`, every config its extends chain reaches — the resolve
-- key's input; `responses` names,
-- per JSON database, every response path its expansion read — the
-- resolve key's input; `packages` names each carried DESCRIPTION's
-- package directory with its code (`[dir, [file]]`, the declared targets
-- an R package node reaches); `mains` names the carried cabals' declared
-- executable and test mains that are walked, `crates` the walked crate
-- roots of the Cargo.toml files `rs.manifests` names, and `private` the
-- files of `hs.owners` their owning cabal keeps private and those of
-- `rs.owners` their owning Cargo package keeps private (the mounts
-- table's bit 1).
module CE.Resolve (respond) where

import CE.Resolve.Answer
import qualified CE.Resolve.Cabal as Cabal
import CE.Resolve.C (resolveC)
import qualified CE.Resolve.Cargo as Cargo
import CE.Resolve.CIndex (Env (..), Found (..), Index, forcedArcs, index)
import CE.Resolve.CompDb (Entry, Expanded (..), parseDb, parseFlags)
import CE.Resolve.Contract (offence, overCap)
import CE.Resolve.Cost
import CE.Resolve.Description (Description (..), packageCode, readDescription)
import CE.Resolve.Go (GoMod (..), parseGoMod, resolveGo)
import CE.Resolve.Hs (resolveHs)
import CE.Resolve.Inspect (inspected)
import CE.Resolve.Java (resolveJava)
import CE.Resolve.JavaPick (javaEnv)
import CE.Resolve.Lua (resolveLua, searched)
import CE.Resolve.Md (MdEnv (..), refTable, resolveMd)
import CE.Resolve.Py (pyproject, resolvePy)
import CE.Resolve.R (resolveR)
import CE.Resolve.Request
import CE.Resolve.Rs (RsEnv (..), ctxFor, needAll, resolveRs)
import CE.Resolve.Str (parentDir)
import CE.Resolve.Ts (TsEnv (..), resolveTs)
import CE.Resolve.TsConfig (package, tsExtendsFiles, tsOptions)
import CE.Resolve.TsFacts (Fact, Facts, Need, facts, tomlOf, wantedRow)
import CE.Resolve.World (World (..), pathOf, world)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M
import Data.Maybe (catMaybes, fromMaybe, isJust)
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
  (parsed, env, ix) = cHalf w rq
  expanded = mconcat (map snd (M.elems parsed))
  py = pyproject (rqPyproject rq)
  luaDirs = searched (searchRoots "lua" rq) (rqLuaTemplates rq)
  gomods = [m | (rel, text) <- rqGoMods rq, let m = parseGoMod rel text, isJust (gmModule m)]
  descs = [d | (rel, text) <- rqDescriptions rq, Just d <- [readDescription (parentDir rel) text]]
  packages = M.fromList [(dDir d, packageCode w d) | d <- descs]
  java = javaEnv w (rqJavaHeaders rq) (searchRoots "java" rq)
  cabals = M.fromList [(rel, Cabal.parse rel text) | (rel, text) <- rqHsCabals rq]
  md = MdEnv w (Set.fromList (rqAssets rq)) (M.fromList (rqMdSlugs rq)) (M.fromList [(f, refTable ds us) | (f, ds, us) <- rqMdRefs rq])
  site s
    | sLang s == langPy = resolvePy w py from (sSpec s)
    | sLang s == langLua = resolveLua w luaDirs (sKind s) from (sSpec s)
    | sLang s == langGo = resolveGo w gomods from (sSpec s)
    | sLang s == langR = resolveR (w, searchRoots "r" rq, descs) (sKind s) from (sSpec s)
    | sLang s == langJava = resolveJava java (sKind s) from (fromMaybe 0 (sLine s)) (sSpec s)
    | sLang s == langHs = resolveHs w (M.elems cabals) from (sSpec s)
    | sLang s == langMd = resolveMd md (sKind s) from (sSpec s)
    -- complete: no TS site waits for a fact
    | isTs s = either (const (AUnresolved OutOfScope)) id (ts s)
    | sLang s == langRs = either (const (AUnresolved OutOfScope)) id (rs s)
    | otherwise = resolveC env ix from (sSpec s)
   where
    from = pathOf w (sFrom s)
  fx = facts False (rqTsFacts rq)
  (ts, reached, tsWaits) = tsHalf w fx rq
  (rs, crates, rsPrivate, rsWaits) = rsHalf w fx rq
  tsWanted = tsWaits <> rsWaits
  wanted = xWanted expanded
  complete = Set.null wanted && Set.null tsWanted
  answers = if complete then map site (rqSites rq) else []
  resolved = length [() | a <- answers, case a of AUnresolved _ -> False; _ -> True]
  body =
    [ "results" .= map answerRow answers
    , "sections" .= [(i, slug) | (i, ASection _ slug _) <- zip [0 :: Int ..] answers]
    , "forced" .= (if complete then map (\(u, h) -> [u, h]) (forcedArcs env ix) else [])
    , "wanted" .= Set.toList wanted
    , "tsWanted" .= map wantedRow (Set.toList tsWanted)
    , "tsReached" .= [(start, files) | (start, Right files) <- reached]
    , "responses" .= [(rel, Set.toList (xResponses x)) | (rel, (_, x)) <- M.toList parsed]
    , "packages" .= M.toList packages
    , "mains" .= Set.toList (Set.unions (map (Cabal.mainTargets w) (M.elems cabals)))
    , "crates" .= Set.toList crates
    , "private" .= Set.toList (Set.fromList [f | (f, owner) <- rqHsOwners rq, maybe False (`Cabal.keepsPrivate` f) (M.lookup owner cabals)] <> rsPrivate)
    ]
      <> ["inspected" .= v | Just v <- [inspected <$> rqInspect rq]]

-- | The C family's half of a request: every JSON database parsed (its
-- entries, the response files it read and wants), the search
-- environment and its index.
cHalf :: World -> ResolveReq -> (M.Map String ([Entry], Expanded), Env, Index)
cHalf w rq = (parsed, env, index env founds)
 where
  c = rqC rq
  base = cRoot c
  texts = M.fromList (cResponses c)
  parsed = M.fromList [(rel, maybe ([], mempty) (parseDb base texts) rows) | (rel, rows) <- cJson c]
  flagText = M.fromList (cFlags c)
  founds =
    [ if probe == 2
        then FFlags dir (snd . parseFlags base rel <$> (M.lookup rel flagText >>= id))
        else FJson dir rel (maybe [] fst (M.lookup rel parsed))
    | Db dir probe rel <- cDbs c
    ]
  env = Env w (searchRoots "c" rq) (M.fromList (cIncludes c)) base

isTs :: Site -> Bool
isTs s = sLang s == langTs || sLang s == langTsx

-- | The TS half of a request: each TS site's answer (Left: the facts it
-- waits for), every `ts.chains` config's reached files, and the facts the
-- sites and the chains want together. A directory's chain is read once.
tsHalf :: World -> Facts -> ResolveReq -> (Site -> Need Answer, [(String, Need [String])], Set.Set Fact)
tsHalf w fx rq = (ts, reached, Set.unions ([m | s <- rqSites rq, isTs s, Left m <- [ts s]] <> [m | (_, Left m) <- reached]))
 where
  chains = M.fromList [(d, tsOptions fx d) | s <- rqSites rq, isTs s, let d = parentDir (pathOf w (sFrom s))]
  tsEnv = TsEnv w fx (\d -> M.findWithDefault (tsOptions fx d) d chains) (catMaybes <$> mapM (package fx) (rqTsPackages rq))
  ts s = resolveTs tsEnv (pathOf w (sFrom s)) (sSpec s)
  reached = [(start, tsExtendsFiles fx start) | start <- rqTsChains rq]

-- | The Rust half of a request: each Rust site's answer (Left: the facts
-- it waits for), the walked crate roots of `rs.manifests`, the files of
-- `rs.owners` their Cargo package keeps private, and the facts all of
-- them want together. A directory's package context is read once, the
-- walk's Cargo.toml files once.
rsHalf :: World -> Facts -> ResolveReq -> (Site -> Need Answer, Set.Set String, Set.Set String, Set.Set Fact)
rsHalf w fx rq = (rs, done crates, done kept, Set.unions (waits crates : waits kept : [m | s <- rqSites rq, sLang s == langRs, Left m <- [rs s]]))
 where
  crates = Set.unions <$> needAll [maybe Set.empty (Cargo.crateRoots (wFiles w) . Cargo.package m) <$> tomlOf fx m | m <- rqRsManifests rq]
  kept = Set.fromList . map fst . filter snd <$> needAll [(\doc -> (f, Cargo.keeps (wFiles w) (Cargo.package m <$> doc) f)) <$> tomlOf fx m | (f, m) <- rqRsOwners rq]
  done = either (const Set.empty) id
  waits = either id (const Set.empty)
  declared = Set.fromList (rqRsCrateRoots rq)
  ctx = ctxFor fx w declared
  ctxs = M.fromList [(d, ctx d) | s <- rqSites rq, sLang s == langRs, let d = parentDir (pathOf w (sFrom s))]
  members = catMaybes <$> needAll [fmap (Cargo.package c) <$> tomlOf fx c | c <- rqRsPackages rq]
  env = RsEnv w fx (\d -> M.findWithDefault (ctx d) d ctxs) members
  rs s = resolveRs env (sKind s) (pathOf w (sFrom s)) (maybe 0 (max 0 . subtract 1) (sLine s)) (sSpec s)

-- | Over-cap: a complete degraded reply with empty tables.
degraded :: String -> ResolveReq -> B8.ByteString
degraded proto rq = reply proto rq 0 [k .= ([] :: [Value]) | k <- ["results", "sections", "forced", "wanted", "tsWanted", "tsReached", "responses", "packages", "mains", "crates", "private"]] True

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
