//! Persistent NDJSON link to ce-core (ADR-003 wire format,
//! contracts/VERSIONING.md). `Link` holds the spawned core across
//! requests — strict lockstep, exactly one request outstanding; the
//! one-shot `run` (hello + EOF) remains for `ce doctor`. A process keeps
//! one core session (plan v2.33 W2-text stage Z, Z1; shared.rs): a link
//! dropped healthy is parked, and the next `open` of the same core takes
//! it back, hello and all — every family a command judges with rides
//! one core process.

pub mod judged;
mod pipe;
mod session;
mod shared;

pub use shared::{locate, release};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Child, Stdio};

/// Protocol version offered by this client (single source together
/// with core/app/CE/Protocol/Version.hs::proto — contracts/VERSIONING.md
/// §1). 9.0.0 = `resolve/1` on text (plan v2.33 W2-text stage A; design
/// booklet docs/reference/algorithm-track.md §3, §6): the edge sweep sends
/// the walked paths, the sites' specifiers and the configuration files it
/// read as text (graph/resolve/), and the core reads go.mod, the root
/// pyproject.toml's keys and the compile databases with their response
/// and flag files; the segment table, vocabulary, affix rows and
/// directory table are retired (a retired key is a major) and the
/// definition package's `resolve` key names the config basenames a
/// request carries, so `tablesDigest` moves. Every other family's bytes
/// stand.
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
pub const PROTO: &str = "9.0.0";

#[derive(Serialize)]
struct Hello<'a> {
    proto: &'a str,
    r#type: &'a str,
    client: &'a str,
    version: &'a str,
}

#[derive(Deserialize, Clone, Default)]
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
    /// The fnv1a64 of the core's definition package (7.7.0): a core
    /// answering `tables/1` names it here. Absent = an older core (the
    /// package fetch refuses it by its missing capability); present
    /// and unequal to the package this run read = refused by name
    /// (crate::tables::check_hello).
    #[serde(default, rename = "tablesDigest")]
    pub tables_digest: Option<u64>,
}

/// A handle on one core session (session.rs) past its accepted hello.
/// Replies arrive via pipe::reader's channel so every wait carries a
/// deadline — an unbounded read_line let a wedged core hold the daemon
/// and every hook behind it forever. Dropped, a link parks a healthy
/// session for the process's next `open` (shared.rs) unless it was
/// opened `own`; a broken session ends.
pub struct Link {
    session: Option<session::Session>,
    own: bool,
}

impl Link {
    /// A session with `core`: the process's parked one when it runs that
    /// core, else spawned and handshaken; the hello it answered.
    pub fn open(core: &str) -> Result<(Link, HelloReply), String> {
        let key = shared::key(core);
        match shared::take(&key) {
            Some(parked) => {
                crate::tables::check_hello(core, parked.hello.tables_digest)?;
                Ok(Link::over(parked, false))
            }
            None => session::Session::open(core, key).map(|s| Link::over(s, false)),
        }
    }

    /// A session of this link's own, spawned and never parked: the
    /// daemon's long-lived core (the process that outlives commands
    /// retires it on purpose) and `ce doctor`'s one-shot hello.
    pub fn own(core: &str) -> Result<(Link, HelloReply), String> {
        let key = shared::key(core);
        session::Session::open(core, key).map(|s| Link::over(s, true))
    }

    fn over(s: session::Session, own: bool) -> (Link, HelloReply) {
        let hello = s.hello.clone();
        let link = Link {
            session: Some(s),
            own,
        };
        (link, hello)
    }

    fn session(&self) -> &session::Session {
        self.session
            .as_ref()
            .expect("a live link holds its session")
    }

    pub fn has(&self, capability: &str) -> bool {
        self.session()
            .hello
            .capabilities
            .iter()
            .any(|c| c == capability)
    }

    /// The core's process id (the slot's battery tells sessions apart).
    #[cfg(test)]
    pub(crate) fn pid(&self) -> u32 {
        self.session().pid()
    }

    /// The proto the core answered its hello with.
    pub fn proto(&self) -> &str {
        &self.session().hello.proto
    }

    /// One `{kind}.request` line out, one `{kind}.result` line in
    /// (session.rs: a refusal is named, a desync breaks the session).
    pub fn request(&mut self, kind: &str, body: Value) -> Result<Value, String> {
        self.session
            .as_mut()
            .expect("a live link holds its session")
            .request(kind, body)
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        if let Some(s) = self.session.take()
            && !self.own
            && !s.broken
        {
            shared::park(s);
        }
    }
}

/// One-shot hello for `ce doctor` (M0 behaviour, kept by CI): a fresh
/// core, never the process's parked one.
pub fn run(core: &str) -> Result<HelloReply, String> {
    Link::own(core).map(|(_link, reply)| reply)
}

/// The effective core binary for a `--core` flag value: an explicit
/// path is used verbatim; the untouched default routes through the
/// daemon's resolver chain — the process's global `--core` (when it
/// named one), then CE_CORE_BIN, a ce-core SIBLING of this executable
/// (the installed layout drops both binaries side by side), then
/// PATH. One authority with daemon/MCP (core_bin), and
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
