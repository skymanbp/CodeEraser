//! The join face's judgment leg (2.33.0, H4): the SAME verdict/1
//! road `ce check` judges with, driven from the join's own single
//! measurement — one judgment, two faces. The score-side tables
//! this face has no stake in (baseline, members, size facts) ride
//! empty and their axes are ignored; the candidate rows and the
//! severity face are what it consumes.

use crate::graph::deadcode::{self, GraphWire};
use crate::join::Pos;
use crate::score::{self, wire};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Rendering names for the lattice's codes — the codes are the
/// core's (CE.Verdict.Join.verdictTable).
pub const VERDICT_NAMES: [&str; 4] = [
    "report_only",
    "merge_candidate",
    "delete_candidate",
    "churn_hotspot",
];

/// One pair's core verdict, keyed for the join rows.
pub struct PairVerdict {
    pub verdict: &'static str,
    pub severity: i64,
    pub confidence: i64,
    pub legs_mask: i64,
    pub reasons: i64,
}

/// Everything the report needs from the judgment: per-pair verdicts
/// plus the wire's own degraded note (a refused judgment reports,
/// never pretends report_only).
pub struct Judged {
    pub pairs: HashMap<(String, String), PairVerdict>,
    pub degraded: Option<String>,
}

pub fn judge_pairs(
    root: &std::path::Path,
    core: &str,
    w: &GraphWire,
    blocks: &[crate::dedup::pairs::Block],
    posmap: &HashMap<String, Pos>,
    ch: &crate::churn::Report,
) -> Result<Judged> {
    let files: Vec<String> = deadcode::file_nodes(w)
        .iter()
        .map(|&(_, p)| p.to_string())
        .collect();
    let idx: HashMap<&str, i64> = files
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_str(), i as i64))
        .collect();
    let mut sim = Vec::new();
    score::sim_rows(blocks, &idx, &mut sim);
    let (churn_t, cochange_t) = score::churn_tables(ch, &idx);
    let pos = score::pos_rows(&files, posmap);
    let req = request(root, files, sim, pos, (churn_t, cochange_t))?;
    let reply = wire::judge(core, &req)?;
    let sev: HashMap<i64, i64> = reply.join_severity.iter().map(|&[c, s]| (c, s)).collect();
    let mut pairs = HashMap::new();
    for &[u, v, code, reasons, legs_mask, confidence] in &reply.candidates {
        let path_of = |i: i64| -> Result<String> {
            usize::try_from(i)
                .ok()
                .and_then(|i| req.files.get(i))
                .cloned()
                .context("candidate index outside the file universe — wire skew")
        };
        let name = *VERDICT_NAMES
            .get(usize::try_from(code).unwrap_or(usize::MAX))
            .context("verdict code outside the table — wire skew")?;
        pairs.insert(
            (path_of(u)?, path_of(v)?),
            PairVerdict {
                verdict: name,
                // an unlisted code ranks 0 by the same absence rule
                // the core's own report_only carries
                severity: sev.get(&code).copied().unwrap_or(0),
                confidence,
                legs_mask,
                reasons,
            },
        );
    }
    Ok(Judged {
        pairs,
        degraded: reply.degraded,
    })
}

/// The join road's verdict request (split from judge_pairs at the
/// E01 hard line when the 3.1.0 class channel joined the record):
/// the three legs' tables plus ce.toml's knob rows; the score-side
/// tables ride empty — so the class channel has no payload on this
/// road by construction — and their axes are ignored.
fn request(
    root: &std::path::Path,
    files: Vec<String>,
    sim: Vec<[i64; 5]>,
    pos: Vec<[i64; 6]>,
    (churn, cochange): score::ChurnTables,
) -> Result<wire::Request> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    Ok(wire::Request {
        sim,
        pos,
        churn,
        cochange,
        continuous: Vec::new(),
        classed: false,
        class_knobs: Vec::new(),
        // the join reads the verdict table, never the ratchet — it
        // sends no baseline, so there is nothing for a fence to guard
        knobs_digest: None,
        discrete: Vec::new(),
        baseline: serde_json::Value::Null,
        floor: None,
        ceilings: score::knobs::ceiling_rows(&cfg.thresholds, &cfg.score),
        weights: score::knobs::weight_rows(&cfg.score)?,
        thresholds: score::knobs::threshold_rows(&cfg.score),
        tolerance: score::knobs::tolerance_rows(&cfg.score),
        dedup: None,
        dedup_distinct: Vec::new(),
        dedup_min_distinct: None,
        judged_loc: Vec::new(),
        doc_files: score::doc_file_indices(&files),
        files,
        judged_mask: crate::scan::lang::Lang::judged_mask(),
    })
}
