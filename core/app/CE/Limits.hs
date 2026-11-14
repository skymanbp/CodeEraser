{-# LANGUAGE OverloadedStrings #-}

-- | The definition package's `limits` key (plan v2.33 W3): the numbers
-- the measuring side lays requests out by and selects with — the
-- judging families' table ceilings and the thresholds the candidate
-- passes and the T1/T2 report apply. Until W3 the measuring side held
-- its own copy of each and a reply's knob echo pinned the copy equal;
-- it now reads them here, from the same core that judges. Each value
-- is the owning family's Cost constant, never a second statement; the
-- keys are snake case like every other package key. W1 added `caps`:
-- the request ceilings the scan, flow, merge, structure and arch
-- families are planned by; W2-text Z4 the bags/1 item ceiling and the
-- protocol's line ceiling, which an index refresh's batched bags ask is
-- planned by. Step 11a (defragmentation) added `graph` — the cycle
-- floor and the advisory table's soft cap the measuring side plans its
-- request by — `zone`, the two per-mille tier lines the guard's
-- declared tiers are drawn at, and `score_defaults`, every verdict knob
-- at its default under its wire knob name (the degraded verdict reply's
-- own echo, CE.Verdict.Knobs.defaultsEcho). Step 11b added two `caps`:
-- erase/1's row ceiling (the plan is one request — the closure chooses
-- across the rows of one path — so the measuring side refuses a plan
-- over it by name before sending) and trend/2's (rows and declared knob
-- rows count against it together; the measuring side refuses a window
-- that cannot fit before measuring a commit of it). v2.35 added
-- cover/1's row ceiling, the T1/T2 run set is split between pairs by it;
-- and `score_opt_in`, the axes that weigh 0 unless a project names their
-- weight (the dead-file amendment), which the knob digest's shipped
-- weight table reads. Release 1.10.0 added `trend_defaults`, the trend
-- family's two knobs at their defaults (CE.Trend.Cost's knob table, the
-- digest's effective default), moved the co-occurrence term cap into
-- `similar`, and dropped six ceilings no reader took.
module CE.Limits (limits) where

import qualified CE.Arch.Cost as Arch
import qualified CE.Candidates.Cost as Candidates
import qualified CE.Clone.Cost as Clone
import qualified CE.Dedup.Cost as Dedup
import qualified CE.Docdup.Cost as Docdup
import qualified CE.Erase.Cost as Erase
import qualified CE.Flow.Cost as Flow
import qualified CE.Graph.Cost as Graph
import qualified CE.FourClass.Moves.Cost as Moves
import qualified CE.Merge.Cost as Merge
import CE.Protocol.Version (maxLineBytes)
import qualified CE.Scan.Cost as Scan
import qualified CE.Similar.Bags as Bags
import qualified CE.Similar.Rank.Cost as Rank
import qualified CE.Structure.Cost as Structure
import qualified CE.Trend.Cost as Trend
import qualified CE.Verdict.Cost as Verdict
import CE.Verdict.Knobs (defaultsEcho)
import Data.Aeson (Value, object, (.=))
import Data.Aeson.Types (Pair)
import Data.Ratio (denominator, numerator)

limits :: Value
limits =
  object
    [ "clone" .= object (cloneJudgment <> clonePlan)
    , "candidates"
        .= object
          [ "unit_cap" .= Candidates.candidateUnitCap
          , "lsh_perms" .= perms
          , "lsh_bands" .= bands
          , "lsh_rows" .= rows
          ]
    , "docdup"
        .= object
          [ "jaccard_num" .= Docdup.jaccardNum
          , "jaccard_den" .= Docdup.jaccardDen
          , "doc_set_cap" .= Docdup.docSetCap
          , "doc_seq_cap" .= Docdup.docSeqCap
          , "doc_pair_cap" .= Docdup.docPairCap
          ]
    , "dedup" .= object ["min_distinct" .= Dedup.minDistinct, "hot_cap" .= Dedup.hotCap]
    , "moves" .= object ["line_cap" .= Moves.movesLineCap, "unit_cap" .= Moves.movesUnitCap]
    , "graph" .= object ["scc_floor" .= Graph.sccFloor, "unmentioned_cap" .= Graph.unmentionedCap]
    , "zone" .= object ["warn_permille" .= Verdict.zoneWarnPermille, "ask_permille" .= Verdict.zoneAskPermille]
    , "score_defaults" .= defaultsEcho
    , "score_opt_in" .= Verdict.optInAxes
    , "trend_defaults" .= object ["min_points" .= knob 0, "decline_floor_micro" .= knob 1]
    , "caps" .= caps
    , "similar" .= similar
    ]
 where
  (perms, bands, rows) = Candidates.lshShape
  knob c = lookup c Trend.knobTable

-- | The clone object, in its two halves. What the T3 judgment decides by:
-- the similarity threshold as a ratio and the smallest unit it judges —
-- the same three a clone reply's verdicts follow from.
cloneJudgment :: [Pair]
cloneJudgment =
  ["tsed_num" .= Clone.tsedNum, "tsed_den" .= Clone.tsedDen, "min_unit_nodes" .= Clone.minUnitNodes]

-- | What one clone request is planned under: the largest tree it may carry,
-- its pair count and its work, Σ n1·n2 over its pairs (plan v2.35 C3) — a
-- request over any of them is answered degraded, so the measuring side
-- cuts its requests below all three.
clonePlan :: [Pair]
clonePlan =
  ["unit_node_cap" .= Clone.unitNodeCap, "pair_cap" .= Clone.pairCap, "work_cap" .= Clone.workCap]

-- | The request ceilings the families are planned by.
caps :: Value
caps =
  object
    [ "scan_rows" .= Scan.scanRowCap
    , "flow_rows" .= Flow.rowCap
    , "merge_groups" .= Merge.groupCap
    , "merge_tree_nodes" .= Merge.treeNodeCap
    , "structure_nodes" .= Structure.structNodeCap
    , "arch_files" .= Arch.fileCap
    , "arch_refs" .= Arch.refCap
    , "bags_items" .= Bags.bagsCap
    -- plan v2.35 C6: the bytes one bags ask is planned under
    , "bags_bytes" .= Bags.bagsAskBytes
    , "line_bytes" .= maxLineBytes
    , "erase_rows" .= Erase.eraseRowCap
    , "trend_rows" .= Trend.trendRowCap
    , "cover_rows" .= Dedup.coverRowCap
    ]

-- | The ranking's constants and the co-occurrence term cap.
similar :: Value
similar =
  object
    [ "k1" .= ratio Rank.k1
    , "b" .= ratio Rank.b
    , "idf_frac_bits" .= Rank.idfFracBits
    , "score_frac_bits" .= Rank.scoreFracBits
    , "w_unit" .= Rank.wUnit
    , "top_m" .= Rank.topM
    , "min_cooc" .= Rank.minCooc
    , "min_ppmi" .= Rank.minPpmi
    , "ppmi_cap" .= Rank.ppmiCap
    , "ppmi_scale" .= Rank.ppmiScale
    , "term_cap" .= Rank.termCap
    , "scored_df_ratio" .= Rank.scoredDfRatio
    , "neighbour_df_ratio" .= Rank.neighbourDfRatio
    ]
 where
  ratio q = [numerator q, denominator q]
