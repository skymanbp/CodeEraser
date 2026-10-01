-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | merge.request handler (plan v2.31 step 6; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §6):
-- the fifteenth judgment family — how a clone group would merge into
-- one function, behind `ce merge`, the MCP tool and the GUI screen.
-- The measuring side sends each group's members (unit, lines, file
-- in-degree) and one tree per member in clone/1's postorder encoding
-- with four more columns — each node's source-text hash, position
-- class, own-token hash and token-text hash (§5.9.2: no name, path or
-- source text crosses, only integers) — and this side aligns the
-- members (a T1/T2 group node for node, a T3 pair by the tree-edit
-- mapping), finds the holes, widens and numbers the parameters,
-- and answers per group the parameter count, the member kept, the
-- lines saved and whether a merge is feasible, with the reason when
-- it is not, and per hole each member's first and last root, so the
-- measuring side reads a gap's whole forest back. Advisory: no
-- condition bit, no gate reads it.
module CE.Merge (respond) where

import CE.Clone (WireTree (..))
import CE.Merge.Contract (MergeReq (..), MergeTree (..), groupsOf, offence, overCap)
import CE.Merge.Holes (suggest)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import Data.ByteString (ByteString)
import Data.ByteString.Lazy (toStrict)
import Data.Maybe (isJust)

-- | decode → caps (groups, nodes) → contract → judge; the degraded
-- reply is the judged reply's shape with nothing judged.
respond :: String -> ByteString -> Either (Maybe Value, String, String) ByteString
respond proto = family "merge" reqId overCap offence (answer proto (Just "merge_too_large")) (answer proto Nothing)

-- | The merge.result object. Judged (no reason): one suggestion row
-- [g,params,kept,savings,feasible,reason] per group in request order
-- (groups ascend by g), its hole rows [g,hole,param,m,post,postEnd]
-- after it (`post` .. `postEnd` the member's roots of the hole, one
-- node twice for a leaf hole, −1 −1 an empty side). Degraded (the reason): no rows — a request the core
-- refused to judge licenses nothing. The counts name the three
-- request dimensions either way.
answer :: String -> Maybe String -> MergeReq -> ByteString
answer proto reason req = toStrict (encode (object (fields <> maybe [] (\why -> ["reason" .= why]) reason)))
 where
  judged = maybe (map suggest (groupsOf req)) (const []) reason
  rows = map fst judged
  holes = concatMap snd judged
  nodes = sum (map (length . wLab . tWire) (treeRows req))
  feasible = length [() | row <- rows, take 1 (drop 4 row) == [1]]
  tallies = map length [groupRows req, memberRows req] <> [nodes, length rows, length holes, feasible]
  fields =
    [ "proto" .= proto
    , "type" .= ("merge.result" :: String)
    , "id" .= reqId req
    , "suggestions" .= rows
    , "holes" .= holes
    , "counts" .= object (zipWith (.=) ["groups", "members", "nodes", "suggestions", "holes", "feasible"] tallies)
    , "degraded" .= isJust reason
    ]
