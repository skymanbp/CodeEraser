//! verdict/1 wire plumbing (design §2.2): ONE request carries the
//! whole fact table — tier universe, sim pairs, graph positions,
//! churn, cochange, continuous fingerprints, the discrete member
//! set, the baseline VERBATIM, and the floor. The reply's ratchet
//! and score come back raw; Rust never recomputes them (ADR-008).

use crate::score::knobs;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const CAPABILITY: &str = "verdict/1";

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
    pub churn: Vec<[i64; 5]>,
    pub cochange: Vec<[i64; 3]>,
    pub continuous: Vec<[u64; 3]>,
    pub discrete: Vec<u64>,
    pub baseline: Value,
    pub floor: Option<u32>,
    /// The four knob tables ce.toml speaks (ADR-008): ceilings
    /// axes 0/1 (the 27b9bc2 road; config is the source, Cost.hs
    /// values are DEFAULTS), and the P4 trio — weights [axis, w]
    /// (the deliberate always-empty array retired; [score.weights]
    /// drives it), thresholds codes 0..6, tolerance legs 0..2.
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
}

/// The core's verdict, raw: nothing here is derived Rust-side.
pub struct Reply {
    pub candidates: Vec<[i64; 5]>,
    pub score: i64,
    pub axes: Vec<[i64; 2]>,
    pub added: Vec<u64>,
    pub removed: Vec<u64>,
    pub over: Vec<[u64; 4]>,
    pub tolerance_drawn: Vec<[u64; 3]>,
    pub fail: bool,
    pub new_baseline: Value,
    /// The FULL effective-knob echo (ADR-008 P4) — every key the
    /// core judged with, so judge() asserts the round trip and the
    /// empty-table drift gate pins the defaults.
    pub knobs: BTreeMap<String, i64>,
    pub degraded: Option<String>,
}

impl Request {
    /// The empty-tables request `ce dedup --check` sends (ADR-008
    /// P2): nothing to score, no baseline, just the pair — the
    /// reply's fail bit is the whole judgment.
    pub fn dedup_only(blocks: u64, budget: u64) -> Self {
        Request {
            files: Vec::new(),
            sim: Vec::new(),
            pos: Vec::new(),
            churn: Vec::new(),
            cochange: Vec::new(),
            continuous: Vec::new(),
            discrete: Vec::new(),
            baseline: Value::Null,
            floor: None,
            ceilings: Vec::new(),
            weights: Vec::new(),
            thresholds: Vec::new(),
            tolerance: Vec::new(),
            dedup: Some([blocks, budget]),
        }
    }
}

pub fn body(r: &Request) -> Value {
    let mut o = json!({
        "sim": r.sim,
        "pos": r.pos,
        "tier": (0..r.files.len()).map(|u| [u as i64, 0]).collect::<Vec<_>>(),
        "churn": r.churn,
        "cochange": r.cochange,
        "continuous": r.continuous,
        "discrete": r.discrete,
        "baseline": r.baseline,
        "floor": r.floor,
    });
    // the four knob tables ride one loop — table-driven at the
    // assembly site too, not just in the core's evaluator
    for (key, rows) in [
        ("ceilings", &r.ceilings),
        ("weights", &r.weights),
        ("thresholds", &r.thresholds),
        ("tolerance", &r.tolerance),
    ] {
        o[key] = json!(rows);
    }
    if let Some(pair) = r.dedup {
        o["dedup"] = json!(pair);
    }
    o
}

/// One verdict.request over the open core link; a missing capability
/// or a non-result reply is an error, never an empty judgment.
pub fn judge(core: &str, r: &Request) -> Result<Reply> {
    let (mut link, _hello) = crate::corelink::Link::open(core).map_err(anyhow::Error::msg)?;
    if !link.has(CAPABILITY) {
        bail!("ce-core offers no {CAPABILITY} capability — upgrade the core");
    }
    let reply = link
        .request("verdict", body(r))
        .map_err(anyhow::Error::msg)?;
    if reply["type"] != json!("verdict.result") {
        bail!("core replied {}: {reply}", reply["type"]);
    }
    let reply = parse(&reply)?;
    // round trip: every knob row sent must be the one judged with
    // (a degraded reply never judged — it echoes the defaults, and
    // degradation already fails the check upstream). weights have
    // no echo key; their lever is pinned by the core battery and
    // the golden pair that sends one.
    if reply.degraded.is_none() {
        assert_echo(&knobs::CEILING_KEYS, &r.ceilings, &reply.knobs)?;
        assert_echo(&knobs::THRESHOLD_KEYS, &r.thresholds, &reply.knobs)?;
        assert_echo(&knobs::TOLERANCE_KEYS, &r.tolerance, &reply.knobs)?;
    }
    Ok(reply)
}

/// Sent [code, value] rows against the reply's knob echo, names
/// resolved through the SAME index-is-code registry that assembled
/// them (score::knobs).
fn assert_echo(keys: &[&str], sent: &[[i64; 2]], got: &BTreeMap<String, i64>) -> Result<()> {
    for &[code, v] in sent {
        let key = keys
            .get(code as usize)
            .with_context(|| format!("no echo key for knob code {code}"))?;
        let echoed = got
            .get(*key)
            .with_context(|| format!("reply echo missing {key}"))?;
        anyhow::ensure!(*echoed == v, "core judged with {key}={echoed}, ce sent {v}");
    }
    Ok(())
}

fn parse(v: &Value) -> Result<Reply> {
    let rows = |key: &str| -> Result<Value> {
        v.get(key)
            .cloned()
            .with_context(|| format!("verdict.result missing {key}"))
    };
    let ratchet = rows("ratchet")?;
    Ok(Reply {
        candidates: serde_json::from_value(rows("candidates")?).context("candidates")?,
        score: v["score"].as_i64().context("score")?,
        axes: serde_json::from_value(rows("axes")?).context("axes")?,
        added: serde_json::from_value(ratchet["added"].clone()).context("added")?,
        removed: serde_json::from_value(ratchet["removed"].clone()).context("removed")?,
        over: serde_json::from_value(ratchet["over"].clone()).context("over")?,
        tolerance_drawn: serde_json::from_value(ratchet["toleranceDrawn"].clone())
            .context("toleranceDrawn")?,
        fail: ratchet["fail"].as_bool().context("fail")?,
        new_baseline: rows("newBaseline")?,
        knobs: serde_json::from_value(rows("knobs")?).context("knobs")?,
        degraded: v["reason"].as_str().map(str::to_string),
    })
}
