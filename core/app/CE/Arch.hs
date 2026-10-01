-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | arch.request handler (plan v2.31 step 8; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §7):
-- the architecture of the tree, as advice. The measuring side sends
-- the measured files with their directories and line counts, the
-- directory tree, the file-to-file references, the package-grain
-- references a file makes to a whole directory (the Go / R / Java
-- package imports and the Markdown directory links structure/1's
-- `dirEdges` drops), and the files `--impact` names; this side folds
-- the references onto the directory graph, finds its cheapest
-- feedback arc set one component at a time, layers what is left,
-- clusters the file graph, names the files outside their cluster's
-- directory, walks the impact of the focus, and measures each
-- directory's fan-in, fan-out and instability. Advisory: the reply
-- carries no condition bit and no face fails a run on it (§1 ruling
-- 3). Two caps — files, and the two reference tables together.
module CE.Arch (respond) where

import CE.Arch.Contract (ArchReq (..), offence, overCap)
import CE.Arch.Dirs (arcsOf, dirOfFile, fileNeighbours)
import CE.Arch.Fas (fas)
import CE.Arch.Impact (impact)
import CE.Arch.Layers (levels, metrics)
import CE.Arch.Louvain (clusters, misplaced)
import qualified CE.Wire as Wire
import Data.Aeson (Value, encode, object, (.=))
import qualified Data.ByteString as B
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S

-- | The six answer tables, ascending by their first column.
data Answer = Answer
  { layers :: [[Integer]]
  , cuts :: [[Integer]]
  , grouped :: [[Integer]]
  , outside :: [[Integer]]
  , reach :: [[Integer]]
  , measured :: [[Integer]]
  }

-- | The judgment over a request the contract admitted.
judge :: ArchReq -> Answer
judge req =
  Answer
    { layers = levels nDirs arcs (S.fromList [(a, b) | (a, b, _, _) <- cut])
    , cuts = [[toInteger a, toInteger b, w, e] | (a, b, w, e) <- cut]
    , grouped = [[toInteger f, toInteger c] | (f, c) <- IM.toList cluster]
    , outside = misplaced dirOf cluster
    , reach = impact dirOf (edgeRows req) (pkgRows req) (focusRows req)
    , measured = metrics nDirs arcs
    }
 where
  nDirs = length (dirRows req)
  dirOf = dirOfFile (fileRows req)
  arcs = arcsOf dirOf (edgeRows req) (pkgRows req)
  cut = fas arcs
  cluster = clusters (length (fileRows req)) (fileNeighbours (edgeRows req))

-- | The arch.result object. Judged: the six tables and their counts.
-- Degraded: the six tables empty — a request the core refused to
-- judge licenses nothing — the five request tables still counted, the
-- answer counts 0, and the reason.
answer :: String -> Bool -> ArchReq -> B.ByteString
answer proto degraded req = BL.toStrict (encode (object (fields <> ["reason" .= ("arch_too_large" :: String) | degraded])))
 where
  a = if degraded then Answer [] [] [] [] [] [] else judge req
  sizes = map (toInteger . length) [fileRows req, dirRows req, edgeRows req, pkgRows req] <> [toInteger (length (focusRows req))]
  distinct = toInteger (S.size (S.fromList [c | [_, c] <- grouped a]))
  tallies = sizes <> [toInteger (length (cuts a)), distinct, toInteger (length (outside a)), toInteger (length (reach a))]
  fields =
    [ "proto" .= proto
    , "type" .= ("arch.result" :: String)
    , "id" .= reqId req
    , "layers" .= layers a
    , "cuts" .= cuts a
    , "clusters" .= grouped a
    , "misplaced" .= outside a
    , "impact" .= reach a
    , "metrics" .= measured a
    , "counts" .= object (zipWith (.=) ["files", "dirs", "edges", "pkgEdges", "focus", "cuts", "clusters", "misplaced", "impact"] tallies)
    , "degraded" .= degraded
    ]

-- | decode → cap → contract → judge; the degraded reply is the judged
-- reply's shape with nothing judged.
respond :: String -> B.ByteString -> Either (Maybe Value, String, String) B.ByteString
respond proto = Wire.family "arch" reqId overCap offence (answer proto True) (answer proto False)
