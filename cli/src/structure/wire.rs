//! structure/1 wire codec (contracts/fixtures/structure/golden.ndjson
//! is the byte-level contract; corelink stamps proto/type/id): the
//! tree-scale fact tables out, the judged axes / score / entropy /
//! findings back. This family keeps NO Rust verdict mirror by
//! design-booklet ruling (no frozen instrument needs one — the
//! review-repair C1 seam class is closed at the design table), so
//! the client parses and relays; a degraded reply to a client-sized
//! request is a cap-mirror drift error, never a judgment.

use anyhow::{Context, Result, ensure};
use serde_json::json;

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "structure/1";

/// Node ceiling — mirror of CE.Structure.Cost.structNodeCap.
pub const STRUCT_NODE_CAP: usize = 524288;

/// The assembled request tables (dense ids; names never cross).
pub struct Request {
    pub nodes: Vec<[u64; 5]>,
    pub patterns: Vec<[u64; 3]>,
    pub conventions: Vec<[u64; 2]>,
    pub file_refs: Vec<[u64; 4]>,
    /// The A-layer template (S3a): [dirId, weight] rows compiled
    /// from ce.toml's [structure] layout, dirId-ascending. Empty =
    /// no declaration — the reply carries no A-layer keys at all.
    pub declared: Vec<[u64; 2]>,
    /// The S6 rollup (S3b): [dirId, dupBlocks, deadUnits] rows.
    /// None = the table stays off the wire and axis 6 is honestly
    /// unjudged; Some(empty) = judged clean (absence vs zero).
    pub redundancy: Option<Vec<[u64; 3]>>,
}

/// The core's verdict, raw: nothing here is derived Rust-side.
pub struct Reply {
    pub axes: Vec<[i64; 2]>,
    pub score: i64,
    pub entropy: Vec<[i64; 2]>,
    /// Sparse [dirId, axis] drill-down rows.
    pub findings: Vec<[i64; 2]>,
    pub knobs: Vec<[i64; 2]>,
    /// per-mille χ² against the declared layout: None = either no
    /// declaration or undeclared territory holds mass (the
    /// deviations rows then say where — the number is never faked).
    pub divergence: Option<i64>,
    /// Named [dirId, kind] deviation rows (0 = undeclared territory
    /// with files, 1 = a declared bin owning none); empty when no
    /// layout is declared.
    pub deviations: Vec<[i64; 2]>,
}

/// One structure.request over one link.
pub fn judge(core: &str, r: &Request) -> Result<Reply> {
    ensure!(
        r.nodes.len() <= STRUCT_NODE_CAP,
        "{} directory nodes exceed the structure/1 cap {STRUCT_NODE_CAP}",
        r.nodes.len()
    );
    let mut link = crate::lockstep::open_family(core, CAP)?;
    let mut body = json!({
        "nodes": r.nodes,
        "patterns": r.patterns,
        "conventions": r.conventions,
        "fileRefs": r.file_refs,
    });
    if !r.declared.is_empty() {
        body["declared"] = json!(r.declared);
    }
    if let Some(rows) = &r.redundancy {
        body["redundancy"] = json!(rows);
    }
    let reply = link
        .request("structure", body)
        .map_err(anyhow::Error::msg)?;
    crate::lockstep::refuse_degraded(&reply, "structure/wire.rs vs Structure/Cost.hs")?;
    let rows = crate::lockstep::reply_rows::<Vec<[i64; 2]>>;
    // the A-layer keys exist exactly when a layout was declared —
    // a missing key on a declared request (or the reverse) is
    // contract drift, surfaced by the decode throat's named error
    let (divergence, deviations) = if r.declared.is_empty() {
        (None, Vec::new())
    } else {
        let div: Vec<i64> = crate::lockstep::reply_rows(&reply, "divergence")?;
        (div.first().copied(), rows(&reply, "deviations")?)
    };
    Ok(Reply {
        axes: rows(&reply, "axes")?,
        score: reply["score"].as_i64().context("score")?,
        entropy: rows(&reply, "entropy")?,
        findings: rows(&reply, "findings")?,
        knobs: rows(&reply, "knobs")?,
        divergence,
        deviations,
    })
}
