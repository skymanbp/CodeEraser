//! MCP server over stdio (M3, plan A8) — grown to the FULL read-only
//! report face at M7-P2 (charter ruling ③): every judgment family's
//! JSON report is a tool, and write verbs (baseline accept, config)
//! are deliberately absent — an agent must never re-baseline itself
//! through this surface. JSON-RPC 2.0, one JSON object per line,
//! shapes per MCP protocol revision 2024-11-05; the tool catalog is
//! ONE table (tools.rs) driving both tools/list and dispatch, so a
//! listed-but-undispatchable tool is unrepresentable.

mod adapters;
mod tools;

use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

/// Longest frame we will buffer. Requests on this surface carry a
/// subpath and a few scalars — never file contents — so 1 MiB is
/// already absurd headroom; without a ceiling a client that never
/// sends a newline streams stdin straight into our heap. A frame that
/// hits the cap cannot be resynchronized (its tail is still arriving
/// mid-object), so we refuse it and stop, the way the daemon closes an
/// over-cap connection rather than parsing the remainder as a new line
/// (daemon/server/conn.rs, read_line_capped's over-cap arm).
const FRAME_MAX: u64 = 1 << 20;

pub fn serve(root: &Path) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    serve_stream(root, stdin.lock(), &mut stdout)
}

/// The stdio loop over any transport, so the frame rules below are
/// reachable from a unit test instead of only from a real stdin.
fn serve_stream(root: &Path, mut input: impl BufRead, out: &mut impl Write) -> Result<()> {
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let n = (&mut input).take(FRAME_MAX).read_until(b'\n', &mut buf)?;
        if n == 0 {
            return Ok(()); // stdin closed
        }
        if !buf.ends_with(b"\n") && n as u64 == FRAME_MAX {
            let text = format!("frame exceeds the {FRAME_MAX}-byte ceiling");
            emit(out, &error(Value::Null, -32600, &text))?;
            return Ok(());
        }
        // bytes first, lossy second: `Lines` surfaced one stray non-UTF-8
        // byte as an InvalidData error that `?` propagated, so a single
        // bad byte on stdin killed the whole server. A malformed frame is
        // the same 坏行 class as bad JSON — answer it, keep serving; the
        // daemon transport already ruled this way
        // (daemon/server/conn.rs:46-51 `坏行`).
        let decoded = String::from_utf8_lossy(&buf);
        let line = decoded.trim();
        if line.is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(line) {
            Ok(msg) => handle(root, &msg),
            // JSON-RPC 2.0 §5.1: an unparseable frame is -32700, not
            // silence. Dropping it left a client whose frame got mangled
            // in transit blocking forever on an id we never answered.
            Err(e) => Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        if let Some(reply) = reply {
            emit(out, &reply)?;
        }
    }
}

fn emit(out: &mut impl Write, frame: &Value) -> Result<()> {
    writeln!(out, "{frame}")?;
    out.flush()?;
    Ok(())
}

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// Genuine notifications (no id) get no reply; everything CARRYING an
/// id gets one. The id is read FIRST because reading `method` first
/// made a frame with a good id and a missing or non-string method
/// return None — the client blocks on that id forever, while an
/// unknown *string* method already got its -32601. Silence there was
/// an oversight, and JSON-RPC 2.0 §4.2 calls it -32600.
fn handle(root: &Path, msg: &Value) -> Option<Value> {
    let id = msg.get("id").filter(|v| !v.is_null())?.clone();
    let Some(method) = msg["method"].as_str() else {
        return Some(error(id, -32600, "invalid request: no string \"method\""));
    };
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
            let text = format!("method not found: {method}");
            return Some(error(id, -32601, &text));
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
/// walk arbitrary directories through us. The test itself lives in
/// scan::walk beside the other path-vocabulary rules, so the daemon's
/// socket surface answers to the same authority.
fn resolve(root: &Path, sub: Option<&str>) -> PathBuf {
    let Some(sub) = sub.filter(|s| !s.is_empty()) else {
        return root.to_path_buf();
    };
    crate::scan::walk::contained(root, sub).unwrap_or_else(|| root.to_path_buf())
}

#[cfg(test)]
#[path = "../../tests/unit/mcp.rs"]
mod tests;
