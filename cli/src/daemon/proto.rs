//! Client↔daemon wire protocol (ADR-003): NDJSON lines over a local
//! socket (Windows named pipe / Unix domain socket). SemVer major
//! mismatch makes the daemon reply `restart` and exit — the client
//! respawns it from its own (newer) binary.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Daemon protocol version — independent of the ce-core handshake
/// proto (contracts/VERSIONING.md governs both). 0.2.0 added the
/// four-class forwarding; 1.0.0 = the M4 content finalization stamp,
/// aligned with the handshake proto (no shape change); 1.1.0 adds
/// the additive hello.token (auth.rs — a pre-1.1.0 line still
/// parses, and gets the unauthorized refusal); 2.0.0 (I round D8,
/// 2026-08-24, the user's "delete it now") drops hello_ok.version —
/// no client ever read it (the negotiation rides `proto`; the binary
/// a respawn needs is the client's own) — and a removed field is a
/// major, so a 1.x client meets `restart` and respawns.
pub const DAEMON_PROTO: &str = "2.0.0";

// Clone: the client's deadline wrapper moves the request into the
// worker thread that owns the connection (plan v2.16-era #85 close);
// plain data, zero wire meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// First line on every connection — and since 1.1.0 the ONLY
    /// line an unauthorized connection gets to send: `token` must
    /// match the served root's .ce/daemon.token (auth.rs), checked
    /// BEFORE the major so a tokenless skew line cannot exit the
    /// daemon at will.
    Hello {
        proto: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        token: String,
    },
    Ping,
    /// Run the dedup pipeline on the daemon's project root.
    Dedup {
        min_tokens: Option<usize>,
        #[serde(default)]
        min_distinct: Option<usize>,
    },
    /// M3 cheap gate: verified T1/T2 matches for content about to be
    /// written to `file_path` (excluded from matching). No refresh.
    Probe {
        file_path: String,
        content: String,
    },
    /// M4 judgment: four-classify the given (before, after) path
    /// pairs of the daemon's root, working tree vs HEAD, via the
    /// daemon-owned ce-core link. Content is read daemon-side; only
    /// paths cross this socket.
    FourClass {
        pairs: Vec<(Option<String>, Option<String>)>,
    },
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    HelloOk {
        proto: String,
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
    ProbeReport {
        matches: serde_json::Value,
        elapsed_ms: u64,
    },
    /// Four-class totals + relocations (fourclass::session shape).
    /// Degradation is inside the report, never silent (A9f).
    FourClassReport {
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

/// (major, minor) for the client's staleness ordering (client.rs);
/// None for unparseable — which Option's Ord ranks below every
/// parsed pair, so garbage protos read as stale, never as current.
pub fn major_minor(v: &str) -> Option<(u64, u64)> {
    let mut parts = v.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}
