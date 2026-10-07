-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The structure.request record and its decoder — the request's
-- spelling, split from CE.Structure when the name-pattern shape road
-- (7.2.0, plan v2.30 step 7b) pushed that module past the E01
-- 300-line line (the CE.Scan.Contract precedent: what arrived, what
-- is checked and what is judged are three jobs). Every optional
-- table is a Maybe here, never defaulted: an absent table means its
-- axis is not judged, an empty one that it judged clean — the
-- churn-table honesty (absence is spoken, never zero-filled).
--
-- Since 9.0.0 (plan v2.33 W1 item 3, additive in the unreleased minor) a
-- request may carry the PATH form instead of the dir-keyed tables
-- (CE.Structure.Raw): the decoder builds the tree and fills the same
-- fields from it, so every check and the judgment below read one shape.
-- `reqPath` keeps what the path form needs past the judgment: the tree
-- the reply echoes, the fault that replaces a judgment, the offence that
-- refuses it.
module CE.Structure.Request (StructReq (..), PathForm (..), patternsOf, seamTables, pathOffence, pathFault, pathKeys) where

import CE.Structure.Raw (Built (..), Raw, buildRaw, parseRaw, rawOffence)
import CE.Structure.Shape (foldShapes)
import CE.Structure.Tree (dirPaths)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (Pair, Parser)

-- | The path form as decoded: the tables built (or the fault naming a
-- path the tree cannot place), the offence refusing it, whether the
-- built tables ride back (`inspect`).
data PathForm = PathForm
  { pfBuilt :: Either String Built
  , pfOffence :: Maybe String
  , pfInspect :: Bool
  }

data StructReq = StructReq
  { reqPath :: Maybe PathForm
  , reqId :: Value
  , reqNodes :: [[Integer]]
  , -- the name-pattern distribution: `patternShapes` carries
    -- [dirId, bits, count] stem facts the core classifies
    -- (CE.Structure.Shape). The pre-7.2.0 road, `patterns` — codes the
    -- producer classified for itself — retired at 8.0.0 (plan v2.32
    -- step 6): this only records that a request still carried it, and
    -- the boundary contract refuses it by name.
    reqPatternsSent :: Bool
  , reqShapes :: Maybe [[Integer]]
  , reqConventions :: [[Integer]]
  , reqFileRefs :: [[Integer]]
  , reqDeclared :: [[Integer]]
  , -- the RAW staleness facts (2.23.0, additive; the pre-judged
    -- staleDocs table's one-minor grace expired and its arm retired
    -- at 2.29.0 — a legacy key is ignored per the §1 unknown-field
    -- rule): one row per md
    -- doc that HAS reference targets — [dirId, docTs], docTs = the
    -- doc's newest change inside the churn window, 0 = unchanged
    -- (the one sentinel, documented); doc identity = row index
    -- (dense by construction, the graph node discipline). Edges
    -- [docIdx, targetTs] exist only for targets that CHANGED in the
    -- window (targetTs >= 1) — an unchanged target can never make a
    -- doc stale, so shipping it would be dead weight.
    reqStaleDocRows :: Maybe [[Integer]]
  , reqStaleEdges :: [[Integer]]
  , reqRedundancy :: Maybe [[Integer]]
  , reqDirEdges :: Maybe [[Integer]]
  -- ^ 7.1.0 (O54): the directed CROSSING dir-edge table, same Maybe
  -- stance. The intra mass is fileRefs' `inside` sum halved and never
  -- rides twice; Mod.crossTableOffence holds the pair to one graph.
  , -- the split-ROI advisory tables (plan v2.6 §C 2.14.0, clones/
    -- churn v2.7 ② 2.15.0 — all additive): seamFiles is the
    -- presence anchor — the two reply keys exist exactly when it
    -- rides; the four unit/edge tables default empty
    reqSeamFiles :: Maybe [[Integer]]
  , reqSeamUnits :: [[Integer]]
  , reqSeamRefs :: [[Integer]]
  , reqSeamClones :: [[Integer]]
  , reqSeamChurn :: [[Integer]]
  , reqKnobs :: [[Integer]]
  }

instance FromJSON StructReq where
  parseJSON = withObject "StructReq" $ \o -> do
    raw <- parseRaw o
    path <- traverse (pathForm o) raw
    req <- integerForm o (maybe (o .: "nodes") (const (pure [])) raw)
    pure (fill req {reqPath = path})

