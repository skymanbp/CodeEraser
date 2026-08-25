-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Base-only deterministic checks for the ce-core wire layer.
-- The golden fixtures under contracts/fixtures/ are shared with the
-- Rust side (cli/tests/core_wire.rs): byte drift on either side
-- reddens both suites.
module Main (main) where

import CE.Docdup.Cost (docPairCap, docSetCap)
import CE.FourClass.Cost (anchorFloor, destFloor, siteOpens)
import CE.Graph.Cost (edgeCap, nodeCap)
import CE.Verdict.Ratchet (ratchetBound, tolerated)
import qualified AuditProps
import qualified ClassProps
import qualified CloneProps
import qualified EntropyProps
import qualified EraseProps
import qualified GraphWireProps
import qualified GraphProps
import qualified JoinProps
import qualified Reference
import qualified ReferenceGraph
import qualified ReferenceJaccard
import qualified ScanProps
import qualified StructureProps
import qualified TrendProps
import qualified VerdictProps
import qualified SplitProps
import qualified VerdictKnobProps
import qualified VerdictWireProps
import qualified CE.Protocol as Protocol
import Control.Monad (unless)
import Data.Aeson (Value (..), decodeStrict)
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import Data.Version (showVersion)
import Paths_ce_core (version)
import System.Exit (exitFailure)
import System.IO (hSetEncoding, stderr, stdout, utf8)

-- | cabal test runs with the package root (core/) as cwd.
fixtureDir :: FilePath
fixtureDir = "../contracts/fixtures/"

-- | The version respond echoes — the cabal-generated single source
-- (Main.hs derives the same way; an injected literal here once
-- pinned 0.0.1 into the hello golden while the shipped core moved).
coreVersion :: String
coreVersion = showVersion version

main :: IO ()
main = do
  -- Test names carry Unicode (≥, −); GHC's String output encodes
  -- with the host locale, so a non-UTF-8 Windows codepage crashes
  -- the whole suite mid-run. The harness pins its own encoding.
  hSetEncoding stdout utf8
  hSetEncoding stderr utf8
  results <-
    sequence
      [ goldenPairs "handshake/hello-ok.ndjson"
      , goldenPairs "handshake/wire-errors.ndjson"
      , goldenPairs "fourclass/golden.ndjson"
      , goldenPairs "graph/golden.ndjson"
      , goldenPairs "clone/golden.ndjson"
      , goldenPairs "docdup/golden.ndjson"
      , goldenPairs "verdict/golden.ndjson"
      , goldenPairs "scan/golden.ndjson"
      , goldenPairs "structure/golden.ndjson"
      , goldenPairs "trend/golden.ndjson"
      , goldenPairs "erase/golden.ndjson"
      , goldenPairs "audit/golden.ndjson"
      , structural
      , refusalProbes
      , docdupStructural
      , costModel
      , Reference.equivalence
      , ReferenceGraph.equivalence
      , ReferenceJaccard.equivalence
      , GraphProps.battery
      , GraphWireProps.battery
      , CloneProps.battery
      , EntropyProps.battery
      , JoinProps.battery
      , ScanProps.battery
      , StructureProps.battery
      , TrendProps.battery
      , EraseProps.battery
      , AuditProps.battery
      , VerdictProps.battery
      , VerdictWireProps.battery
      , VerdictKnobProps.battery
      , SplitProps.battery
      , ClassProps.battery
      ]
  unless (and results) exitFailure

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
      "{\"proto\":\"6.0.0\",\"type\":\"docdup.request\",\"id\":9,\"sets\":[["
        <> ints [0 .. docSetCap]
        <> "]],\"pairs\":[]}"
  -- cap check precedes validation by design, so identical pair rows
  -- are fine here (never validated)
  pairCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"6.0.0\",\"type\":\"docdup.request\",\"id\":10,\"sets\":[[1,2]],\"pairs\":["
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

check :: String -> Bool -> IO Bool
check name ok = do
  putStrLn ((if ok then "ok   " else "FAIL ") <> name)
  pure ok

-- | Fixture files alternate request line / expected reply line.
-- Trailing \r is stripped defensively: .gitattributes pins *.ndjson
-- to -text, but a stray CRLF checkout must not turn a byte-golden
-- mismatch into a mystery.
goldenPairs :: FilePath -> IO Bool
goldenPairs file = do
  raw <- B8.readFile (fixtureDir <> file)
  let rows = pairs (filter (not . B8.null) (map stripCR (B8.lines raw)))
  results <- mapM (checkPair file) (zip [1 :: Int ..] rows)
  pure (and results)
 where
  stripCR l = if not (B8.null l) && B8.last l == '\r' then B8.init l else l
  pairs (a : b : rest) = (a, b) : pairs rest
  pairs [] = []
  pairs [_] = error (file <> ": odd line count — fixtures are request/reply pairs")

checkPair :: FilePath -> (Int, (B8.ByteString, B8.ByteString)) -> IO Bool
checkPair file (n, (request, expected)) = do
  let got = Protocol.respond coreVersion request
  ok <- check (file <> " pair " <> show n) (got == expected)
  unless ok $ do
    B8.putStrLn ("  expected: " <> expected)
    B8.putStrLn ("  got:      " <> got)
  pure ok

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
  unknownReply = Protocol.respond coreVersion "{\"proto\":\"6.0.0\",\"type\":\"mystery\",\"id\":7}"
  oversizeReply = Protocol.respond coreVersion (B8.replicate 33554433 'x')
  majorReply = Protocol.respond coreVersion "{\"proto\":\"9.0.0\",\"type\":\"hello\"}"
  overCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"6.0.0\",\"type\":\"graph.request\",\"id\":3,\"nodes\":["
        <> B8.intercalate "," (replicate (fromInteger nodeCap + 1) "[0,0,0]")
        <> "],\"edges\":[],\"pos\":[]}"
  -- cap check precedes validation by design, so identical edge rows
  -- are fine here (never validated)
  edgeCapReply =
    Protocol.respond coreVersion $
      "{\"proto\":\"6.0.0\",\"type\":\"graph.request\",\"id\":4,\"nodes\":[[0,0,0]],\"edges\":["
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
      ]
  pure (and results)
 where
  probe (name, bytes, key, want) = check name (field bytes key == Just want)
  badEnvReply = Protocol.respond coreVersion "{\"proto\":\"6.0.0\",\"id\":42}"
  dupPosReply =
    Protocol.respond
      coreVersion
      "{\"proto\":\"6.0.0\",\"type\":\"graph.request\",\"id\":5,\"nodes\":[[0,0,0],[0,0,0]],\"edges\":[],\"pos\":[1,1]}"
  dupPairReply =
    Protocol.respond
      coreVersion
      "{\"proto\":\"6.0.0\",\"type\":\"fourclass.request\",\"id\":6,\"pairs\":[{\"i\":3,\"rem\":[],\"add\":[]},{\"i\":3,\"rem\":[],\"add\":[]}]}"

field :: B8.ByteString -> String -> Maybe Value
field bytes key = do
  Object o <- decodeStrict bytes
  KM.lookup (Key.fromString key) o

subfield :: B8.ByteString -> String -> String -> Maybe Value
subfield bytes key sub = do
  Object o <- field bytes key
  KM.lookup (Key.fromString sub) o
