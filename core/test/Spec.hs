-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Base-only deterministic checks for the ce-core wire layer.
-- The golden fixtures under contracts/fixtures/ are shared with the
-- Rust side (cli/tests/it/core_wire.rs): byte drift on either side
-- reddens both suites.
module Main (main) where

import qualified AdvisoryProps
import qualified ArchProps
import qualified AuditProps
import qualified DeclProps
import qualified TombstoneProps
import qualified SimilarProps
import qualified ClassProps
import qualified CloneProps
import qualified EntropyProps
import qualified EraseProps
import qualified FlowProps
import qualified GraphWireProps
import qualified GraphProps
import qualified JoinProps
import qualified MergeProps
import qualified QueryProps
import qualified Reference
import qualified ReferenceGraph
import qualified ReferenceJaccard
import qualified ReferenceQuery
import qualified ScanCyclesProps
import qualified ScanEventsProps
import qualified ScanProps
import qualified StructureModularityProps
import qualified StructureProps
import qualified TrendProps
import qualified VerdictProps
import qualified SpecProbes
import qualified SplitProps
import qualified StackingProps
import qualified VerdictKnobProps
import qualified VerdictWireProps
import qualified VerdictFenceProps
import qualified CE.Protocol as Protocol
import Control.Monad (unless)
import qualified Data.ByteString.Char8 as B8
import Data.Version (showVersion)
import Paths_ce_core (version)
import System.Exit (exitFailure)
import System.IO (hSetEncoding, stderr, stdout, utf8)
import WireHarness (runChecks)

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
  results <- sequence batteries
  unless (and results) exitFailure

batteries :: [IO Bool]
batteries =
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
  , goldenPairs "tombstone/golden.ndjson"
  , goldenPairs "similar/golden.ndjson"
  , goldenPairs "query/golden.ndjson"
  , goldenPairs "flow/golden.ndjson"
  , goldenPairs "merge/golden.ndjson"
  , goldenPairs "arch/golden.ndjson"
  , SpecProbes.structural
  , SpecProbes.refusalProbes
  , SpecProbes.docdupStructural
  , SpecProbes.costModel
  , Reference.equivalence
  , ReferenceQuery.equivalence
  , ReferenceGraph.equivalence
  , ReferenceJaccard.equivalence
  , GraphProps.battery
  , GraphWireProps.battery
  , AdvisoryProps.battery
  , CloneProps.battery
  , EntropyProps.battery
  , JoinProps.battery
  , ScanProps.battery
  , ScanCyclesProps.battery
  , ScanEventsProps.battery
  , StructureProps.battery
  , StructureModularityProps.battery
  , TrendProps.battery
  , EraseProps.battery
  , AuditProps.battery
  , TombstoneProps.battery
  , SimilarProps.battery
  , QueryProps.battery
  , FlowProps.battery
  , MergeProps.battery
  , ArchProps.battery
  , VerdictProps.battery
  , VerdictWireProps.battery
  , VerdictFenceProps.battery
  , VerdictKnobProps.battery
  , SplitProps.battery
  , ClassProps.battery
  , StackingProps.battery
  , DeclProps.battery
  ]

-- | One named check through the shared runner.
check :: String -> Bool -> IO Bool
check name ok = runChecks [(name, ok)]

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
