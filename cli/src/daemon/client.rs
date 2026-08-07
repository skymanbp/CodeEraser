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

/// One round-trip ONLY IF the daemon is already up — never spawns.
/// `ce doctor` uses this: a diagnostic must not mutate the state it
/// reports (attack review 2026-08-07: doctor's warm-up ping left a
/// detached 30-min daemon per invocation dir, locking the exe).
pub fn request_if_running(root: &Path, req: &Request) -> Result<Response> {
    let mut conn = BufReader::new(try_connect(root)?);
    match hello(&mut conn)? {
        Response::HelloOk { .. } => round_trip(&mut conn, req),
        other => bail!("incompatible daemon: {other:?}"),
    }
}

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
    unset_stdio_inheritance();
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

/// Windows handle-inheritance leak guard (found by the guard_hook
/// e2e: it hung until the daemon died). When a hook process lazily
/// spawns the long-lived daemon, the daemon inherits the hook's
/// STDOUT PIPE write end — the hook's consumer (Claude Code, or a
/// test's wait_with_output) then never sees EOF until the daemon
/// exits, hanging the hook slot. Marking our own std handles
/// non-inheritable before the spawn closes that hole; the daemon's
/// stdio is null anyway. Unix needs nothing: std sets CLOEXEC on
/// non-std fds and the null stdio covers the rest.
#[cfg(windows)]
fn unset_stdio_inheritance() {
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
    use windows_sys::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: querying our own std handle and flipping its
        // inherit flag touches no memory and cannot invalidate it.
        unsafe {
            let handle = GetStdHandle(which);
            if !handle.is_null() && handle as isize != -1 {
                SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
            }
        }
    }
}

#[cfg(not(windows))]
fn unset_stdio_inheritance() {}
