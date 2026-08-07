//! MCP server round-trip (self-consistency; real-client compliance is
//! a 0.x preview acceptance item) + pre-commit gate e2e on a real
//! git repo with staged duplication.

use std::io::{BufRead, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn rust_fn(seed: u32) -> String {
    format!(
        "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
    )
}

#[test]
fn mcp_initialize_list_and_call_round_trip() {
    let dir = tmp("mcp-rt");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (T2 clone)");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("mcp")
        .arg(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut lines = BufReader::new(child.stdout.take().expect("stdout")).lines();
    let mut ask = |req: serde_json::Value| -> serde_json::Value {
        writeln!(stdin, "{req}").expect("write");
        stdin.flush().expect("flush");
        serde_json::from_str(&lines.next().expect("reply").expect("line")).expect("json")
    };
    let init = ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {}}
    }));
    assert_eq!(init["result"]["serverInfo"]["name"], "codeeraser");
    let list = ask(serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, ["scan", "check_duplication"]);
    let dup = ask(serde_json::json!({
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
    let scan = ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {"name": "scan", "arguments": {}}
    }));
    let stext = scan["result"]["content"][0]["text"].as_str().expect("text");
    let sreport: serde_json::Value = serde_json::from_str(stext).expect("scan json");
    assert_eq!(sreport["schema"], "ce.scan-report/0.1.0");
    // unknown method gets a JSON-RPC error, not a hang or crash
    let err = ask(serde_json::json!({"jsonrpc": "2.0", "id": 5, "method": "nope"}));
    assert_eq!(err["error"]["code"], -32601);
    drop(stdin); // EOF ends serve()
    let _ = child.wait();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
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
    let ok = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("precommit")
        .current_dir(&dir)
        .output()
        .expect("run");
    assert!(ok.status.success(), "clean staged change passes");
    // staged T2 clone blocks in deny mode
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    git(&dir, &["add", "b.rs"]);
    let blocked = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("precommit")
        .current_dir(&dir)
        .output()
        .expect("run");
    assert!(!blocked.status.success(), "deny mode must block");
    let text = String::from_utf8_lossy(&blocked.stdout);
    assert!(text.contains("b.rs"), "report names the clone: {text}");
}
