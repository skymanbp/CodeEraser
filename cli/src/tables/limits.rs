//! The numbers requests are laid out by and candidates selected with
//! (plan v2.33 W3, CE.Limits): the definition package's `limits` key,
//! each table the owning family's constants, typed once at read like
//! the rest of the package (pack.rs).

use super::leak::leaked;

leaked! {
    /// The clone judgment's thresholds and ceilings (CE.Clone.Cost).
    CloneLimits { tsed_num: i64, tsed_den: i64, min_unit_nodes: i64, unit_node_cap: i64,
                  pair_cap: usize }
    /// The candidates family's ceilings and the MinHash/LSH shape
    /// (CE.Candidates.Cost).
    CandidateLimits { unit_cap: usize, sig_cap: usize, print_cap: usize, near_cap: usize,
                      lsh_perms: usize, lsh_bands: usize, lsh_rows: usize }
    /// The docdup judgment's ratio and ceilings (CE.Docdup.Cost).
    DocdupLimits { jaccard_num: u64, jaccard_den: u64, doc_set_cap: usize, doc_seq_cap: usize,
                   doc_corpus_cap: usize, doc_pair_cap: usize }
    /// The T1/T2 report's diversity floor and the hot-group cap every
    /// hash-group walk chains by (CE.Dedup.Cost).
    DedupLimits { min_distinct: usize, hot_cap: usize }
    /// The similar family's table ceiling (CE.Similar.Cost) and the
    /// ranking's constants (CE.Similar.Rank.Cost): the frozen eval docs
    /// echo them; the measuring side fetches by the two ratios and the
    /// co-occurrence floor and computes nothing else with them.
    SimilarLimits { similar_cap: usize, rank_cap: usize, k1: (i64, i64), b: (i64, i64),
                    idf_frac_bits: i64, score_frac_bits: i64, w_unit: i64, top_m: usize,
                    min_cooc: u64, min_ppmi: i64, ppmi_cap: i64, ppmi_scale: i64,
                    scored_df_ratio: u64, neighbour_df_ratio: u64 }
    /// The L1 judgment's request ceilings (CE.FourClass.Moves.Cost).
    MovesLimits { line_cap: usize, unit_cap: usize }
    /// The request ceilings the scan, flow, merge, structure and arch
    /// families plan their requests by (plan v2.33 W1; each its
    /// family's Cost constant, once held here as a second literal), and
    /// the two an index refresh's batched bags ask is planned by (W2-text
    /// Z4: the bags/1 item ceiling, the protocol's line in bytes).
    WireCaps { scan_rows: usize, flow_rows: usize, merge_groups: usize,
               merge_tree_nodes: usize, structure_nodes: usize, arch_files: usize,
               arch_refs: usize, bags_items: usize, line_bytes: usize }
    /// The numbers requests are laid out by and candidates selected with
    /// (plan v2.33 W3, CE.Limits): each the owning family's constant.
    Limits { clone: CloneLimits, candidates: CandidateLimits, docdup: DocdupLimits,
             dedup: DedupLimits, similar: SimilarLimits, moves: MovesLimits, caps: WireCaps }
}
