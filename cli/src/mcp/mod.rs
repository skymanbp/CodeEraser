//! MCP server over stdio (M3, plan A8) — grown to the FULL read-only
//! report face at M7-P2 (charter ruling ③): every judgment family's
//! JSON report is a tool, and write verbs (baseline accept, config)
//! are deliberately absent — an agent must never re-baseline itself
//! through this surface. JSON-RPC 2.0, one JSON object per line,
//! shapes per MCP protocol revision 2024-11-05; the tool catalog is
//! ONE table (tools.rs) driving both tools/list and dispatch, so a
//! listed-but-undispatchable tool is unrepresentable.

mod tools;

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
    json!({"tools": tools::TOOLS.iter().map(tools::descriptor).collect::<Vec<_>>()})
}

fn tool_call(root: &Path, params: &Value) -> Result<Value> {
    let name = params["name"].as_str().unwrap_or_default();
    let args = &params["arguments"];
    let target = resolve(root, args["path"].as_str());
    let Some(tool) = tools::TOOLS.iter().find(|t| t.name == name) else {
        anyhow::bail!("unknown tool: {name}");
    };
    let text = (tool.run)(&target, args)?;
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
