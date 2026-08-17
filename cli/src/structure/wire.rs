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
}

/// The core's verdict, raw: nothing here is derived Rust-side.
pub struct Reply {
    pub axes: Vec<[i64; 2]>,
    pub score: i64,
    pub entropy: Vec<[i64; 2]>,
    /// Sparse [dirId, axis] drill-down rows.
    pub findings: Vec<[i64; 2]>,
    pub knobs: Vec<[i64; 2]>,
}

/// One structure.request over one link.
pub fn judge(core: &str, r: &Request) -> Result<Reply> {
    ensure!(
        r.nodes.len() <= STRUCT_NODE_CAP,
        "{} directory nodes exceed the structure/1 cap {STRUCT_NODE_CAP}",
        r.nodes.len()
    );
    let mut link = crate::lockstep::open_family(core, CAP)?;
    let body = json!({
        "nodes": r.nodes,
        "patterns": r.patterns,
        "conventions": r.conventions,
        "fileRefs": r.file_refs,
    });
    let reply = link
        .request("structure", body)
        .map_err(anyhow::Error::msg)?;
    crate::lockstep::refuse_degraded(&reply, "structure/wire.rs vs Structure/Cost.hs")?;
    let rows = |key: &str| crate::lockstep::reply_field(&reply, key);
    Ok(Reply {
        axes: serde_json::from_value(rows("axes")?).context("axes")?,
        score: reply["score"].as_i64().context("score")?,
        entropy: serde_json::from_value(rows("entropy")?).context("entropy")?,
        findings: serde_json::from_value(rows("findings")?).context("findings")?,
        knobs: serde_json::from_value(rows("knobs")?).context("knobs")?,
    })
}
