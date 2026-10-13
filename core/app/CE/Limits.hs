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
-- that cannot fit before measuring a commit of it).
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
import qualified CE.Similar.Cost as Similar
import qualified CE.Similar.Rank.Cost as Rank
import qualified CE.Structure.Cost as Structure
import qualified CE.Trend.Cost as Trend
import qualified CE.Verdict.Cost as Verdict
import CE.Verdict.Knobs (defaultsEcho)
import Data.Aeson (Value, object, (.=))
import Data.Ratio (denominator, numerator)

limits :: Value
limits =
  object
    [ "clone"
        .= object
          [ "tsed_num" .= Clone.tsedNum
          , "tsed_den" .= Clone.tsedDen
          , "min_unit_nodes" .= Clone.minUnitNodes
          , "unit_node_cap" .= Clone.unitNodeCap
          , "pair_cap" .= Clone.pairCap
          ]
    , "candidates"
        .= object
          [ "unit_cap" .= Candidates.candidateUnitCap
          , "sig_cap" .= Candidates.candidateSigCap
          , "print_cap" .= Candidates.candidatePrintCap
          , "near_cap" .= Candidates.candidateNearCap
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
          , "doc_corpus_cap" .= Docdup.docCorpusCap
          , "doc_pair_cap" .= Docdup.docPairCap
          ]
    , "dedup" .= object ["min_distinct" .= Dedup.minDistinct, "hot_cap" .= Dedup.hotCap]
    , "moves" .= object ["line_cap" .= Moves.movesLineCap, "unit_cap" .= Moves.movesUnitCap]
    , "graph" .= object ["scc_floor" .= Graph.sccFloor, "unmentioned_cap" .= Graph.unmentionedCap]
    , "zone" .= object ["warn_permille" .= Verdict.zoneWarnPermille, "ask_permille" .= Verdict.zoneAskPermille]
    , "score_defaults" .= defaultsEcho
    , "caps" .= caps
    , "similar" .= similar
    ]
 where
  (perms, bands, rows) = Candidates.lshShape

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
    , "line_bytes" .= maxLineBytes
    , "erase_rows" .= Erase.eraseRowCap
    , "trend_rows" .= Trend.trendRowCap
    ]

-- | The similar family's ceilings and the ranking's constants.
similar :: Value
similar =
  object
    [ "similar_cap" .= Similar.similarCap
    , "rank_cap" .= Rank.rankCap
    , "k1" .= ratio Rank.k1
    , "b" .= ratio Rank.b
    , "idf_frac_bits" .= Rank.idfFracBits
    , "score_frac_bits" .= Rank.scoreFracBits
    , "w_unit" .= Rank.wUnit
    , "top_m" .= Rank.topM
    , "min_cooc" .= Rank.minCooc
    , "min_ppmi" .= Rank.minPpmi
    , "ppmi_cap" .= Rank.ppmiCap
    , "ppmi_scale" .= Rank.ppmiScale
    , "scored_df_ratio" .= Rank.scoredDfRatio
    , "neighbour_df_ratio" .= Rank.neighbourDfRatio
    ]
 where
  ratio q = [numerator q, denominator q]
