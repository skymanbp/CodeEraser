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
    /// The S5 staleness join (S3c): [dirId, stale, total] rows —
    /// same absence semantics as redundancy, driven by --days.
    /// The raw S5 tables (2.23.0) — the predicate is the core's
    /// (deriveStale); the type lives with its producer.
    pub stale_docs: Option<super::rows::StaleTables>,
    /// The S6 rollup (S3b): [dirId, dupBlocks, deadUnits] rows.
    /// None = the table stays off the wire and axis 6 is honestly
    /// unjudged; Some(empty) = judged clean (absence vs zero).
    pub redundancy: Option<Vec<[u64; 3]>>,
    /// The split-ROI seam tables (plan v2.6 §C, 2.14.0). None = the
    /// advisory is not armed and the reply carries no split keys.
    pub seams: Option<SeamTables>,
    /// Knob rows for the core's table (codes per Structure/Knobs.hs).
    /// This channel was DEAD in Rust: the request never carried a
    /// `knobs` key, so the core priced every seam at its built-in
    /// 300/750 while the measurement selected seam files by the
    /// committed soft line — two different numbers, and Cost.hs's
    /// "the two families cannot disagree" comment was false as built.
    /// The measurement's own S and H now ride codes 12/13.
    pub knobs: Vec<[u64; 2]>,
}

/// files [fileId, total] / units [fileId, unit, start, end] /
/// refs [fileId, from, to] / clones [fileId, start, end] /
/// churn [fileId, a, b] — one named bundle (clippy's
/// type-complexity line agrees with the wire doc here). The two
/// v1.1 tables (2.15.0) price a seam's cut clone blocks and its
/// crossing co-change pairs. The MEASUREMENT (seams.rs) assembles
/// this same type in place — one shape, no mirror struct.
#[derive(Default, Clone)]
pub struct SeamTables {
    pub files: Vec<[u64; 2]>,
    pub units: Vec<[u64; 4]>,
    pub refs: Vec<[u64; 3]>,
    pub clones: Vec<[u64; 3]>,
    pub churn: Vec<[u64; 3]>,
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
    /// [fileId, afterUnit, benefitMilli, costMilli] — at most one
    /// viable seam per file; rows exist exactly when seams rode.
    pub split_candidates: Vec<[i64; 4]>,
    /// [fileId, bestBenefitMilli, bestCostMilli] — past the soft
    /// line with no viable seam (0/0 = no seam at all).
    pub size_exempt: Vec<[i64; 3]>,
}

/// Pin the echo the way every sibling family does (scan compares the
/// whole grade table, verdict asserts each sent knob round-trips,
/// trend pins the row count). structure decoded the table and read
/// one code — so a 2.14.0 core silently dropping seamClones/seamChurn
/// under the unknown-field rule answered with two of four legs
/// unpriced, no error, no degraded flag. The full table is a free
/// minor-version fingerprint; sent rows must echo.
fn pinned_echo(reply: &serde_json::Value, sent: &[[u64; 2]]) -> Result<Vec<[i64; 2]>> {
    let echoed: Vec<[i64; 2]> = crate::lockstep::reply_rows(reply, "knobs")?;
    for pair in sent.iter().map(|[c, v]| [*c as i64, *v as i64]) {
        ensure!(
            echoed.contains(&pair),
            "structure reply echo missing knob {}={} — a core minor too old for this request?",
            pair[0],
            pair[1]
        );
    }
    Ok(echoed)
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
    if let Some((docs, edges)) = &r.stale_docs {
        body["staleDocRows"] = json!(docs);
        body["staleEdgeRows"] = json!(edges);
    }
    if let Some(rows) = &r.redundancy {
        body["redundancy"] = json!(rows);
    }
    if let Some(s) = &r.seams {
        body["seamFiles"] = json!(s.files);
        body["seamUnits"] = json!(s.units);
        body["seamRefs"] = json!(s.refs);
        body["seamClones"] = json!(s.clones);
        body["seamChurn"] = json!(s.churn);
    }
    if !r.knobs.is_empty() {
        body["knobs"] = json!(r.knobs);
    }
    let reply = link
        .request("structure", body)
        .map_err(anyhow::Error::msg)?;
    crate::lockstep::refuse_degraded(&reply, "structure/wire.rs vs Structure/Cost.hs")?;
    let rows = crate::lockstep::reply_rows::<Vec<[i64; 2]>>;
    let echoed = pinned_echo(&reply, &r.knobs)?;
    // the A-layer keys exist exactly when a layout was declared —
    // a missing key on a declared request (or the reverse) is
    // contract drift, surfaced by the decode throat's named error
    let (divergence, deviations) = if r.declared.is_empty() {
        (None, Vec::new())
    } else {
        let div: Vec<i64> = crate::lockstep::reply_rows(&reply, "divergence")?;
        (div.first().copied(), rows(&reply, "deviations")?)
    };
    // the split keys ride exactly when the seam tables did (the
    // divergence precedent): decoding them on an unarmed request
    // would hide the contract drift a missing key means
    let (split_candidates, size_exempt) = if r.seams.is_some() {
        (
            crate::lockstep::reply_rows(&reply, "splitCandidates")?,
            crate::lockstep::reply_rows(&reply, "sizeExempt")?,
        )
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(Reply {
        axes: rows(&reply, "axes")?,
        score: reply["score"].as_i64().context("score")?,
        entropy: rows(&reply, "entropy")?,
        findings: rows(&reply, "findings")?,
        knobs: echoed,
        divergence,
        deviations,
        split_candidates,
        size_exempt,
    })
}
