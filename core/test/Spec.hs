-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Base-only deterministic checks for the ce-core wire layer.
-- The golden fixtures under contracts/fixtures/ are shared with the
-- Rust side (cli/tests/core_wire.rs): byte drift on either side
-- reddens both suites.
module Main (main) where

import CE.FourClass.Cost (anchorFloor, destFloor, siteOpens)
import CE.Graph.Cost (edgeCap, nodeCap)
import qualified Reference
import qualified CE.Protocol as Protocol
import Control.Monad (unless)
import Data.Aeson (Value (..), decodeStrict)
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import System.Exit (exitFailure)

-- | cabal test runs with the package root (core/) as cwd.
fixtureDir :: FilePath
fixtureDir = "../contracts/fixtures/"

main :: IO ()
main = do
  results <-
    sequence
      [ goldenPairs "handshake/hello-ok.ndjson"
      , goldenPairs "handshake/wire-errors.ndjson"
      , goldenPairs "fourclass/golden.ndjson"
      , goldenPairs "graph/golden.ndjson"
      , structural
      , costModel
      , Reference.equivalence
      ]
  unless (and results) exitFailure

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
  pure (a && b && c && d)

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
  let got = Protocol.respond "0.0.1" request
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
  pure (a && b && c && d && e && f && g && h && i)
 where
  unknownReply = Protocol.respond "0.0.1" "{\"proto\":\"2.1.0\",\"type\":\"mystery\",\"id\":7}"
  oversizeReply = Protocol.respond "0.0.1" (B8.replicate 33554433 'x')
  majorReply = Protocol.respond "0.0.1" "{\"proto\":\"9.0.0\",\"type\":\"hello\"}"
  overCapReply =
    Protocol.respond "0.0.1" $
      "{\"proto\":\"2.1.0\",\"type\":\"graph.request\",\"id\":3,\"nodes\":["
        <> B8.intercalate "," (replicate (fromInteger nodeCap + 1) "[0,0,0]")
        <> "],\"edges\":[],\"pos\":[]}"
  -- cap check precedes validation by design, so identical edge rows
  -- are fine here (never validated)
  edgeCapReply =
    Protocol.respond "0.0.1" $
      "{\"proto\":\"2.1.0\",\"type\":\"graph.request\",\"id\":4,\"nodes\":[[0,0,0]],\"edges\":["
        <> B8.intercalate "," (replicate (fromInteger edgeCap + 1) "[0,0,0,0]")
        <> "],\"pos\":[]}"

field :: B8.ByteString -> String -> Maybe Value
field bytes key = do
  Object o <- decodeStrict bytes
  KM.lookup (Key.fromString key) o

subfield :: B8.ByteString -> String -> String -> Maybe Value
subfield bytes key sub = do
  Object o <- field bytes key
  KM.lookup (Key.fromString sub) o