-- | The integer form, `nodes` read by the caller (required unless the
-- path form rides).
integerForm :: Object -> Parser [[Integer]] -> Parser StructReq
integerForm o nodes =
    StructReq Nothing
      <$> o .: "id"
      <*> nodes
      <*> pure (KM.member "patterns" o)
      <*> o .:? "patternShapes"
      <*> o .:? "conventions" .!= []
      <*> o .:? "fileRefs" .!= []
      <*> o .:? "declared" .!= []
      <*> o .:? "staleDocRows"
      <*> o .:? "staleEdgeRows" .!= []
      <*> o .:? "redundancy"
      <*> o .:? "dirEdges"
      <*> o .:? "seamFiles"
      <*> o .:? "seamUnits" .!= []
      <*> o .:? "seamRefs" .!= []
      <*> o .:? "seamClones" .!= []
      <*> o .:? "seamChurn" .!= []
      <*> o .:? "knobs" .!= []

-- | The path form decoded: refused when it rides beside a dir-keyed
-- table or its own contract fails (then nothing is built), else built.
pathForm :: Object -> Raw -> Parser PathForm
pathForm o raw = do
  inspect <- o .:? "inspect" .!= False
  let mixed = [k | k <- integerKeys, KM.member k o]
      offence = case mixed of
        k : _ -> Just ("paths beside the dir-keyed table " <> show k)
        [] -> rawOffence raw
  pure (PathForm (maybe (buildRaw raw) Left offence) offence inspect)
 where
  integerKeys = ["nodes", "patternShapes", "conventions", "fileRefs", "declared", "staleDocRows", "staleEdgeRows", "redundancy", "dirEdges"]

-- | The dir-keyed fields filled from the built path form.
fill :: StructReq -> StructReq
fill req = case reqPath req of
  Just (PathForm (Right b) Nothing _) ->
    req
      { reqNodes = bNodes b
      , reqShapes = Just (bShapes b)
      , reqConventions = bConventions b
      , reqFileRefs = fst (bRefs b)
      , reqDeclared = bDeclared b
      , reqStaleDocRows = fst <$> bStale b
      , reqStaleEdges = maybe [] snd (bStale b)
      , reqRedundancy = bRedundancy b
      , reqDirEdges = Just (snd (bRefs b))
      }
  _ -> req

-- | The path form's refusal, when it rides and is refused.
pathOffence :: StructReq -> Maybe String
pathOffence req = reqPath req >>= pfOffence

-- | The path the tree could not place, answered instead of a judgment.
pathFault :: StructReq -> Maybe String
pathFault req = case reqPath req of
  Just (PathForm (Left fault) Nothing _) -> Just fault
  _ -> Nothing

-- | What a path-form reply adds: the tree rows and each directory's name
-- (the root `.`), the rows a degraded request was priced at, and — asked
-- with `inspect` — the dir-keyed tables built.
pathKeys :: Integer -> StructReq -> [Pair]
pathKeys priced req = case reqPath req of
  Just (PathForm (Right b) Nothing inspect) ->
    ["tree" .= bNodes b, "dirs" .= map dotted (dirPaths (bTree b))]
      <> ["priced" .= priced | priced > 0]
      <> ["built" .= object (built b) | inspect]
  _ -> []
 where
  dotted p = if null p then "." else p
  built b =
    [ "patternShapes" .= bShapes b
    , "conventions" .= bConventions b
    , "fileRefs" .= fst (bRefs b)
    , "dirEdges" .= snd (bRefs b)
    , "declared" .= bDeclared b
    , "staleDocRows" .= fmap fst (bStale b)
    , "staleEdgeRows" .= maybe [] snd (bStale b)
    , "redundancy" .= bRedundancy b
    ]

-- | The [dirId, code, count] distribution the axes read: the shape
-- facts folded here; no shape table, no distribution.
patternsOf :: StructReq -> [[Integer]]
patternsOf req = maybe [] foldShapes (reqShapes req)

-- | The four unit/edge tables as ONE bundle — the same tuple
-- CE.Structure.Split consumes on both its faces (offence + rows).
seamTables :: StructReq -> ([[Integer]], [[Integer]], [[Integer]], [[Integer]])
seamTables req =
  (reqSeamUnits req, reqSeamRefs req, reqSeamClones req, reqSeamChurn req)
