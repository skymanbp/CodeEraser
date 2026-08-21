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
    McpSession::over(&dir)
}

impl McpSession {
    fn over(dir: &std::path::Path) -> McpSession {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
            .arg("mcp")
            .arg(dir)
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
    // M7-P2 ruling ③: the full read-only report face — and nothing
    // with a write verb (no baseline, no config, no establish; the
    // trend cache is index-cache bookkeeping, not a write action).
    assert_eq!(
        names,
        [
            "scan",
            "check_duplication",
            "churn",
            "graph_sites",
            "deadcode",
            "clone",
            "docdup",
            "join",
            "structure",
            "check",
            "trend",
        ]
    );
    s.finish();
}

/// M7-P2 acceptance: each report tool's MCP text equals the SAME
/// report produced through the family's public library face — the
/// catalog is a transport, never a second serializer. The project is
/// git-seeded so the churn/join windows have history to read.
#[test]
fn mcp_report_faces_match_library() {
    let dir = tmp("mcp-faces");
    common::seed_clone_pair(&dir);
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "seed"]);
    // Steady-state the index once first: Summary carries refresh
    // counters (refreshed/removed/stale_skipped), so both faces must
    // observe the same warm index — otherwise this compares cache
    // states, not serializations.
    codeeraser::dedup::analyze(&dir, None, None, None).expect("warm");
    let mut s = McpSession::over(&dir);
    for (id, (name, args, want)) in library_reports(&dir).into_iter().enumerate() {
        let got = s.ask(serde_json::json!({
            "jsonrpc": "2.0", "id": 100 + id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        assert_eq!(got["result"]["isError"], false, "{name} errored: {got}");
        let text = got["result"]["content"][0]["text"].as_str().expect("text");
        assert_eq!(text, want, "{name} drifted from its library face");
    }
    s.finish();
}

/// The eleven expected report strings, each produced through the
/// public library face the MCP adapter claims to be a transport for —
/// rows built by one closure so the builder cannot re-grow parallel
/// row stanzas (the census caught two builders doing exactly that;
/// the over-50-line length is that finding's why — the 50-line split
/// WAS the twin-stanza clone, so one long builder is the honest
/// shape). The trend row runs library-first on purpose: it seeds the
/// cache, so the MCP call reads the same warm rows (the Summary
/// refresh-counter lesson, applied to the trend cache).
fn library_reports(dir: &std::path::Path) -> Vec<(&'static str, serde_json::Value, String)> {
    use codeeraser::{churn, dedup, docdup, graph, join, report, scan, score, structure, trend};
    let core = common::core_bin();
    let a = serde_json::json!({});
    let row = |name: &'static str, text: String| (name, a.clone(), text);
    let (files, findings, summary, _fail) = scan::analyze_judged(dir, &core).expect("scan");
    let (found, dsum) = dedup::analyze(dir, None, None, None).expect("dedup");
    let opts = score::Opts {
        db: None,
        core: core.clone(),
        days: None,
        floor: None,
        establish: false,
        pinned_soft: None,
    };
    vec![
        row(
            "scan",
            scan::report_string(&files, &findings, summary).expect("scan json"),
        ),
        row(
            "check_duplication",
            dedup::report_json(&found, &dsum).expect("dj").to_string(),
        ),
        row(
            "churn",
            churn::report_json(&churn::run(dir, 14).expect("churn")).to_string(),
        ),
        row(
            "graph_sites",
            graph::sites_json(&graph::analyze(dir).expect("sites")),
        ),
        row(
            "deadcode",
            report::deadcode_json(&graph::deadcode::run(dir, None, &core).expect("dead"))
                .to_string(),
        ),
        row(
            "clone",
            report::envelope(
                (dedup::t3::SCHEMA_ID, "clones"),
                &dedup::t3::run(dir, None, &core).expect("clone"),
            )
            .to_string(),
        ),
        row(
            "docdup",
            report::envelope(
                (docdup::judge::SCHEMA_ID, "dups"),
                &docdup::judge::run(dir, None, &core).expect("docdup"),
            )
            .to_string(),
        ),
        row(
            "join",
            join::report_json(&join::run(dir, None, &core, 14).expect("join")).to_string(),
        ),
        row(
            "structure",
            structure::judge::report_json(
                &structure::judge::run(dir, None, &core, (false, None, false)).expect("structure"),
            )
            .to_string(),
        ),
        row(
            "check",
            score::report_json(&score::run(dir, opts).expect("check")).to_string(),
        ),
        row(
            "trend",
            // commits=10 mirrors the MCP adapter's default exactly
            trend::report_json(&trend::run(dir, None, &core, 10, None).expect("trend")).to_string(),
        ),
    ]
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
