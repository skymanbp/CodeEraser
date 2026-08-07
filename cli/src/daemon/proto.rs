//! Client↔daemon wire protocol (ADR-003): NDJSON lines over a local
//! socket (Windows named pipe / Unix domain socket). SemVer major
//! mismatch makes the daemon reply `restart` and exit — the client
//! respawns it from its own (newer) binary.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Daemon protocol version — independent of the ce-core handshake
/// proto (contracts/VERSIONING.md governs both).
pub const DAEMON_PROTO: &str = "0.1.0";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// First line on every connection.
    Hello {
        proto: String,
    },
    Ping,
    /// Run the dedup pipeline on the daemon's project root.
    Dedup {
        min_tokens: Option<usize>,
        #[serde(default)]
        min_distinct: Option<usize>,
    },
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    HelloOk {
        proto: String,
        version: String,
    },
    /// Version skew: the daemon exits after this reply; the client
    /// respawns a fresh daemon from its own binary.
    Restart {
        reason: String,
    },
    Pong {
        uptime_ms: u64,
    },
    DedupReport {
        report: serde_json::Value,
    },
    Error {
        message: String,
    },
    Bye,
}

/// Socket name = hash of the canonicalized project root (one daemon
/// per project; credentials are the local user, per ADR-003).
pub fn socket_name(root: &Path) -> String {
    let canon = std::fs::canonicalize(root)
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string();
    format!(
        "ce-daemon-{:016x}",
        crate::dedup::tokens::fnv1a(canon.as_bytes())
    )
}

pub fn major(v: &str) -> Option<u64> {
    v.split('.').next().and_then(|s| s.parse().ok())
}
