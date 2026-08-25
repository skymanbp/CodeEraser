//! The `ce doctor` document (K round step 6) — the diagnostic as
//! DATA, so the console line, the GUI screen and the MCP surface are
//! three renderings of one measurement rather than three
//! measurements that agree most days (§5, no second report form).
//!
//! Every field is measured the way `ce doctor` measures it, which
//! for two of them means deliberately NOT the way SessionStart does:
//! the daemon probe never spawns and the index is peeked rather than
//! opened. A diagnostic that starts a daemon, or that rebuilds the
//! index it claims to be reporting on, is reporting a state it
//! created — the defect this module inherits the fix for.

use crate::corelink;
use crate::i18n::line;
use serde_json::{Value, json};
use std::path::Path;

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.doctor-report/0.1.0";

/// The whole diagnostic. Never returns an error: a core that will
/// not answer IS the finding, and a doctor that fails outward tells
/// the operator nothing about the thing they came to ask about.
pub fn document(root: &Path, core: &str) -> Value {
    let root = crate::root::project_root(root);
    let (degraded, entries) = super::degraded_runs(&root);
    let (version, proto, handshake, error) = match corelink::run(core) {
        Ok(reply) => (Some(reply.version), Some(reply.proto), true, Value::Null),
        Err(e) => (None, None, false, json!(e)),
    };
    json!({
        "schema": SCHEMA_ID,
        // the anchor, named: `ce doctor cli` reports the enclosing
        // project, and a silent re-root is half the defect
        "root": root.display().to_string(),
        "ce": {"version": env!("CARGO_PKG_VERSION"), "proto": corelink::PROTO},
        "guard": crate::config::tier_of(
            &crate::config::Config::load(&root),
            crate::config::PROMOTED_DEFAULT,
        ),
        "index": super::index_summary(&root),
        "daemon": super::daemon_status(&root),
        // the total frames the count: the feed is append-only, so a
        // degraded count alone never returns to zero after one
        // incident and reads as a live alarm forever
        "degradedRuns": {"degraded": degraded, "entries": entries},
        "core": {
            "version": version,
            "proto": proto,
            "handshake": handshake,
            "error": error,
        },
    })
}

/// The console RENDERING of the document above, moved here from
/// main_cmds.rs (K step 8) when that file passed its own 300-line
/// gate: a renderer belongs beside the measurement it renders, which
/// is the whole reason this module exists. Returns the lines to print
/// and the handshake bit, so the caller owns the exit code and stdout
/// / stderr split without this function knowing about either.
pub fn console(d: &Value) -> (Vec<String>, bool) {
    let s = |p: &str, k: &str| d[p][k].as_str().unwrap_or("?").to_string();
    let mut out = vec![
        format!("ce {} (proto {})", s("ce", "version"), s("ce", "proto")),
        line(
            "project: {} [ce {} | guard: {} | index: {} | daemon: {}]",
            "项目：{} 〔ce {} | 守卫：{} | 索引：{} | daemon：{}〕",
            &[
                &d["root"].as_str().unwrap_or("?"),
                &s("ce", "version"),
                &d["guard"].as_str().unwrap_or("?"),
                &d["index"].as_str().unwrap_or("?"),
                &d["daemon"].as_str().unwrap_or("?"),
            ],
        ),
        line(
            "degraded runs (observe feed): {} of {} entries",
            "降级运行（observe 流水）：{} / {} 条",
            &[
                &d["degradedRuns"]["degraded"],
                &d["degradedRuns"]["entries"],
            ],
        ),
    ];
    let ok = d["core"]["handshake"] == json!(true);
    if ok {
        out.push(format!(
            "ce-core {} (proto {})",
            s("core", "version"),
            s("core", "proto")
        ));
        // OK/FAILED is this diagnostic's own verdict, not the
        // exit-code vocabulary `check` borrows FAIL/pass from — a
        // reader of an otherwise Chinese report has no reason to
        // know either English word
        out.push(line("handshake: OK", "握手：正常", &[]));
    } else {
        out.push(line(
            "handshake: FAILED — {}",
            "握手：失败 — {}",
            &[&d["core"]["error"].as_str().unwrap_or("?")],
        ));
    }
    (out, ok)
}
