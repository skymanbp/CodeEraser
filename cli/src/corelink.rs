//! Persistent NDJSON link to ce-core (ADR-003 wire format,
//! contracts/VERSIONING.md). `Link` holds the spawned core across
//! requests — strict lockstep, exactly one request outstanding; the
//! one-shot `run` (hello + EOF) remains for `ce doctor`.

mod pipe;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::process::{Child, Stdio};

/// Protocol version offered by this client (single source together
/// with core/app/CE/Protocol.hs::proto — contracts/VERSIONING.md §1).
/// 6.6.0 = the tombstone family (ADR-008 fifth instalment, plan
/// v2.27), additive: one new family, tombstone/1. This side measures
/// every candidate surface a changeset wrote and sends one
/// [kind, marks, erasedNames] row per surface plus the declared budget
/// as knob 0; the core judges which rows are sites (a label binding an
/// erased name; a prose sentence with a mark AND a name) and whether
/// the changeset is over its budget, and answers the site indices,
/// their label / prose split and `over` -- this side re-labels the
/// indices into places and never applies the conjunction itself. A
/// core without the capability is named, never read as "no sites".
/// The per-version change ledger lives in contracts/VERSIONING.md and
/// nowhere else; Version.hs points here for the reason. The ledger
/// used to be mirrored beside both constants, and the copies drifted
/// (four entries sat in one mirror and not the other) while a mirror
/// that gains an entry every minor grows without bound inside a
/// size-gated file. What stays beside each constant is THIS version's
/// entry and nothing else, because a reader standing at the constant
/// needs to know what today's number means -- what every past number
/// meant is a ledger question, and the ledger has an address. Four
/// entries had stacked up here by 6.1.0 and pushed the file past its
/// own ratchet: the ledger that documents a size gate is not exempt.
pub const PROTO: &str = "6.6.0";

#[derive(Serialize)]
struct Hello<'a> {
    proto: &'a str,
    r#type: &'a str,
    client: &'a str,
    version: &'a str,
}

#[derive(Deserialize)]
pub struct HelloReply {
    pub proto: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub server: String,
    pub version: String,
    pub accept: bool,
    #[serde(default)]
    pub reason: Option<String>,
    /// Informational discovery only — SemVer stays the sole authority
    /// for accept/reject (§1). Absent capability = run L1, degraded.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// A live core process past its accepted hello. Replies arrive via
/// pipe::reader's channel so every wait carries a deadline — an
/// unbounded read_line let a wedged core hold the daemon and every
/// hook behind it forever.
pub struct Link {
    child: Child,
    replies: std::sync::mpsc::Receiver<std::io::Result<String>>,
    deadline: std::time::Duration,
    caps: Vec<String>,
    next_id: u64,
}

impl Link {
    /// Spawn `core` and perform the handshake; the link stays open.
    pub fn open(core: &str) -> Result<(Link, HelloReply), String> {
        let mut child = spawn(core)?;
        let replies = pipe::reader(child.stdout.take().ok_or("no stdout pipe")?);
        let mut link = Link {
            child,
            replies,
            deadline: pipe::deadline(),
            caps: Vec::new(),
            next_id: 0,
        };
        let hello = Hello {
            proto: PROTO,
            r#type: "hello",
            client: "ce",
            version: env!("CARGO_PKG_VERSION"),
        };
        let line = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        link.send(&line)?;
        let parsed = serde_json::from_str(&link.read_line()?)
            .map_err(|e| format!("bad hello reply: {e}"))?;
        let reply = validate(parsed)?;
        link.caps = reply.capabilities.clone();
        Ok((link, reply))
    }

    pub fn has(&self, capability: &str) -> bool {
        self.caps.iter().any(|c| c == capability)
    }

    /// One `{kind}.request` line out, one `{kind}.result` line in.
    /// Stamps proto/type/id; a reply that does not echo the id or
    /// carry the expected type is a desync — the caller falls back to
    /// L1, visibly (A9f).
    pub fn request(&mut self, kind: &str, mut body: Value) -> Result<Value, String> {
        self.next_id += 1;
        let obj = body
            .as_object_mut()
            .ok_or("request body must be an object")?;
        obj.insert("proto".into(), PROTO.into());
        obj.insert("type".into(), format!("{kind}.request").into());
        obj.insert("id".into(), self.next_id.into());
        let line = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        self.send(&line)?;
        let reply: Value =
            serde_json::from_str(&self.read_line()?).map_err(|e| format!("bad reply: {e}"))?;
        let expected = format!("{kind}.result");
        // an error reply that echoes our id is a REFUSAL, not a
        // desync: surface the core's named reason (review C4 — the
        // knob roads put ce.toml values behind these messages, and
        // "desync" hid every one of them)
        if reply["type"] == "error" && reply["id"] == self.next_id {
            return Err(format!(
                "core refused {kind}.request: {}: {}",
                reply["code"].as_str().unwrap_or("?"),
                reply["message"].as_str().unwrap_or("?")
            ));
        }
        if reply["type"] != expected.as_str() || reply["id"] != self.next_id {
            return Err(format!("desync: expected {expected} id {}", self.next_id));
        }
        Ok(reply)
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        let stdin = self.child.stdin.as_mut().ok_or("no stdin pipe")?;
        writeln!(stdin, "{line}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))
    }

    fn read_line(&mut self) -> Result<String, String> {
        pipe::next_line(&self.replies, self.deadline, &mut self.child)
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        drop(self.child.stdin.take()); // EOF: the polite exit
        // then make exit unconditional — a bare wait() on a wedged
        // core blocked Drop forever, and the core keeps no state a
        // kill could corrupt (pure stdin/stdout judge)
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// One-shot hello for `ce doctor` (M0 behaviour, kept by CI).
pub fn run(core: &str) -> Result<HelloReply, String> {
    Link::open(core).map(|(_link, reply)| reply)
}

/// The effective core binary for a `--core` flag value: an explicit
/// path is used verbatim; the untouched default routes through the
/// daemon's resolver chain — CE_CORE_BIN, a ce-core SIBLING of this
/// executable (the installed layout drops both binaries side by
/// side), then PATH. One authority with daemon/MCP (core_bin), and
/// applied at the ONE spawn throat below, so every judgment family
/// resolves identically with no per-flag plumbing.
pub fn resolve_core(core: &str) -> String {
    if core != "ce-core" {
        return core.to_string();
    }
    crate::daemon::judge::core_bin().unwrap_or_else(|| core.to_string())
}

fn spawn(core: &str) -> Result<Child, String> {
    let effective = resolve_core(core);
    crate::proc::command(&effective)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{effective}`: {e}"))
}

fn validate(reply: HelloReply) -> Result<HelloReply, String> {
    if reply.kind != "hello" || reply.server != "ce-core" {
        return Err(format!(
            "unexpected reply type/server: {}/{}",
            reply.kind, reply.server
        ));
    }
    if major(&reply.proto)? != major(PROTO)? {
        return Err(format!(
            "proto mismatch: core {} vs ce {PROTO}",
            reply.proto
        ));
    }
    match reply.accept {
        true => Ok(reply),
        false => Err(reply
            .reason
            .unwrap_or_else(|| "core rejected handshake".into())),
    }
}

fn major(v: &str) -> Result<u64, String> {
    v.split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("bad semver `{v}`"))
}

#[cfg(test)]
#[path = "../tests/unit/corelink.rs"]
mod tests;
