//! verdict/1 wire plumbing (design §2.2): ONE request carries the
//! whole fact table — tier universe, sim pairs, graph positions,
//! churn, cochange, continuous fingerprints, the discrete member
//! set, the baseline VERBATIM, and the floor. The reply's ratchet
//! and score come back raw; Rust never recomputes them (ADR-008).

use crate::score::wire_check;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const CAPABILITY: &str = "verdict/1";

/// One knob table: [code, value] rows, code-ascending (the shared
/// wire grammar every knob family speaks).
pub type KnobTable = Vec<[i64; 2]>;

/// The assembled fact tables, index space = `files` (tier F, dense,
/// sorted — the caller builds it from the graph wire's own node
/// order so the two judgments share one universe).
pub struct Request {
    pub files: Vec<String>,
    pub sim: Vec<[i64; 5]>,
    pub pos: Vec<[i64; 6]>,
    pub churn: Vec<[i64; 3]>,
    pub cochange: Vec<[i64; 3]>,
    /// [entity, code, value, classId] — the class column rides the
    /// wire only when `classed` (3.1.0, plan v2.13 ①): a repo with no
    /// [[rules.class]] sends the three-column row it always did, so
    /// its bytes never move; a classed repo sends every row four
    /// wide (the core refuses a mixed table). The identity prefix
    /// stays (entity, code) either way — the ratchet never sees the
    /// column, and the baseline stays three columns forever.
    pub continuous: Vec<[u64; 4]>,
    pub discrete: Vec<u64>,
    pub baseline: Value,
    pub floor: Option<u32>,
    /// The four knob tables ce.toml speaks (ADR-008): ceilings
    /// axes 0/1 (the 27b9bc2 road; config is the source, Cost.hs
    /// values are DEFAULTS), and the P4 trio — weights [axis, w]
    /// (the deliberate always-empty array retired; [score.weights]
    /// drives it), thresholds codes 0..7, tolerance legs 0..2.
    /// score::knobs owns every code registry.
    pub ceilings: KnobTable,
    pub weights: KnobTable,
    pub thresholds: KnobTable,
    pub tolerance: KnobTable,
    /// The dedup budget pair [blocks, budget] (ADR-008 P2): sent by
    /// `ce dedup --check` alone — the second ratchet's comparison
    /// is the core's. None = the condition is not evaluated, which
    /// keeps the ce check/baseline road byte-identical.
    pub dedup: Option<[u64; 2]>,
    /// batch-7 slice 1 (2.19.0): the PRE-filter per-block distinct
    /// counts, riding beside the pair — the core re-derives the
    /// admitted count with CE.Dedup.Cost.minDistinct and judges the
    /// budget from ITS number; check() proves the local filter equal
    /// against the echoed dedupBlocks. Empty = the pair-only road.
    pub dedup_distinct: Vec<u64>,
    /// The effective floor, sent only when the CLI overrode
    /// --min-distinct (absent = the core default judges).
    pub dedup_min_distinct: Option<u64>,
    /// The judged-language file-LOC multiset (2.14.0, plan v2.6 §B):
    /// values only, non-descending — what the core derives the soft
    /// line from AT ESTABLISH. Empty = no derivable S.
    pub judged_loc: Vec<u64>,
    /// Markdown/documentation file indices in the same `files` universe.
    /// Empty preserves the pre-2.27 cycle-axis semantics.
    pub doc_files: Vec<i64>,
    /// The judged-language set as a Lang-code bitmask (H1 slice 2,
    /// 2.29.0): batch-7 dispositioned the PREDICATE to Rust and
    /// promised the SET as an echo-pinned knob — this is that knob.
    /// 0 = not declared (the dedup-only road); the echo pins the
    /// round trip so the core can see and ablate the set.
    pub judged_mask: i64,
    /// The rulepack channel (3.1.0): `classed` = any class declared,
    /// so every continuous row rides four wide; `class_knobs` =
    /// [classId, code, value], the ceilings codes 0/1/2 under a class
    /// (score::knobs::class_knob_rows), empty = no override rides.
    pub classed: bool,
    pub class_knobs: Vec<[i64; 3]>,
    /// The rulepack FINGERPRINT (5.1.0), None when no class is
    /// declared. The baseline records the digest its ceilings were
    /// established under, and the core fails by name when the two
    /// disagree — so a glob edit stops being a silent way to move
    /// every line at once (config::RulesCfg::digest).
    pub knobs_digest: Option<u64>,
    /// The export surface (6.1.0): `[u, visibility]` in the `files`
    /// universe, deduped and ascending — graph/1's `symbols` table
    /// re-keyed, not re-judged. The raw visibility word travels so
    /// that which bit means "exported" stays the core's call
    /// (Graph.Cost.exportVisBit); the lattice's RG10 guard reads the
    /// flag bit derived from it. Empty = the legacy road.
    pub symbols: Vec<[i64; 2]>,
    /// The provenance table (6.4.0, O40): ascending file entities on
    /// disk under the scope that own no continuous row this run —
    /// the walk's own candidate set read with no ignore file and no
    /// exclude (score::provenance), minus the measured set. Some on
    /// every check road, an empty table included (the reply then
    /// still answers `dropped`, which is how a pre-6.4.0 core is told
    /// apart from a clean tree); None on the dedup-only and join
    /// roads, which carry no baseline for it to be read against.
    pub present: Option<Vec<u64>>,
    /// The self-loop table (6.4.0, O59): verdict-universe indices
    /// carrying an exact self-arc, projected from the graph reply's
    /// singleton cycles. Rides exactly when `[graph] scc_floor = 1`
    /// — the core requires it at cycleFloor 1 and refuses it elsewhere.
    pub cycle_self_loops: Option<Vec<i64>>,
}

