//! MCP server round-trip (self-consistency; real-client compliance is
//! a 0.x preview acceptance item) + pre-commit gate e2e on a real
//! git repo with staged duplication.

use std::io::{BufRead, BufReader, Write as _};
use std::process::{Command, Stdio};

mod common;
use common::{git, rust_fn, tmp};

/// Server over a seeded project + a request closure; EOF on drop.
struct McpSession {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    lines: std::io::Lines<BufReader<std::process::ChildStdout>>,
}

fn mcp_session(name: &str) -> McpSession {
    let dir = tmp(name);
    common::seed_clone_pair(&dir);
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("mcp")
        .arg(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mcp");
    let stdin = child.stdin.take().expect("stdin");
    let lines = BufReader::new(child.stdout.take().expect("stdout")).lines();
    McpSession {
        child,
        stdin,
        lines,
    }
}

impl McpSession {
    fn ask(&mut self, req: serde_json::Value) -> serde_json::Value {
        writeln!(self.stdin, "{req}").expect("write");
        self.stdin.flush().expect("flush");
        serde_json::from_str(&self.lines.next().expect("reply").expect("line")).expect("json")
    }
    fn finish(mut self) {
        drop(self.stdin); // EOF ends serve()
        let _ = self.child.wait();
    }
}

#[test]
fn mcp_initialize_and_list() {
    let mut s = mcp_session("mcp-init");
    let init = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {}}
    }));
    assert_eq!(init["result"]["serverInfo"]["name"], "codeeraser");
    let list = s.ask(serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, ["scan", "check_duplication"]);
    s.finish();
}

#[test]
fn mcp_tool_calls_and_unknown_method() {
    let mut s = mcp_session("mcp-tools");
    let dup = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "check_duplication", "arguments": {}}
    }));
    assert_eq!(dup["result"]["isError"], false);
    let text = dup["result"]["content"][0]["text"].as_str().expect("text");
    let report: serde_json::Value = serde_json::from_str(text).expect("report json");
    assert!(
        !report["blocks"].as_array().expect("blocks").is_empty(),
        "T2 pair must be reported over MCP"
    );
    let scan = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {"name": "scan", "arguments": {}}
    }));
    let stext = scan["result"]["content"][0]["text"].as_str().expect("text");
    let sreport: serde_json::Value = serde_json::from_str(stext).expect("scan json");
    assert_eq!(sreport["schema"], "ce.scan-report/0.1.0");
    // unknown method gets a JSON-RPC error, not a hang or crash
    let err = s.ask(serde_json::json!({"jsonrpc": "2.0", "id": 5, "method": "nope"}));
    assert_eq!(err["error"]["code"], -32601);
    s.finish();
}

/// A9f: a broken index degrades the precommit VISIBLY — exit 0 even
/// in deny (unverifiable state must not brick the commit), but the
/// human is told, never shown a clean pass.
#[test]
fn precommit_broken_index_is_visible_not_silent() {
    let dir = tmp("precommit-degraded");
    common::seed_git_clone_repo(&dir, "deny");
    common::corrupt_index(&dir);
    let out = common::run_ce(&dir, &["precommit"]);
    assert!(out.status.success(), "degraded fails open, never bricks");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("DEGRADED"),
        "degradation said out loud: {text}"
    );
    assert!(
        text.contains("staged file(s)"),
        "degraded line keeps the staged summary: {text}"
    );
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "precommit", "not mislabeled stop_audit");
    assert_eq!(line["degraded"], true);
}

#[test]
fn precommit_blocks_staged_duplication_in_deny() {
    let dir = tmp("precommit");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), "[guard]\nmode = \"deny\"\n").expect("ce.toml");
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "seed"]);
    // clean staged change passes
    std::fs::write(dir.join("c.rs"), "fn fresh(n: u8) -> u8 { n / 2 }\n").expect("c.rs");
    git(&dir, &["add", "c.rs"]);
    let ok = common::run_ce(&dir, &["precommit"]);
    assert!(ok.status.success(), "clean staged change passes");
    // staged T2 clone blocks in deny mode
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    git(&dir, &["add", "b.rs"]);
    let blocked = common::run_ce(&dir, &["precommit"]);
    assert!(!blocked.status.success(), "deny mode must block");
    let text = String::from_utf8_lossy(&blocked.stdout);
    assert!(text.contains("b.rs"), "report names the clone: {text}");
}
