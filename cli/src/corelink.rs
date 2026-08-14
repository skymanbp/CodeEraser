//! Persistent NDJSON link to ce-core (ADR-003 wire format,
//! contracts/VERSIONING.md). `Link` holds the spawned core across
//! requests — strict lockstep, exactly one request outstanding; the
//! one-shot `run` (hello + EOF) remains for `ce doctor`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

/// Protocol version offered by this client (single source together
/// with core/app/CE/Protocol.hs::proto — contracts/VERSIONING.md §1).
/// 2.3.0 = verdict.request `ceilings` rows + the reply's effective-
/// knobs echo (ADR-008 first step, additive). 2.2.0 = clone/1 +
/// docdup/1 + verdict/1 declared in one additive minor (M5-3a).
/// 2.1.0 = graph/1 (M5-2a). 2.0.0 was the M5-1c-iii anchor-width
/// request shape (a breaking change, major per §2); 1.0.0 was the
/// M4 content finalization freeze.
pub const PROTO: &str = "2.3.0";

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

/// A live core process past its accepted hello.
pub struct Link {
    child: Child,
    out: BufReader<ChildStdout>,
    caps: Vec<String>,
    next_id: u64,
}

impl Link {
    /// Spawn `core` and perform the handshake; the link stays open.
    pub fn open(core: &str) -> Result<(Link, HelloReply), String> {
        let mut child = spawn(core)?;
        let out = BufReader::new(child.stdout.take().ok_or("no stdout pipe")?);
        let mut link = Link {
            child,
            out,
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
        let mut line = String::new();
        self.out
            .read_line(&mut line)
            .map_err(|e| format!("read: {e}"))?;
        if line.is_empty() {
            return Err("core closed the pipe".into());
        }
        Ok(line.trim().to_string())
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        drop(self.child.stdin.take()); // EOF => core exits
        let _ = self.child.wait();
    }
}

/// One-shot hello for `ce doctor` (M0 behaviour, kept by CI).
pub fn run(core: &str) -> Result<HelloReply, String> {
    Link::open(core).map(|(_link, reply)| reply)
}

fn spawn(core: &str) -> Result<Child, String> {
    Command::new(core)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{core}`: {e}"))
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