/// The core's verdict, raw: nothing here is derived Rust-side.
pub struct Reply {
    /// [u, v, code, reasons, legsMask, confidence] — the 6th column
    /// is the leg-agreement confidence (2.33.0, H4).
    pub candidates: Vec<[i64; 6]>,
    /// The verdict table's (code, severity) face, shipped once
    /// (2.33.0) — the faces rank with the core's numbers.
    pub join_severity: Vec<[i64; 2]>,
    pub score: i64,
    pub axes: Vec<[i64; 2]>,
    pub added: Vec<u64>,
    pub removed: Vec<u64>,
    pub over: Vec<[u64; 4]>,
    pub tolerance_drawn: Vec<[u64; 3]>,
    pub fail: bool,
    /// The HELD fail-condition names (review C8, 2.8.0): consumers
    /// attribute the fail bit by name, never by construction-time
    /// coincidence.
    pub failed: Vec<String>,
    pub new_baseline: Value,
    /// The FULL effective-knob echo (ADR-008 P4) — every key the
    /// core judged with, so judge() asserts the round trip and the
    /// empty-table drift gate pins the defaults.
    pub knobs: BTreeMap<String, i64>,
    /// The effective per-axis weight table 0..6 (review C3, 2.8.0):
    /// the one knob family that had no round trip until the panel
    /// caught the no-op golden covering for it.
    pub weights: Vec<[i64; 2]>,
    /// The class knob rows the core judged with (3.1.0), echoed
    /// exactly when they rode — empty on a legacy reply.
    pub class_knobs: Vec<[i64; 3]>,
    /// The core's own admitted-block count (2.19.0), None when the
    /// distinct rows did not ride — check() proves the local filter
    /// equal against it.
    pub dedup_blocks: Option<u64>,
    /// The committed rows an exclusion explains (6.4.0, O40): [entity,
    /// code, committed value] for every baseline row whose file is in
    /// the `present` table and whose (entity, code) this run did not
    /// measure. Some exactly when `present` rode (an empty table on a
    /// clean tree); None on the roads that sent none. `rows_dropped`
    /// is its fail name.
    pub dropped: Option<Vec<[u64; 3]>>,
    pub degraded: Option<String>,
}

