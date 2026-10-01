-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce structure` document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/structure/judge.rs
-- `run` / `tree_rows` / `split_relabel` / `relabel`, the document in
-- cli/src/structure/report.rs `report_json`): the score and its
-- scale, the entropy and axis rows, the findings and deviations named
-- by directory, the flat directory tree with each node's axes rolled
-- from the findings, and — only when the advisory rode — the split
-- candidates and the size exemptions. The structure family has no
-- degraded document: a judgment that does not happen prints nothing,
-- so this family takes no reason. Every array keeps the order the
-- judgment answered it in; the tree is in directory-id order.
module CE.Structure.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, (.=))
import Data.Aeson.Key (Key)
import Data.Aeson.Types (Pair)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = docFamily "structure" schemaId statement checked assemble []

schemaId :: String
schemaId = "ce.structure-report/0.6.0"

-- | The request: a tree row is [id, parent, depth, subdirs, files]
-- (the root its own parent); a split candidate structure/1's [file,
-- unit, benefit, cost] with the unit's last line after it; an
-- exemption structure/1's [file, benefit, cost]. `seamFiles` numbers
-- the files the advisory measured.
statement :: String
statement =
  "range dirs\nrange seamFiles\n\
  \fact score judged\nfact scale judged\nfact declaredDirs judged\nfact deep judged\nfact split judged\n\
  \rows days 1 judged -\nrows divergence 1 judged -\n\
  \rows entropy 2 judged - -\nrows axes 2 judged - -\n\
  \rows tree 5 judged dirs dirs - - -\n\
  \rows findings 2 judged dirs -\nrows deviations 2 judged dirs -\n\
  \rows splitCandidates 5 judged seamFiles - - - -\nrows sizeExempt 3 judged seamFiles - -\n\
  \ref dir dirs\nref path seamFiles\nref unit seamFiles -\n"

-- | One tree row per directory in id order, the nullable tables one
-- row at most, the two switches 0 / 1, and no advisory row unless it
-- rode.
checked :: DocReq -> Maybe String
checked req =
  asum
    [ dense req "tree" (range req "dirs")
    , asum [single req t | t <- ["days", "divergence"]]
    , bits req ["deep", "split"]
    , if flag req "split" || all (null . rows req) ["splitCandidates", "sizeExempt"] then Nothing else Just "split: advisory rows without the advisory"
    ]

assemble :: DocReq -> Value
assemble req =
  object $
    [ "schema" .= schemaId
    , "score" .= fact req "score"
    , "scoreScale" .= fact req "scale"
    , "entropy" .= rows req "entropy"
    , "axes" .= rows req "axes"
    , "findings" .= labelled "axis" (rows req "findings")
    , "dirs" .= range req "dirs"
    , "divergence" .= optional req "divergence"
    , "deviations" .= labelled "kind" (rows req "deviations")
    , "declaredDirs" .= fact req "declaredDirs"
    , "deep" .= flag req "deep"
    , "days" .= optional req "days"
    , "tree" .= map (node rolled) (rows req "tree")
    , "split" .= flag req "split"
    ]
      <> advisory req
 where
  rolled = M.fromListWith (flip (<>)) [(d, [a]) | [d, a] <- rows req "findings"]

-- | [dirId, code] rows as {dir, <code key>} objects, in reply order.
labelled :: Key -> [[Integer]] -> [Value]
labelled key table = [object ["dir" .= ref "dir" [d], key .= c] | [d, c] <- table]

-- | One directory with the axes its findings name, in findings order.
node :: M.Map Integer [Integer] -> [Integer] -> Value
node rolled row = case row of
  [i, parent, depth, subdirs, files] ->
    object
      [ "id" .= i
      , "parent" .= parent
      , "name" .= ref "dir" [i]
      , "depth" .= depth
      , "subdirs" .= subdirs
      , "files" .= files
      , "axes" .= M.findWithDefault [] i rolled
      ]
  _ -> Null

-- | The two advisory arrays, present exactly when the advisory rode.
advisory :: DocReq -> [Pair]
advisory req
  | flag req "split" =
      [ "splitCandidates" .= [object (milli f b c <> ["afterLine" .= after, "unit" .= ref "unit" [f, u]]) | [f, u, b, c, after] <- rows req "splitCandidates"]
      , "sizeExempt" .= [object (milli f b c) | [f, b, c] <- rows req "sizeExempt"]
      ]
  | otherwise = []
 where
  milli f b c = ["path" .= ref "path" [f], "benefitMilli" .= b, "costMilli" .= c]
