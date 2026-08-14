//! verdict/1 wire plumbing (design §2.2): ONE request carries the
//! whole fact table — tier universe, sim pairs, graph positions,
//! churn, cochange, continuous fingerprints, the discrete member
//! set, the baseline VERBATIM, and the floor. The reply's ratchet
//! and score come back raw; Rust never recomputes them (ADR-008).

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

pub const CAPABILITY: &str = "verdict/1";

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
    /// [axis, ceiling] rows from ce.toml (0 = size, 1 = coc):
    /// the config is the source, this wire is the road, and the
    /// core's Cost.hs values are DEFAULTS — no longer half of the
    /// uncheckable 300/15 mirror ADR-008 retired (audit D2).
    pub ceilings: Vec<[i64; 2]>,
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
    /// The EFFECTIVE [sizeCeil, cocCeil] the core judged with —
    /// echoed so judge() can assert the round trip.
    pub knobs: [i64; 2],
    pub degraded: Option<String>,
}

pub fn body(r: &Request) -> Value {
    json!({
        "sim": r.sim,
        "pos": r.pos,
        "tier": (0..r.files.len()).map(|u| [u as i64, 0]).collect::<Vec<_>>(),
        "churn": r.churn,
        "cochange": r.cochange,
        "continuous": r.continuous,
        "discrete": r.discrete,
        "baseline": r.baseline,
        // weights deliberately empty: equal weights are the decided
        // opening stance (decision ⑦) and live in the core's Cost
        "weights": [],
        "floor": r.floor,
        "ceilings": r.ceilings,
    })
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
    // round trip: every ceiling row sent must be the one judged with
    // (a degraded reply never judged — it echoes the defaults, and
    // degradation already fails the check upstream)
    if reply.degraded.is_none() {
        for &[axis, v] in &r.ceilings {
            let got = reply.knobs[axis as usize];
            anyhow::ensure!(
                got == v,
                "core judged with ceiling {got} on axis {axis}, ce sent {v}"
            );
        }
    }
    Ok(reply)
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
        knobs: {
            let k = rows("knobs")?;
            [
                k["sizeCeil"].as_i64().context("knobs.sizeCeil")?,
                k["cocCeil"].as_i64().context("knobs.cocCeil")?,
            ]
        },
        degraded: v["reason"].as_str().map(str::to_string),
    })
}