impl Request {
    /// The empty-tables request `ce dedup --check` sends (ADR-008
    /// P2): nothing to score, no baseline, just the pair — the
    /// reply's fail bit is the whole judgment.
    pub fn dedup_only(blocks: u64, budget: u64, distincts: Vec<u64>, floor: Option<u64>) -> Self {
        Request {
            files: Vec::new(),
            sim: Vec::new(),
            pos: Vec::new(),
            symbols: Vec::new(),
            present: None,
            cycle_self_loops: None,
            churn: Vec::new(),
            cochange: Vec::new(),
            continuous: Vec::new(),
            classed: false,
            class_knobs: Vec::new(),
            knobs_digest: None,
            discrete: Vec::new(),
            baseline: Value::Null,
            floor: None,
            ceilings: Vec::new(),
            weights: Vec::new(),
            thresholds: Vec::new(),
            tolerance: Vec::new(),
            dedup: Some([blocks, budget]),
            dedup_distinct: distincts,
            dedup_min_distinct: floor,
            judged_loc: Vec::new(),
            doc_files: Vec::new(),
            judged_mask: 0,
        }
    }
}

/// A table's rows, or None when it has none: an empty optional table
/// and an absent one are the same wire fact, and the difference
/// between them is a byte a legacy request never sent.
fn some_rows<T: serde::Serialize>(rows: &[T]) -> Option<Value> {
    (!rows.is_empty()).then(|| json!(rows))
}

pub fn body(r: &Request) -> Value {
    // the class column rides only on a classed run — a legacy
    // request is the three-column row, byte for byte (3.1.0 C1)
    let continuous: Vec<Vec<u64>> = r
        .continuous
        .iter()
        .map(|row| row[..if r.classed { 4 } else { 3 }].to_vec())
        .collect();
    let mut o = json!({
        "sim": r.sim,
        "pos": r.pos,
        "tier": (0..r.files.len()).map(|u| [u as i64, 0]).collect::<Vec<_>>(),
        "churn": r.churn,
        "cochange": r.cochange,
        "continuous": continuous,
        "discrete": r.discrete,
        "baseline": r.baseline,
        "floor": r.floor,
    });
    // every key that rides CONDITIONALLY rides one table (the loop
    // below already did this for the four knob tables; the ifs around
    // it were the same statement written eight more times). Absent is
    // not null and not []: a repo without a feature must send the
    // bytes it sent before the feature existed, which is what every
    // "absent = byte-identical" counterfactual in VERSIONING rests on.
    let optional = [
        ("classKnobs", some_rows(&r.class_knobs)),
        // the fence rides whenever a rulepack is declared, even one
        // whose knobs are all defaults: what it fences is the
        // DECLARATION, and a class with no knobs still decides which
        // files it owns
        ("knobsDigest", r.knobs_digest.map(|d| json!(d))),
        ("dedup", r.dedup.map(|p| json!(p))),
        ("dedupDistinct", some_rows(&r.dedup_distinct)),
        ("dedupMinDistinct", r.dedup_min_distinct.map(|f| json!(f))),
        (
            "judgedMask",
            (r.judged_mask != 0).then(|| json!(r.judged_mask)),
        ),
        ("docFiles", some_rows(&r.doc_files)),
        ("symbols", some_rows(&r.symbols)),
        // the two 6.4.0 tables ride as OPTIONS, not as some_rows: an
        // empty present table is a fact (every candidate measured)
        // the reply must answer, and an absent one is a road with no
        // baseline; the self-loop table is required at floor 1 even
        // when empty and refused anywhere else
        ("present", r.present.as_ref().map(|p| json!(p))),
        (
            "cycleSelfLoops",
            r.cycle_self_loops.as_ref().map(|l| json!(l)),
        ),
    ];
    for (key, value) in optional.into_iter().flat_map(|(k, v)| v.map(|v| (k, v))) {
        o[key] = value;
    }
    // the four knob tables ride unconditionally, so they keep their
    // own loop rather than pretending to be optional
    for (key, rows) in [
        ("ceilings", &r.ceilings),
        ("weights", &r.weights),
        ("thresholds", &r.thresholds),
        ("tolerance", &r.tolerance),
    ] {
        o[key] = json!(rows);
    }
    o["judgedLoc"] = json!(r.judged_loc);
    o
}

