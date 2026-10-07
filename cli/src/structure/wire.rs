//! structure/1 wire codec (contracts/fixtures/structure/golden.ndjson
//! is the byte-level contract; corelink stamps proto/type/id): the
//! request's path form out — since plan v2.33 W1 item 3 the core builds
//! the directory tree and the dir-keyed tables from the paths
//! (CE.Structure.Raw) — the judged axes / score / entropy / findings and
//! the tree it built back. This family keeps NO Rust verdict mirror by
//! design-booklet ruling (no frozen instrument needs one — the
//! review-repair C1 seam class is closed at the design table), so
//! the client parses and relays; a degraded reply to a client-sized
//! request is a cap-mirror drift error, never a judgment.

use anyhow::{Context, Result, ensure};
use serde_json::json;

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "structure/1";

/// The request's path form (the core builds the tree and the tables
/// from it, CE.Structure.Raw).
pub struct Request {
    /// The walked judged paths the tree is built over.
    pub paths: Vec<String>,
    /// The graph's measured files and their file-to-file arcs: the
    /// reference split (S3) and the crossing dir-edge table (7.1.0, O54)
    /// the core counts off them.
    pub arcs: super::rows::Arcs,
    /// ce.toml's [structure] layout (S3a), in key order. Empty = no
    /// declaration — the reply carries no A-layer keys at all.
    pub layout: Vec<(String, u32)>,
    /// The S5 staleness facts — None = axis 5 unjudged (no --days).
    pub stale: Option<super::rows::StaleDocs>,
    /// The S6 facts — None = axis 6 honestly unjudged (no --deep);
    /// Some with nothing in it = judged clean (absence vs zero).
    pub redundancy: Option<super::rows::Redundancy>,
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
    /// The tree the core built: [dirId, parent, depth, subdirs, files].
    pub tree: Vec<[i64; 5]>,
    /// Each directory's path by id, the root `.`.
    pub dirs: Vec<String>,
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

/// The request body: the paths and the graph's files and arcs every
/// request carries, then each optional table exactly when its
/// measurement rode — absence is the core's "unjudged", never a zero
/// (the layout / staleness / redundancy / seam semantics on `Request`).
pub(crate) fn request_body(r: &Request) -> serde_json::Value {
    let mut body = json!({
        "paths": r.paths,
        "refPaths": r.arcs.paths,
        "refPairs": r.arcs.files,
    });
    if !r.layout.is_empty() {
        body["layout"] = json!(r.layout);
    }
    if let Some(docs) = &r.stale {
        body["staleDocs"] = json!(docs);
    }
    if let Some((blocks, dead)) = &r.redundancy {
        body["clonePairs"] = json!(blocks);
        body["deadPaths"] = json!(dead);
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
    body
}

/// The path form's own answers, read before any verdict: a path the
/// core's tree could not place (its fault — the message this side
/// printed before the core built the tree), a request past the node cap
/// (the rows the core priced — CE.Structure.Cost.structNodeCap, refused
/// by name as this side refused it), and the tree with the shape echo a
/// core that built the tables answers (a core without the path form
/// refuses the request for its missing `nodes`, so a reply without them
/// is a skewed one); the tree rows and each directory's path handed back.
type Built = (Vec<[i64; 5]>, Vec<String>);
fn path_answers(reply: &serde_json::Value) -> Result<Built> {
    if let Some(fault) = reply["fault"].as_str() {
        anyhow::bail!("{fault}");
    }
    if reply["degraded"].as_bool() == Some(true) {
        let cap = crate::tables::get().limits.caps.structure_nodes;
        let priced = reply["priced"]
            .as_u64()
            .context("structure reply: degraded without its priced rows")?;
        anyhow::bail!("{priced} structure/1 request rows exceed the cap {cap}");
    }
    ensure!(
        reply["patternShapes"].as_u64().is_some() && reply["tree"].is_array(),
        "structure reply carries no built tree — a core without the path form (9.0.0)"
    );
    Ok((
        crate::lockstep::reply_rows(reply, "tree")?,
        crate::lockstep::reply_rows(reply, "dirs")?,
    ))
}

/// One structure.request over a fresh link.
pub fn judge(core: &str, r: &Request) -> Result<Reply> {
    judge_on(&mut crate::lockstep::open_family(core, CAP)?, r)
}

/// One structure.request over a link already open (the face keeps it
/// for its document, plan v2.32 step 4).
pub fn judge_on(link: &mut crate::corelink::Link, r: &Request) -> Result<Reply> {
    let reply = link
        .request("structure", request_body(r))
        .map_err(anyhow::Error::msg)?;
    let (tree, dirs) = path_answers(&reply)?;
    let rows = crate::lockstep::reply_rows::<Vec<[i64; 2]>>;
    let echoed = pinned_echo(&reply, &r.knobs)?;
    // the A-layer keys exist exactly when a layout was declared —
    // a missing key on a declared request (or the reverse) is
    // contract drift, surfaced by the decode throat's named error
    let (divergence, deviations) = if r.layout.is_empty() {
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
        tree,
        dirs,
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
