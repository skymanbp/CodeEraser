{-# LANGUAGE OverloadedStrings #-}

-- | The definition package's `limits` key (plan v2.33 W3): the numbers
-- the measuring side lays requests out by and selects with — the
-- judging families' table ceilings and the thresholds the candidate
-- passes and the T1/T2 report apply. Until W3 the measuring side held
-- its own copy of each and a reply's knob echo pinned the copy equal;
-- it now reads them here, from the same core that judges. Each value
-- is the owning family's Cost constant, never a second statement; the
-- keys are snake case like every other package key.
module CE.Limits (limits) where

import qualified CE.Candidates.Cost as Candidates
import qualified CE.Clone.Cost as Clone
import qualified CE.Dedup.Cost as Dedup
import qualified CE.Docdup.Cost as Docdup
import qualified CE.Similar.Cost as Similar
import qualified CE.Similar.Rank.Cost as Rank
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
          , "pair_cap" .= Candidates.candidatePairCap
          ]
    , "docdup"
        .= object
          [ "jaccard_num" .= Docdup.jaccardNum
          , "jaccard_den" .= Docdup.jaccardDen
          , "doc_set_cap" .= Docdup.docSetCap
          , "doc_pair_cap" .= Docdup.docPairCap
          ]
    , "dedup" .= object ["min_distinct" .= Dedup.minDistinct]
    , "similar"
        .= object
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
    ]
 where
  ratio q = [numerator q, denominator q]