/// One verdict.request over the open core link; a missing capability
/// or a non-result reply is an error, never an empty judgment.
pub fn judge(core: &str, r: &Request) -> Result<Reply> {
    let mut link = crate::lockstep::open_family(core, CAPABILITY)?;
    let reply = link
        .request("verdict", body(r))
        .map_err(anyhow::Error::msg)?;
    let reply = parse(&reply)?;
    // every invariant a reply must hold, degraded or not — the knob
    // echoes, the fail/failed law, the fence policy, the newBaseline
    // shape, the provenance answer (wire_check, O32 / 6.4.0)
    wire_check::check_reply(r, &reply)?;
    Ok(reply)
}

/// The ratchet sub-object's seven fields, decoded once (split from
/// parse() when the 2.8.0 `failed` row pushed it past the repo's
/// own cyclomatic gate).
struct RatchetEcho {
    added: Vec<u64>,
    removed: Vec<u64>,
    over: Vec<[u64; 4]>,
    tolerance_drawn: Vec<[u64; 3]>,
    fail: bool,
    failed: Vec<String>,
    dropped: Option<Vec<[u64; 3]>>,
}

fn ratchet_of(r: &Value) -> Result<RatchetEcho> {
    use crate::lockstep::reply_rows as rows;
    Ok(RatchetEcho {
        added: rows(r, "added")?,
        removed: rows(r, "removed")?,
        over: rows(r, "over")?,
        tolerance_drawn: rows(r, "toleranceDrawn")?,
        fail: r["fail"].as_bool().context("fail")?,
        failed: rows(r, "failed")?,
        // absent on a pre-6.4.0 reply and on one whose request sent
        // no present table — wire_check tells the two apart
        dropped: if r["dropped"].is_null() {
            None
        } else {
            Some(rows(r, "dropped")?)
        },
    })
}

fn parse(v: &Value) -> Result<Reply> {
    use crate::lockstep::reply_rows as rows;
    let r = ratchet_of(&crate::lockstep::reply_field(v, "ratchet")?)?;
    Ok(Reply {
        candidates: rows(v, "candidates")?,
        join_severity: rows(v, "joinSeverity")?,
        score: v["score"].as_i64().context("score")?,
        axes: rows(v, "axes")?,
        added: r.added,
        removed: r.removed,
        over: r.over,
        tolerance_drawn: r.tolerance_drawn,
        fail: r.fail,
        failed: r.failed,
        dropped: r.dropped,
        new_baseline: crate::lockstep::reply_field(v, "newBaseline")?,
        knobs: rows(v, "knobs")?,
        weights: rows(v, "weights")?,
        // absent on a legacy reply (the key rides only when rows did)
        class_knobs: if v["classKnobs"].is_null() {
            Vec::new()
        } else {
            rows(v, "classKnobs")?
        },
        dedup_blocks: v["dedupBlocks"].as_u64(),
        degraded: degraded_of(v),
    })
}

/// The AUTHORITATIVE degraded bit, with reason as its label (review
/// C9: deriving from reason presence let a reasonless degraded
/// reply pass as judged; split keeps parse() under the cyclomatic
/// gate).
fn degraded_of(v: &Value) -> Option<String> {
    if v["degraded"] == Value::Bool(true) {
        Some(v["reason"].as_str().unwrap_or("degraded").to_string())
    } else {
        None
    }
}

#[cfg(test)]
#[path = "../../tests/unit/score/wire.rs"]
mod tests;
