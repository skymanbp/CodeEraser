-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The runtime-generated probe batteries that lived in Spec.hs until
-- the thirteenth golden file and its two batteries would have pushed
-- Main past the core's 290-line wall (plan v2.31 step 1; the
-- CE.Scan.Contract precedent): the envelope's structural checks, the
-- ledger-clearance refusal probes, the docdup cap posture and the
-- derived-floor cost model. Spec.hs keeps the goldenPairs list, which
-- cli/tests/it/fixture_contract.rs reads as the ledger of golden
-- files, and calls these four by name.
module SpecProbes (costModel, docdupStructural, refusalProbes, structural) where

import CE.Docdup.Cost (docPairCap, docSetCap)
import CE.FourClass.Cost (anchorFloor, destFloor, siteOpens)
import CE.Graph.Cost (edgeCap, nodeCap)
import qualified CE.Protocol as Protocol
import CE.Verdict.Ratchet (ratchetBound, tolerated)
import Data.Aeson (Value (..), decodeStrict)
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import Data.Version (showVersion)
import Paths_ce_core (version)
import WireHarness (runChecks)

-- | The version respond echoes — the cabal-generated single source.
coreVersion :: String
coreVersion = showVersion version

-- | One named check through the shared runner.
check :: String -> Bool -> IO Bool
check name ok = runChecks [(name, ok)]

-- | Runtime-generated docdup cap probes (the graph over-cap posture:
-- an 8k-element set has no business weighing down a fixture file).
-- BOTH halves of the cap guard fire — a compensating guard with a
-- dead half was the Graph edgeCap defect the first draft shipped.
docdupStructural :: IO Bool
docdupStructural = do
  a <- check "over-cap docdup SET degrades" (field setCapReply "reason" == Just "docdup_too_large")
  b <- check "over-cap docdup PAIRS degrade" (field pairCapReply "reason" == Just "docdup_too_large")
  c <-
    check
      "degraded docdup reply keeps type and id"
      (field setCapReply "type" == Just "docdup.result" && field setCapReply "id" == Just (Number 9))
  pure (a && b && c)
 where
  ints ns = B8.intercalate "," (map (B8.pack . show) ns)
  setCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"7.0.0\",\"type\":\"docdup.request\",\"id\":9,\"sets\":[["
        <> ints [0 .. docSetCap]
        <> "]],\"pairs\":[]}"
  -- cap check precedes validation by design, so identical pair rows
  -- are fine here (never validated)
  pairCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"7.0.0\",\"type\":\"docdup.request\",\"id\":10,\"sets\":[[1,2]],\"pairs\":["
        <> B8.intercalate "," (replicate (fromInteger docPairCap + 1) "[0,0,0]")
        <> "]}"

