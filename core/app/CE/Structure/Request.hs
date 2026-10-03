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
module CE.Structure.Request (StructReq (..), patternsOf, seamTables) where

import CE.Structure.Shape (foldShapes)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM

data StructReq = StructReq
  { reqId :: Value
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
  parseJSON = withObject "StructReq" $ \o ->
    StructReq
      <$> o .: "id"
      <*> o .: "nodes"
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

-- | The [dirId, code, count] distribution the axes read: the shape
-- facts folded here; no shape table, no distribution.
patternsOf :: StructReq -> [[Integer]]
patternsOf req = maybe [] foldShapes (reqShapes req)

-- | The four unit/edge tables as ONE bundle — the same tuple
-- CE.Structure.Split consumes on both its faces (offence + rows).
seamTables :: StructReq -> ([[Integer]], [[Integer]], [[Integer]], [[Integer]])
seamTables req =
  (reqSeamUnits req, reqSeamRefs req, reqSeamClones req, reqSeamChurn req)
