-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The graph screen's document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/graph/canvas.rs
-- `screen` / `document` / `file_edges` / `file_cycles`): one judgment,
-- two halves — the canvas the GUI draws and the deadcode document
-- (CE.Graph.Document, the same assembly `ce deadcode` prints). The
-- canvas lists every file node in node order with its verdict, its
-- position and whether a reported cycle holds it; the edges collapse
-- onto files (a package endpoint drops the arc, a section stands for
-- the file that holds it, self-loops and repeats drop) in ascending
-- order. The measuring side sends the deadcode request and, beside
-- it, each node's kind and the file row its path names (−1: none),
-- the graph request's edges, the reply's positions and its cycle
-- members.
module CE.Graph.Screen (doc) where

import CE.Document.Contract
import qualified CE.Graph.Document as Dead
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import qualified Data.Set as S

doc :: DocFamily
doc = docFamily "graphscreen" screenId statement checked assemble []

screenId, canvasId :: String
screenId = "ce.graph-screen/0.1.0"
canvasId = "ce.graph-canvas/0.4.0"

-- | The deadcode request and the canvas's: a graph row is [node, kind
-- (0 file, 1 package, 2 section), file row]; a cycle row [cycle,
-- member].
statement :: String
statement =
  Dead.statement
    <> "rows graph 3 judged nodes - -\nrows edges 2 judged nodes nodes\n\
       \rows pos 6 judged nodes - - - - -\nrows cycles 2 judged - nodes\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ dfCheck Dead.doc req
    , if dDegraded req == Nothing then dense req "graph" (range req "nodes") else Nothing
    , codes req "graph" 1 0 2
    , codes req "graph" 2 (-1) (toInteger (length (fileNodes req)) - 1)
    ]

-- | The file nodes in node order: the canvas's rows.
fileNodes :: DocReq -> [Integer]
fileNodes req = [n | [n, 0, _] <- rows req "graph"]

assemble :: DocReq -> Value
assemble req = object ["schema" .= screenId, "canvas" .= canvas req, "deadcode" .= Dead.deadcode req]

canvas :: DocReq -> Value
canvas req =
  object
    [ "schema" .= canvasId
    , "files" .= zipWith row [0 ..] (fileNodes req)
    , "edges" .= [[a, b] | (a, b) <- S.toAscList arcs]
    , "counts" .= object ["files" .= length (fileNodes req), "edges" .= S.size arcs, "dead" .= length (rows req "dead"), "cycles" .= S.size cyclic]
    , "unresolvedSites" .= fact req "unresolvedSites"
    , "degraded" .= degradedOf req ["reason"]
    ]
 where
  graph = IM.fromList [(fromInteger n, (k, f)) | [n, k, f] <- rows req "graph"]
  kindOf n = maybe (-1) fst (IM.lookup (fromInteger n) graph)
  fileOf n = maybe (-1) snd (IM.lookup (fromInteger n) graph)
  byFile table = M.fromList [(fileOf n, rest) | n : rest <- rows req table, fileOf n >= 0]
  (dead, positions) = (byFile "dead", byFile "pos")
  members = [(c, m) | [c, m] <- rows req "cycles", kindOf m == 0]
  held = S.fromList [fileOf m | (_, m) <- members]
  cyclic = S.fromList (map fst members)
  arcs = S.fromList [(a, b) | [s, d] <- rows req "edges", kindOf s /= 1, kindOf d /= 1, let (a, b) = (fileOf s, fileOf d), a >= 0, b >= 0, a /= b]
  row r n =
    let verdict = M.lookup r dead
        code v = if v > 2 then 1 else 0 :: Integer
     in object
          [ "path" .= ref "path" [n]
          , "verdict" .= maybe Null (Dead.verdictName . head') verdict
          , "why" .= maybe Null (spelled Dead.whyCodes . code . head') verdict
          , "whyCode" .= fmap (code . head') verdict
          , "conf" .= (verdict >>= second)
          , "pos" .= M.lookup r positions
          , "cycle" .= S.member r held
          ]
  head' xs = case xs of
    x : _ -> x
    [] -> 0
  second xs = case xs of
    _ : t : _ -> Just t
    _ -> Nothing
