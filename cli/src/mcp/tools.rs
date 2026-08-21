//! The MCP tool catalog: one row per family — name, description,
//! extra input properties, report adapter. The table is the single
//! authority for BOTH tools/list and dispatch. Read-only by
//! construction (M7 charter ruling ③): every adapter ends at a
//! family's public report serialization; none touches baselines or
//! config, and the judgment families reuse the daemon's core-binary
//! resolver instead of growing a second one.

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

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "scan",
        desc: "Size/complexity/readability metrics report (ce.scan-report schema).",
        extra: &[],
        run: scan,
    },
    Tool {
        name: "check_duplication",
        desc: "Verified T1/T2 clone blocks (ce.dedup-report schema).",
        extra: &[("min_tokens", "integer", "report threshold (default 50)")],
        run: check_duplication,
    },
    Tool {
        name: "churn",
        desc: "Append-vs-rewrite and co-change over a git window (ce.churn-report schema).",
        extra: &[DAYS],
        run: churn,
    },
    Tool {
        name: "graph_sites",
        desc: "Reference sites, resolution-free (the ce graph --sites rows).",
        extra: &[],
        run: graph_sites,
    },
    Tool {
        name: "deadcode",
        desc: "Liveness verdicts over the reference graph (ce.deadcode-report schema).",
        extra: &[],
        run: deadcode,
    },
    Tool {
        name: "clone",
        desc: "T3 near-miss clone judgment (ce.clone-report schema).",
        extra: &[],
        run: clone_report,
    },
    Tool {
        name: "docdup",
        desc: "Documentation-duplication judgment (ce.docdup-report schema).",
        extra: &[],
        run: docdup,
    },
    Tool {
        name: "join",
        desc: "Three-signal join: similarity, graph position, churn (ce.join-report schema).",
        extra: &[DAYS],
        run: join,
    },
    Tool {
        name: "structure",
        desc: "Tree-scale structure judgment, seven axes (ce.structure-report schema).",
        extra: &[
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
        ],
        run: structure,
    },
    Tool {
        name: "check",
        desc: "ADR-006 baseline judgment: score, ratchet and axes (report only — \
               this surface never writes a baseline).",
        extra: &[],
        run: check,
    },
    Tool {
        name: "trend",
        desc: "Score trajectory over mainline git history (ce.trend-report schema; \
               points cache in the index, rebuildable from history).",
        extra: &[
            // the number here is pinned to trend::DEFAULT_COMMITS by
            // the test below — help prose that drifts from the code
            // is how this face came to answer 10 while the CLI said 30
            ("commits", "integer", "mainline window size (default 30)"),
            ("batch", "integer", "max uncached commits measured per call"),
        ],
        run: trend,
    },
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

/// CE_CORE_BIN → sibling → PATH: the daemon's resolver, one authority.
fn core() -> String {
    crate::daemon::judge::core_bin().unwrap_or_else(|| "ce-core".into())
}

/// A count arg, or the default — ONE guard for every window. `as u32`
/// TRUNCATED (`days = 4294967296` became 0 and judged a zero-day
/// window) and `commits` took `as usize` bare beside it, so a zero
/// there judged an EMPTY history: absent, unparsable, oversized and
/// zero all mean the default now.
fn count(args: &Value, key: &str, default: usize) -> usize {
    args[key]
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

fn days(args: &Value, default: u32) -> u32 {
    let v = count(args, "days", default as usize);
    u32::try_from(v).unwrap_or(default)
}

// Every adapter body below is one call into crate::faces — the ONE
// JSON face per family both machine surfaces consume (batch 4 lifted
// the bodies there when the GUI backend was about to copy them; the
// P4 cross-surface-shell precedent). This file keeps only the MCP
// concerns: the catalog table, arg parsing, and stringification.

// scan and graph_sites are the two STRING-native library faces, and
// mcp_precommit byte-pins this catalog to them (the catalog is a
// transport, never a second serializer) — a Value round-trip
// re-orders keys and drops the pretty form, so these two stay on the
// direct string faces; faces::scan/graph_sites serve the GUI's
// Value-shaped consumption of the same documents. Since batch-7
// slice 8 scan judges through the core like every verdict surface
// (the byte pin survives: findings ≡ mirror by the drift ensure).
fn scan(root: &Path, _a: &Value) -> Result<String> {
    let (files, findings, summary, _fail) = crate::scan::analyze_judged(root, &core())?;
    crate::scan::report_string(&files, &findings, summary)
}

fn check_duplication(root: &Path, a: &Value) -> Result<String> {
    let min = a["min_tokens"].as_u64().map(|v| v as usize);
    Ok(crate::faces::dedup(root, min)?.to_string())
}

fn churn(root: &Path, a: &Value) -> Result<String> {
    Ok(crate::faces::churn(root, days(a, 14))?.to_string())
}

fn graph_sites(root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::graph::sites_json(&crate::graph::analyze(root)?))
}

