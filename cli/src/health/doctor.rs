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
pub const SCHEMA_ID: &str = "ce.doctor-report/0.3.0";

/// The whole diagnostic. Never returns an error: a core that will
/// not answer IS the finding, and a doctor that fails outward tells
/// the operator nothing about the thing they came to ask about.
pub fn document(root: &Path, core: &str) -> Value {
    let root = crate::root::project_root(root);
    let (degraded, entries) = super::degraded_runs(&root);
    let index = super::index_fact(&root);
    let daemon = super::daemon_fact(&root);
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
        // codes and counts, never sentences (plan v2.15): prose on
        // the machine face is prose no face can translate
        "index": {"state": index.0, "files": index.1},
        // 0.3.0 (O64): the client deadline's residue — workers the
        // deadline detached that have not returned (a connect the
        // kernel still holds); 0 in every healthy process, and the
        // one leak the deadline could not close, counted by name
        "daemon": {
            "state": daemon.0,
            "ms": daemon.1,
            "parkedWorkers": crate::daemon::cancel::PARKED.load(std::sync::atomic::Ordering::SeqCst),
        },
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

/// Read one `{state, <count>}` object back out of the document. The
/// console renders from the DOCUMENT, not from a second measurement —
/// that is the whole point of the document existing — so the facts
/// come back through this narrow door rather than being re-probed.
/// An unreadable state defaults to the WORST code, never to a healthy
/// one: a diagnostic that cannot read itself must not report health.
fn fact(v: &Value, count: &str) -> (i64, Option<i64>) {
    (v["state"].as_i64().unwrap_or(3), v[count].as_i64())
}

fn ms_fact(v: &Value) -> (i64, Option<u128>) {
    (
        v["state"].as_i64().unwrap_or(2),
        v["ms"].as_u64().map(u128::from),
    )
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
                &super::index_words(fact(&d["index"], "files")),
                &super::daemon_words(ms_fact(&d["daemon"])),
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
    // rendered only when non-zero: a gauge that reads 0 in every
    // healthy process is noise on the line and a finding otherwise
    if let Some(parked) = d["daemon"]["parkedWorkers"].as_u64().filter(|n| *n > 0) {
        out.push(line(
            "parked daemon workers: {} (past the client deadline, not returned)",
            "滞留的 daemon 工人线程：{} 条（客户端期限已过仍未返回）",
            &[&parked],
        ));
    }
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
