-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce join` document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/join/mod.rs
-- `file_rows` / `judged_files`, cli/src/join/verdicts.rs
-- `pair_verdicts`, cli/src/join/churn_unit.rs `rows`, the document in
-- cli/src/join/report.rs `report_json`): the file pairs in path order,
-- each with both sides' graph position and window churn, the
-- co-change count where the pair made the table, and the lattice's
-- verdict on the pair; the unit pairs in the order the clone families
-- found them, each with its family's metric and churn, its graph leg
-- null by design with the reason as a code. The measuring side sends
-- one path table (every path a row or a candidate names, with each
-- path's place in string order), the graph reply's positions, the
-- verdict reply's candidates and severities re-numbered into that
-- table, its own sums, and each degraded reply's reason.
module CE.Join.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)
import Data.List (sortOn)
import qualified Data.Map.Strict as M

-- | The catalogue lists the degraded reasons the two legs are sent by
-- (plan v2.32 step 4B).
doc :: DocFamily
doc = docFamily "join" schemaId statement checked assemble ["reasons" .= coreReasons]

schemaId :: String
schemaId = "ce.join-report/0.4.0"

-- | A file row is [a, b, blocks, tokens, near-miss, a appended, a
-- rewrote, b appended, b rewrote]; a pos row graph/1's [path, indeg,
-- outdeg, sccId, sccSize, reachIn]; a co-change row [a, b, commits];
-- a candidate verdict/1's six columns; a unit row [k, a path, a nth,
-- b path, b nth, family (0 t1t2, 1 t3), three metric columns (tokens;
-- or ted, n1, n2), then the two sides' churn].
statement :: String
statement =
  "range paths\nrange units\nrange why\n\
  \fact days judged\nfact commits judged\n\
  \rows rankPaths 2 judged paths -\n\
  \rows files 9 judged paths paths - - - - - - -\nrows pos 6 judged paths - - - - -\n\
  \rows cochange 3 judged paths paths -\nrows candidates 6 judged paths paths - - - -\n\
  \rows joinSeverity 2 judged - -\n\
  \rows units 13 judged units paths - paths - - - - - - - - -\n\
  \rows graphReason 1 judged -\nrows verdictReason 1 judged -\n\
  \ref path paths\nref key units -\nref why why\n"

-- | The lattice's codes by name (CE.Verdict.Join.verdictTable; 0 is
-- report_only).
verdictNames :: [String]
verdictNames = words "report_only merge_candidate delete_candidate churn_hotspot"

-- | Why a unit row's graph leg is null: import granularity has no unit
-- nodes. A code, never a sentence; there is no 0.
importGranularity :: Integer
importGranularity = 1

checked :: DocReq -> Maybe String
checked req =
  asum
    [ if dDegraded req == Nothing then dense req "rankPaths" (range req "paths") else Nothing
    , dense req "units" (range req "units")
    , codes req "candidates" 2 0 (toInteger (length verdictNames) - 1)
    , codes req "units" 5 0 1
    , asum [x | t <- ["graphReason", "verdictReason"], x <- [single req t, codes req t 0 0 (toInteger (length coreReasons) - 1)]]
    ]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= schemaId
    , "days" .= fact req "days"
    , "commits" .= fact req "commits"
    , "degraded" .= degradedOf req ["graphReason", "verdictReason"]
    , "files" .= map (filePair req) (sortOn placed (rows req "files"))
    , "units" .= map unitPair (rows req "units")
    ]
 where
  rank = M.fromList [(p, r) | [p, r] <- rows req "rankPaths"]
  placed r = [M.findWithDefault (-1) p rank | p <- take 2 r]

-- | One file pair with its legs: the positions keyed by path (the last
-- row wins, the reply's own map), the first co-change row of the pair,
-- the candidate on exactly (a, b) — orientation included — and the
-- severity its code ranks (0 when unlisted).
filePair :: DocReq -> [Integer] -> Value
filePair req row = case row of
  [a, b, blocks, tokens, nearMiss, aa, ar, ba, br] ->
    let verdict = M.lookup (a, b) judged
     in object
          [ "a" .= ref "path" [a]
          , "b" .= ref "path" [b]
          , "blocks" .= blocks
          , "tokens" .= tokens
          , "near_miss" .= nearMiss
          , "graph_a" .= M.lookup a positions
          , "graph_b" .= M.lookup b positions
          , "churn_a" .= churn aa ar
          , "churn_b" .= churn ba br
          , "cochange" .= M.lookup (a, b) cochanged
          , "verdict" .= fmap (spelled verdictNames . fst) verdict
          , "severity" .= fmap (\(c, _) -> M.findWithDefault 0 c severity) verdict
          , "confidence" .= fmap snd verdict
          ]
  _ -> Null
 where
  positions = M.fromList [(p, rest) | p : rest <- rows req "pos"]
  cochanged = M.fromListWith (\_ first -> first) [((x, y), n) | [x, y, n] <- rows req "cochange"]
  judged = M.fromList [((u, v), (c, conf)) | [u, v, c, _, _, conf] <- rows req "candidates"]
  severity = M.fromList [(c, s) | [c, s] <- rows req "joinSeverity"]

-- | Lines appended and rewritten over the window.
churn :: Integer -> Integer -> Value
churn appended rewrote = object ["appended" .= appended, "rewrote" .= rewrote]

-- | One unit pair: each side's identity, the family with its metric,
-- the churn, the null graph leg and its code.
unitPair :: [Integer] -> Value
unitPair row = case row of
  [k, af, an, bf, bn, family, m1, m2, m3, aa, ar, ba, br] ->
    object $
      [ "a" .= side k 0 af an
      , "b" .= side k 1 bf bn
      , "churn_a" .= churn aa ar
      , "churn_b" .= churn ba br
      , "graph" .= Null
      , "caveatCode" .= importGranularity
      ]
        <> if family == 0
          then ["kind" .= ("t1t2" :: String), "tokens" .= m1]
          else ["kind" .= ("t3" :: String), "ted" .= m1, "n1" .= m2, "n2" .= m3]
  _ -> Null
 where
  side k s f nth = object ["path" .= ref "path" [f], "key" .= ref "key" [k, s], "nth" .= nth]