/// The plain judged faces (no extra knobs) share ONE adapter body —
/// the census caught their three shells chaining into clone blocks;
/// the shell exists once and thin per-name fns satisfy the table's
/// fn-pointer field (the pre-batch-4 banked shape, now over faces).
fn judged(root: &Path, which: &str) -> Result<String> {
    let core = core();
    let doc = match which {
        "deadcode" => crate::faces::deadcode(root, &core)?,
        "clone" => crate::faces::clone_t3(root, &core)?,
        "docdup" => crate::faces::docdup(root, &core)?,
        other => anyhow::bail!("not a plain judged face: {other}"),
    };
    Ok(doc.to_string())
}

fn deadcode(root: &Path, _a: &Value) -> Result<String> {
    judged(root, "deadcode")
}

fn clone_report(root: &Path, _a: &Value) -> Result<String> {
    judged(root, "clone")
}

fn docdup(root: &Path, _a: &Value) -> Result<String> {
    judged(root, "docdup")
}

fn join(root: &Path, a: &Value) -> Result<String> {
    Ok(crate::faces::join(root, &core(), days(a, 14))?.to_string())
}

fn structure(root: &Path, a: &Value) -> Result<String> {
    let deep = a["deep"].as_bool().unwrap_or(false);
    // absent = axis 5 unjudged (the honest Option), but a PRESENT and
    // unusable value must not truncate into a zero-day window
    let d = a["days"]
        .as_u64()
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX).max(1));
    // the split advisory joins the MCP face in v0.7 (plan v2.7 ③):
    // same report schema, same rows the CLI prints
    let split = a["split"].as_bool().unwrap_or(false);
    Ok(crate::faces::structure(root, &core(), (deep, d, split))?.to_string())
}

fn check(root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::faces::check(root, &core())?.to_string())
}

fn trend(root: &Path, a: &Value) -> Result<String> {
    let commits = count(a, "commits", crate::trend::DEFAULT_COMMITS);
    // absent = measure every uncached commit, but a PRESENT batch of 0
    // measured NOTHING and left `pending` pinned for a GUI polling it
    let batch = a["batch"]
        .as_u64()
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX).max(1));
    Ok(crate::faces::trend(root, &core(), commits, batch)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The trend window has ONE default (trend::DEFAULT_COMMITS): this
    /// face answered 10 while clap and the GUI answered 30, and the
    /// tools/list prose said 10 too. Both halves are pinned here.
    #[test]
    fn trend_window_default_is_the_shared_one() {
        let d = crate::trend::DEFAULT_COMMITS;
        let row = TOOLS
            .iter()
            .find(|t| t.name == "trend")
            .and_then(|t| t.extra.iter().find(|(n, ..)| *n == "commits"))
            .expect("the trend tool declares a commits arg");
        assert!(row.2.contains(&d.to_string()), "tools/list says: {}", row.2);
        assert_eq!(count(&json!({}), "commits", d), d, "absent = the default");
        assert_eq!(
            count(&json!({"commits": 0}), "commits", d),
            d,
            "0 is no window"
        );
    }
}
