//! The MCP tool catalog: one row per family — name, description,
//! extra input properties, report adapter. The table is the single
//! authority for BOTH tools/list and dispatch. Read-only by
//! construction (M7 charter ruling ③): every adapter ends at a
//! family's public report serialization; none touches baselines or
//! config, and the judgment families reuse the daemon's core-binary
//! resolver instead of growing a second one.

use super::adapters::{
    check, check_duplication, churn, clone_report, deadcode, docdup, doctor, erase, graph_sites,
    join, scan, structure, trend, update_check,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

pub struct Tool {
    pub name: &'static str,
    pub desc: &'static str,
    /// Extra inputSchema properties beyond the shared `path`:
    /// (name, JSON type, description).
    pub extra: &'static [(&'static str, &'static str, &'static str)],
    pub run: fn(&Path, &Value) -> Result<String>,
}

const DAYS: (&str, &str, &str) = ("days", "integer", "git history window in days (default 14)");

/// One catalog row. The field names spelled out per tool were the
/// same eight tokens fifteen times over, and at thirteen tools this
/// repo's own clone gate counted three overlapping blocks in them —
/// the bite `face_cmd!` took in the GUI's command file, on the same
/// shape for the same reason. The table is still data; only the
/// punctuation left.
macro_rules! tool {
    ($name:literal, $run:ident, $desc:expr) => {
        tool!($name, $run, $desc, &[])
    };
    ($name:literal, $run:ident, $desc:expr, $extra:expr) => {
        Tool {
            name: $name,
            desc: $desc,
            extra: $extra,
            run: $run,
        }
    };
}

pub const TOOLS: &[Tool] = &[
    tool!(
        "scan",
        scan,
        "Size/complexity/readability metrics report (ce.scan-report schema)."
    ),
    tool!(
        "check_duplication",
        check_duplication,
        "Verified T1/T2 clone blocks (ce.dedup-report schema).",
        &[
            ("min_tokens", "integer", "report threshold (default 50)"),
            (
                "min_distinct",
                "integer",
                "diversity floor: suppress blocks with fewer distinct tokens",
            ),
        ]
    ),
    tool!(
        "churn",
        churn,
        "Append-vs-rewrite and co-change over a git window (ce.churn-report schema).",
        &[DAYS]
    ),
    tool!(
        "graph_sites",
        graph_sites,
        "Reference sites, resolution-free (the ce graph --sites rows)."
    ),
    tool!(
        "deadcode",
        deadcode,
        "Liveness verdicts over the reference graph, plus the symbol-level advisory of \
         declarations no other file spells — never a verdict (ce.deadcode-report schema)."
    ),
    tool!(
        "clone",
        clone_report,
        "T3 near-miss clone judgment (ce.clone-report schema); `units` lists the \
         cached unit universe instead.",
        &[(
            "units",
            "boolean",
            "list the unit universe (ce.clone-units schema) instead of judging",
        )]
    ),
    tool!(
        "docdup",
        docdup,
        "Documentation-duplication judgment (ce.docdup-report schema)."
    ),
    tool!(
        "join",
        join,
        "Three-signal join: similarity, graph position, churn (ce.join-report schema).",
        &[DAYS]
    ),
    tool!(
        "structure",
        structure,
        "Tree-scale structure judgment, seven axes (ce.structure-report schema).",
        &[
            ("deep", "boolean", "also judge the S6 redundancy axis"),
            (
                "days",
                "integer",
                "judge the S5 doc-staleness axis over this window",
            ),
            (
                "split",
                "boolean",
                "price the split-ROI advisory for files past the soft line",
            ),
        ]
    ),
    tool!(
        "check",
        check,
        "ADR-006 baseline judgment: score, ratchet and axes (report only — this \
         surface never writes a baseline).",
        &[(
            "floor",
            "integer",
            "arm the --fail-under score floor; absent = the ratchet alone judges",
        )]
    ),
    tool!(
        "erase",
        erase,
        "The deterministic erase PLAN (ce.erase-plan schema) — dry-run by \
         construction: this surface reaches the plan and nothing else, and \
         applying it is a human act at the CLI or the GUI."
    ),
    tool!(
        "doctor",
        doctor,
        "This machine's state (ce.doctor-report schema): ce-core handshake, guard \
         tier, index freshness, daemon, degraded-run counter. The daemon is asked \
         without being started."
    ),
    tool!(
        "trend",
        trend,
        "Score trajectory over mainline git history (ce.trend-report schema; points \
         cache in the index, rebuildable from history).",
        &[
            // the number here is pinned to trend::DEFAULT_COMMITS by
            // the test below — help prose that drifts from the code
            // is how this face came to answer 10 while the CLI said 30
            ("commits", "integer", "mainline window size (default 30)"),
            ("batch", "integer", "max uncached commits measured per call"),
        ]
    ),
    tool!(
        "update_check",
        update_check,
        "Whether a newer CodeEraser release exists (ce.update-report schema): the \
         latest tag, that tag's committed SHA256 pins, and the one action for this \
         install. Reads the release index over the network; places nothing — \
         applying is a human act at the CLI or the GUI."
    ),
];

/// tools/list descriptor for one row — `path` rides every schema.
pub fn descriptor(t: &Tool) -> Value {
    let mut props = serde_json::Map::new();
    props.insert(
        "path".into(),
        json!({"type": "string", "description": "subpath (default: project root)"}),
    );
    for (name, ty, desc) in t.extra {
        props.insert((*name).into(), json!({"type": ty, "description": desc}));
    }
    json!({
        "name": t.name,
        "description": t.desc,
        "inputSchema": {"type": "object", "properties": props},
    })
}

#[cfg(test)]
#[path = "../../tests/unit/mcp/tools.rs"]
mod tests;
