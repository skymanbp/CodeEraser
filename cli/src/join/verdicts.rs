//! The join face's judgment leg (2.33.0, H4): the SAME verdict/1
//! road `ce check` judges with, driven from the join's own single
//! measurement — one judgment, two faces. The score-side tables
//! this face has no stake in (baseline, members, size facts) ride
//! empty and their axes are ignored; the candidate rows and the
//! severity face are what it consumes, sent on to the join document
//! as the reply gave them (CE.Join.Document names the verdicts and
//! ranks them, plan v2.32 step 4).

use crate::graph::deadcode::{self, GraphWire};
use crate::join::Pos;
use crate::score::{self, wire};
use anyhow::Result;
use std::collections::HashMap;

/// What the document reads from the judgment: the verdict universe
/// the candidates' two columns index, the candidate rows and the
/// severity face as the reply gave them (the reasons / legs columns
/// ride along, unrendered since 2.33.0), and the degraded note (a
/// refused judgment reports, never pretends report_only).
pub struct Judged {
    pub(super) files: Vec<String>,
    pub(super) candidates: Vec<[i64; 6]>,
    pub(super) join_severity: Vec<[i64; 2]>,
    pub degraded: Option<String>,
    /// The verdict's core link, whole: the join document rides it.
    pub(super) held: crate::document::Held,
}

pub fn judge_pairs(
    root: &std::path::Path,
    core: &str,
    w: &GraphWire,
    sim: &score::Similar<'_>,
    (posmap, self_loops): (&HashMap<String, Pos>, Vec<i64>),
    ch: &crate::churn::Report,
) -> Result<Judged> {
    let files: Vec<String> = deadcode::measured_nodes(w)
        .iter()
        .map(|&(_, p)| p.to_string())
        .collect();
    let idx = score::row_index(&files);
    let mut rows = Vec::new();
    score::clone_rows(sim, &idx, &mut rows);
    // two clone families on this road too (5b-9): one row per pair
    score::one_row_per_pair(&mut rows);
    let (churn_t, cochange_t) = score::churn_tables(ch, &idx);
    let pos = score::pos_rows(&files, posmap);
    // RG10's fact on this road too (6.1.0): `ce join` is the face
    // that PRINTS a delete verdict, so leaving the export surface off
    // it would keep the guard inert exactly where a reader acts on it
    let symbols = crate::graph::symwire::rekeyed(w, &idx)?;
    let req = request(
        root,
        files,
        rows,
        pos,
        (symbols, self_loops),
        (churn_t, cochange_t),
    )?;
    let (reply, link) = wire::judge(core, &req)?;
    Ok(Judged {
        files: req.files,
        candidates: reply.candidates,
        join_severity: reply.join_severity,
        degraded: reply.degraded,
        held: Ok(link),
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
    (symbols, self_loops): (Vec<[i64; 2]>, Vec<i64>),
    (churn, cochange): score::ChurnTables,
) -> Result<wire::Request> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    Ok(wire::Request {
        sim,
        pos,
        symbols,
        // no baseline rides this road, so there is no provenance to
        // answer; the self-loop table rides at floor 1 like the check
        // road's, because the cycle axis is judged here too
        present: None,
        cycle_self_loops: (cfg.graph.scc_floor == Some(1)).then_some(self_loops),
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
        thresholds: score::knobs::threshold_rows(&cfg.score, cfg.graph.scc_floor),
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