-- | The floor is derived, and perturbing the site cost must move it
-- (plan §7.4 sensitivity: a dead knob cannot hide).
costModel :: IO Bool
costModel = do
  a <- check "cross floor derives to 2" (destFloor == 2)
  b <- check "single cross line is a tie, does not open" (not (siteOpens 2 1))
  c <-
    check
      "floor tracks the site cost (ablation table)"
      ([minimum [n | n <- [1 .. 9], siteOpens s n] | s <- [0, 2, 4, 6]] == [1, 2, 3, 4])
  -- Decided, not derived: the top of the measured safe window
  -- (Cost.anchorFloor's why-comment carries the ablation evidence).
  d <- check "anchor floor pinned to the decided window top" (anchorFloor == 19)
  -- The ADR-006 tolerance legs cross at ceiling 500: below it the
  -- +10 leg wins, above it the 2% leg — one assertion per branch
  -- (plan §7.1), so neither leg can silently die.
  e <- check "tolerance below the crossover rides +10" (tolerated ratchetBound Nothing 100 == 110)
  f <- check "tolerance above the crossover rides +2%" (tolerated ratchetBound Nothing 1000 == 1020)
  -- a class allowance REPLACES both legs (5.1.0), so zero is zero at
  -- a ceiling where the +10 leg would otherwise have paid out
  g <- check "a class allowance replaces both legs" (tolerated ratchetBound (Just 0) 100 == 100)
  pure (a && b && c && d && e && f && g)

-- | Field-level assertions that do not depend on fixture bytes.
-- The oversize and over-cap probes are generated at run time — a
-- 32 MiB line or a 131k-node request has no business weighing down
-- a committed fixture file.
structural :: IO Bool
structural = do
  a <- check "unknown type echoes id" (field unknownReply "id" == Just (Number 7))
  b <- check "unknown type is error/unknown_type" (field unknownReply "code" == Just "unknown_type")
  c <- check "oversize line is error/too_large" (field oversizeReply "code" == Just "too_large")
  d <- check "major mismatch is rejected" (field majorReply "accept" == Just (Bool False))
  e <- check "over-cap graph degrades visibly" (field overCapReply "reason" == Just "graph_too_large")
  f <- check "over-cap graph is a degraded result" (field overCapReply "degraded" == Just (Bool True))
  -- the degraded result is the ONLY success shape the 2a stub emits;
  -- a wrong type or dropped id is a client-side desync
  -- (corelink.rs), so the whole shape is pinned (Opus review)
  g <-
    check
      "over-cap reply keeps type and id"
      (field overCapReply "type" == Just "graph.result" && field overCapReply "id" == Just (Number 3))
  h <-
    check
      "over-cap counts echo input, kept 0"
      ( subfield overCapReply "counts" "nodes" == Just (Number (fromInteger nodeCap + 1))
          && subfield overCapReply "counts" "kept" == Just (Number 0)
      )
  -- both halves of the compensating guard fire — edgeCap was a dead
  -- knob in the first draft (Opus review)
  i <- check "over-cap EDGES degrade too" (field edgeCapReply "reason" == Just "graph_too_large")
  pure (and [a, b, c, d, e, f, g, h, i])
 where
  unknownReply = Protocol.respond coreVersion "{\"proto\":\"7.0.0\",\"type\":\"mystery\",\"id\":7}"
  oversizeReply = Protocol.respond coreVersion (B8.replicate 33554433 'x')
  majorReply = Protocol.respond coreVersion "{\"proto\":\"9.0.0\",\"type\":\"hello\"}"
  overCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"7.0.0\",\"type\":\"graph.request\",\"id\":3,\"nodes\":["
        <> B8.intercalate "," (replicate (fromInteger nodeCap + 1) "[0,0,0]")
        <> "],\"edges\":[],\"pos\":[]}"
  -- cap check precedes validation by design, so identical edge rows
  -- are fine here (never validated)
  edgeCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"7.0.0\",\"type\":\"graph.request\",\"id\":4,\"nodes\":[[0,0,0]],\"edges\":["
        <> B8.intercalate "," (replicate (fromInteger edgeCap + 1) "[0,0,0,0]")
        <> "],\"pos\":[]}"

-- | The ledger-clearance refusal probes, split from structural at
-- the E01 50-line function cap and TABLE-driven (the dedup ratchet
-- caught the check-ladder shape cloning structural's): a typo'd
-- envelope still echoes its id (without it the client reads a shape
-- mistake as L2-down, VERSIONING.md §1), the two new boundary rows
-- refuse by name, and the exception barrier's error code is pinned
-- to the §1 enum — the clearance review caught `internal` shipping
-- outside the booklet's closed set.
refusalProbes :: IO Bool
refusalProbes = do
  results <-
    mapM
      probe
      [ ("envelope decode failure echoes a present id", badEnvReply, "id", Number 42)
      , ("graph pos must ascend", dupPosReply, "message", String "pos 1: not strictly ascending")
      , ("duplicate fourclass pair index refused", dupPairReply, "message", String "duplicate pair index: 3")
      , ("barrier reply carries code internal", Protocol.internalError "boom", "code", String "internal")
      , ("barrier id stays null", Protocol.internalError "boom", "id", Null)
      , ("a clone leaf column of the wrong length refused (7.5.0)", cloneReply "[11]", "message", String "tree 0: leaf length mismatch")
      ]
  -- the leaf column is merge/1's: clone/1 judges the same bytes with it
  same <- check "a clone request with a leaf column answers the bytes it answers without" (cloneReply "[11,0]" == cloneReply "")
  pure (and results && same)
 where
  probe (name, bytes, key, want) = check name (field bytes key == Just want)
  badEnvReply = Protocol.respond coreVersion "{\"proto\":\"7.0.0\",\"id\":42}"
  -- two two-node trees, one pair; a `leaf` of "" leaves the key out
  cloneReply leaf =
    Protocol.respond coreVersion $
      "{\"proto\":\"7.0.0\",\"type\":\"clone.request\",\"id\":8,\"trees\":[{\"lab\":[1,9],\"lld\":[0,0]"
        <> (if B8.null leaf then "" else ",\"leaf\":" <> leaf)
        <> "},{\"lab\":[1,9],\"lld\":[0,0]}],\"pairs\":[[0,1]]}"
  dupPosReply =
    Protocol.respond
      coreVersion
      "{\"proto\":\"7.0.0\",\"type\":\"graph.request\",\"id\":5,\"nodes\":[[0,0,0],[0,0,0]],\"edges\":[],\"pos\":[1,1]}"
  dupPairReply =
    Protocol.respond
      coreVersion
      "{\"proto\":\"7.0.0\",\"type\":\"fourclass.request\",\"id\":6,\"pairs\":[{\"i\":3,\"rem\":[],\"add\":[]},{\"i\":3,\"rem\":[],\"add\":[]}]}"


field :: B8.ByteString -> String -> Maybe Value
field bytes key = do
  Object o <- decodeStrict bytes
  KM.lookup (Key.fromString key) o

subfield :: B8.ByteString -> String -> String -> Maybe Value
subfield bytes key sub = do
  Object o <- field bytes key
  KM.lookup (Key.fromString sub) o
