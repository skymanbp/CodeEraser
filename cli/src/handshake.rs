//! NDJSON handshake client for ce-core (ADR-003 wire format).
//! One `hello` line out, one reply line back; SemVer major must match.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

/// Protocol version offered by this client (contracts/VERSIONING.md).
pub const PROTO: &str = "0.1.0";

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
}

/// Spawn `core`, perform the handshake, close stdin (core exits on EOF).
pub fn run(core: &str) -> Result<HelloReply, String> {
    let mut child = spawn(core)?;
    let result = exchange(&mut child);
    let _ = child.wait();
    result
}

fn spawn(core: &str) -> Result<Child, String> {
    Command::new(core)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{core}`: {e}"))
}

fn exchange(child: &mut Child) -> Result<HelloReply, String> {
    let hello = Hello {
        proto: PROTO,
        r#type: "hello",
        client: "ce",
        version: env!("CARGO_PKG_VERSION"),
    };
    let line = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
    {
        let stdin = child.stdin.as_mut().ok_or("no stdin pipe")?;
        writeln!(stdin, "{line}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))?;
    }
    let reply = read_reply(child)?;
    drop(child.stdin.take()); // EOF => core exits
    validate(reply)
}

fn read_reply(child: &mut Child) -> Result<HelloReply, String> {
    let stdout = child.stdout.take().ok_or("no stdout pipe")?;
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .map_err(|e| format!("read: {e}"))?;
    serde_json::from_str(line.trim()).map_err(|e| format!("bad reply `{}`: {e}", line.trim()))
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
