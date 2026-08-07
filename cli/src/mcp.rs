//! Minimal MCP server over stdio (M3, plan A8: sit exactly where
//! jscpd already lives for agent ecosystems). JSON-RPC 2.0, one JSON
//! object per line. Two tools: `scan` (metrics report) and
//! `check_duplication` (verified clone blocks). Shapes follow MCP
//! protocol revision 2024-11-05; self-consistency is pinned by the
//! round-trip test, and real-client compliance is an explicit 0.x
//! preview acceptance item (M3) — not claimed here.

use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub fn serve(root: &Path) -> Result<()> {
    let root = root.to_path_buf();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue; // not JSON: ignore, keep serving
        };
        if let Some(reply) = handle(&root, &msg) {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// Notifications (no id) get no reply; unknown methods get an error.
fn handle(root: &Path, msg: &Value) -> Option<Value> {
    let method = msg["method"].as_str()?;
    let id = msg.get("id").filter(|v| !v.is_null())?.clone();
    let result = match method {
        "initialize" => initialize_result(),
        "tools/list" => tools_list(),
        "tools/call" => match tool_call(root, &msg["params"]) {
            Ok(r) => r,
            Err(e) => {
                return Some(json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{"type": "text", "text": format!("{e:#}")}],
                        "isError": true,
                    }
                }));
            }
        },
        _ => {
            return Some(json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": -32601, "message": format!("method not found: {method}")}
            }));
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {"tools": {}},
        "serverInfo": {"name": "codeeraser", "version": env!("CARGO_PKG_VERSION")},
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "scan",
                "description": "Size/complexity/readability metrics report \
                    (ce.scan-report schema) for the project or a subpath.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "subpath (default: project root)"}
                    }
                }
            },
            {
                "name": "check_duplication",
                "description": "Verified T1/T2 clone blocks \
                    (ce.dedup-report schema) for the project or a subpath.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "subpath (default: project root)"},
                        "min_tokens": {"type": "integer", "description": "report threshold (default 50)"}
                    }
                }
            }
        ]
    })
}

fn tool_call(root: &Path, params: &Value) -> Result<Value> {
    let name = params["name"].as_str().unwrap_or_default();
    let args = &params["arguments"];
    let target = resolve(root, args["path"].as_str());
    let text = match name {
        "scan" => scan_json(&target)?,
        "check_duplication" => {
            let min = args["min_tokens"].as_u64().map(|v| v as usize);
            let (found, summary) = crate::dedup::analyze(&target, None, min, None)?;
            crate::dedup::report_json(&found, &summary)?.to_string()
        }
        other => anyhow::bail!("unknown tool: {other}"),
    };
    Ok(json!({"content": [{"type": "text", "text": text}], "isError": false}))
}

/// Subpaths stay inside the served root — an MCP client must not
/// walk arbitrary directories through us.
fn resolve(root: &Path, sub: Option<&str>) -> PathBuf {
    let Some(sub) = sub.filter(|s| !s.is_empty()) else {
        return root.to_path_buf();
    };
    let joined = root.join(sub);
    let canon_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    match std::fs::canonicalize(&joined) {
        Ok(c) if c.starts_with(&canon_root) => joined,
        _ => root.to_path_buf(),
    }
}

/// The scan report as a JSON string (same library pipeline and
/// schema as `ce scan --format json`).
fn scan_json(root: &Path) -> Result<String> {
    let (files, findings, summary) = crate::scan::analyze(root)?;
    crate::scan::report_string(&files, &findings, summary)
}
