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
