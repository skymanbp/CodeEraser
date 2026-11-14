//! Persistent NDJSON link to ce-core (ADR-003 wire format,
//! contracts/VERSIONING.md). `Link` holds the spawned core across
//! requests — strict lockstep, exactly one request outstanding;
//! `Link::own` is the one-shot hello `ce doctor` and trend's toolchain
//! stamp read (a fresh core, never the parked one). A process keeps
//! one core session (plan v2.33 W2-text stage Z, Z1; shared.rs): a link
//! dropped healthy is parked, and the next `open` of the same core takes
//! it back, hello and all — every family a command judges with rides
//! one core process.

mod cputime;
pub mod judged;
mod pipe;
mod session;
mod shared;
mod watch;

pub use session::sent_on_this_thread;
pub use shared::{locate, release};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Child, Stdio};

/// Protocol version offered by this client (single source together
/// with core/app/CE/Protocol/Version.hs::proto — contracts/VERSIONING.md
/// §1). 10.0.0 = lane v235_score (breaking): verdict/1 judges the score
/// by the extent law (plan v2.35) — every verdict request carries the
/// four tables contSpans / dupSpans / refLines / proseLines
/// (score::located), the retired knob codes (thresholds 0, ceilings 3)
/// are refused, the reply's axes are own shares and `extent` carries E
/// and L. Same version, lane v235_licence part 4C: resolve/1's `also`
/// also names, for a Markdown or HTML site whose answer is a confirmed
/// section of another file, that file (CE.Resolve.Answer `holderOf`).
/// Same version, lane v235_imports: cover/1 (additive), the T1/T2
/// coverage rule C_t (dedup/pairs/cover.rs sends the verified runs, the
/// core classes them; at most `limits.caps.cover_rows` rows a request).
/// Same version, lane v235_cleanup2: verdict/1's `pos` row is
/// [u, indeg, sccId, sccSize, reachIn] (score::pos_rows); graph/1's
/// outdegree was never read by the verdict family and is not sent.
/// Every other family's bytes stand.
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
pub const PROTO: &str = "10.0.0";

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
    /// answering `tables/1` names it here. Absent = skew (the package
    /// fetch refuses a core without the capability, corelink::skew);
    /// present and unequal to the package this run read = refused by
    /// name (crate::tables::check_hello).
    #[serde(default, rename = "tablesDigest")]
    pub tables_digest: Option<u64>,
}

/// A handle on one core session (session.rs) past its accepted hello.
/// Replies arrive via pipe::reader's channel so every exchange is watched
/// (watch.rs: a core that makes no progress, or spends past its CPU
/// budget, is killed) — an unbounded read_line let a wedged core hold the
/// daemon and every hook behind it forever. Dropped, a link parks a healthy
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
    /// retires it on purpose) and the one-shot hellos (`ce doctor`,
    /// trend's toolchain stamp; M0 behaviour, kept by CI).
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

    /// Whether the last failed `request` lost the session's framing (a
    /// write or read that failed, a stall or an overspend, a reply that does not
    /// parse, a desync) rather than meeting the core's refusal: the
    /// error text is the same either way, so a reader that keys a retry
    /// budget or a link's life on the failure asks here after an `Err`.
    /// A refused request leaves the session whole (session.rs).
    pub fn is_broken(&self) -> bool {
        self.session().broken
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

/// What this ce needs and the core it reached does not carry — a
/// capability or a reply key every core of the pinned major (PROTO)
/// answers with. `validate` pins the major and `tables::check_hello` the
/// definition package's digest, so no other core gets this far: reaching it means the two
/// binaries come from different builds, and the one remedy is named.
pub fn skew(what: &str) -> String {
    format!("core offers no {what}: ce and ce-core are out of step (install both from one release)")
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
