//! Daemon client with lazy start (ADR-003): connect, else spawn
//! `ce daemon` detached and retry with backoff. On a `restart` reply
//! (version skew) the old daemon exits and one respawn round brings
//! up a daemon from THIS binary.

use super::proto::{DAEMON_PROTO, Request, Response, socket_name};
use anyhow::{Context, Result, bail};
use interprocess::local_socket::traits::Stream as _;
use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;

const SPAWN_RETRIES: u32 = 20;
const RETRY_DELAY: Duration = Duration::from_millis(100);

/// One request/response round-trip, lazily starting the daemon.
pub fn request(root: &Path, req: &Request) -> Result<Response> {
    let mut conn = connect_or_spawn(root)?;
    match hello(&mut conn)? {
        Response::HelloOk { .. } => {}
        Response::Restart { reason } => {
            // old daemon exited; one respawn round from this binary
            drop(conn);
            conn = connect_or_spawn(root)?;
            match hello(&mut conn)? {
                Response::HelloOk { .. } => {}
                other => bail!("daemon still incompatible after restart ({reason}): {other:?}"),
            }
        }
        other => bail!("unexpected hello reply: {other:?}"),
    }
    round_trip(&mut conn, req)
}

fn hello(conn: &mut BufReader<Stream>) -> Result<Response> {
    round_trip(
        conn,
        &Request::Hello {
            proto: DAEMON_PROTO.into(),
        },
    )
}

fn round_trip(conn: &mut BufReader<Stream>, req: &Request) -> Result<Response> {
    let line = serde_json::to_string(req)?;
    writeln!(conn.get_mut(), "{line}")?;
    conn.get_mut().flush()?;
    let mut reply = String::new();
    if conn.read_line(&mut reply)? == 0 {
        bail!("daemon closed the connection");
    }
    serde_json::from_str(reply.trim())
        .with_context(|| format!("bad daemon reply `{}`", reply.trim()))
}

fn connect_or_spawn(root: &Path) -> Result<BufReader<Stream>> {
    if let Ok(s) = try_connect(root) {
        return Ok(BufReader::new(s));
    }
    spawn_daemon(root)?;
    for _ in 0..SPAWN_RETRIES {
        std::thread::sleep(RETRY_DELAY);
        if let Ok(s) = try_connect(root) {
            return Ok(BufReader::new(s));
        }
    }
    bail!(
        "daemon for {} did not come up within {:?}",
        root.display(),
        RETRY_DELAY * SPAWN_RETRIES
    )
}

fn try_connect(root: &Path) -> Result<Stream> {
    let ns = socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .context("socket name")?;
    Ok(Stream::connect(ns)?)
}

/// Detached spawn of THIS binary as the daemon; losing the race to
/// another spawner is fine — the retry loop connects to whoever won.
fn spawn_daemon(root: &Path) -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    std::process::Command::new(exe)
        .arg("daemon")
        .arg(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("spawn ce daemon")?;
    Ok(())
}
