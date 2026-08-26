//! Daemon client with lazy start (ADR-003): connect, else spawn
//! `ce daemon` detached and retry with backoff. On a `restart` reply
//! (version skew) the old daemon exits and one respawn round brings
//! up a daemon from THIS binary.

use super::auth;
use super::proto::{DAEMON_PROTO, Request, Response, major, socket_name};
use anyhow::{Context, Result, bail, ensure};
use interprocess::local_socket::traits::Stream as _;
use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;

const SPAWN_RETRIES: u32 = 20;
const RETRY_DELAY: Duration = Duration::from_millis(100);

/// The client's whole-conversation deadline (audit #85, closed here
/// after its "1.0 后再议"): connect, hello rounds and the request
/// reply together. DAEMON.md recorded the asymmetry in so many words
/// — the daemon bounds its peers and its core, and the CLIENT alone
/// read forever, so a wedged daemon parked a PreToolUse hook
/// indefinitely. Default 75 s = the daemon's own 60 s core deadline
/// plus margin: the slowest legitimate reply is a four_class batch
/// waiting on ce-core, and a client deadline under the server's
/// would convert healthy slow judgments into false hangs.
const CLIENT_DEADLINE: Duration = Duration::from_secs(75);

fn deadline_from_env() -> Duration {
    std::env::var("CE_CLIENT_DEADLINE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(CLIENT_DEADLINE)
}

/// Run one whole negotiate on a worker thread that OWNS the
/// connection, and give up at `deadline`. This shape — rather than a
/// per-read timeout — because interprocess 2.4.3 exposes none for
/// named pipes (only the discouraged PIPE_NOWAIT polling mode), and
/// a bound on the conversation is what the hook actually needs. On
/// expiry the worker stays parked on its read and dies with the
/// process — every hook exits immediately after; the one long-lived
/// caller (the GUI's doctor probe) leaks at most one parked thread
/// per wedged-daemon event, which DAEMON.md now says out loud.
fn bounded(root: &Path, req: &Request, lazy: bool, deadline: Duration) -> Result<Response> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (root, req) = (root.to_path_buf(), req.clone());
    std::thread::spawn(move || {
        let _ = tx.send(negotiate(&root, &req, lazy));
    });
    rx.recv_timeout(deadline).unwrap_or_else(|_| {
        bail!(
            "daemon did not answer within {}s — a wedged daemon is replaced by \
             `ce ping` after it exits, or killed by hand; CE_CLIENT_DEADLINE_SECS overrides",
            deadline.as_secs()
        )
    })
}

/// One round-trip ONLY IF the daemon is already up — never spawns.
/// `ce doctor` uses this: a diagnostic must not mutate the state it
/// reports (attack review 2026-08-07: doctor's warm-up ping left a
/// detached 30-min daemon per invocation dir, locking the exe). It
/// may still RETIRE a daemon whose hello it refuses to trust
/// (refuse_stale) — leaving an unauthenticated one serving is the
/// worse mutation, and no diagnostic reads through it either way.
/// One unauthorized retry, same window as `request`: eject's
/// shutdown raced a freshly bound daemon's token write, gave up on
/// the refusal, and removed .ce out from under a live daemon
/// (review 2026-08-20 #7).
pub fn request_if_running(root: &Path, req: &Request) -> Result<Response> {
    bounded(root, req, false, deadline_from_env())
}

/// Does anyone still hold this root's socket? A connect that is
/// REFUSED is the only proof the daemon is gone — a failed request
/// is not, because the very race the header names (a fresh daemon
/// minting its token under our shutdown) fails the request while the
/// daemon lives. `ce eject` asks before deleting .ce, and the
/// connection is dropped at once, so the daemon's slot is freed on
/// the next read.
pub fn is_running(root: &Path) -> bool {
    try_connect(root).is_ok()
}

/// A HelloOk is only trusted when the replied proto is not OLDER
/// than this client's (same major — a mismatch gets `restart`). An
/// already-running pre-1.1.0 daemon parses the 1.1.0 hello (serde
/// ignores the unknown token field), checks no credential, and
/// answers HelloOk — trusting that reply kept the credential gate
/// inert until the old daemon idled out (review 2026-08-20 #1).
fn stale(theirs: &str) -> bool {
    use super::proto::major_minor;
    major_minor(theirs) < major_minor(DAEMON_PROTO)
}

/// One round-trip, lazily starting the daemon. Every recoverable
/// hello outcome gets exactly one more round: a skew restart
/// (respawn from this binary), an unauthorized refusal (a fresh
/// daemon mints its token moments after the bind this client raced),
/// or a STALE HelloOk — a same-major older daemon is asked to leave
/// (it answered our hello by its own rules, so it takes a shutdown)
/// and the respawn round brings up one from this binary.
pub fn request(root: &Path, req: &Request) -> Result<Response> {
    bounded(root, req, true, deadline_from_env())
}

/// The one hello loop both entry points share (the dedup ratchet
/// caught them growing as clones). Every recoverable outcome gets
/// exactly one more round; `lazy` gates the respawn arms — the
/// if-running path may reconnect but never bring a daemon up.
fn negotiate(root: &Path, req: &Request, lazy: bool) -> Result<Response> {
    let mut conn = if lazy {
        connect_or_spawn(root)?
    } else {
        BufReader::new(try_connect(root)?)
    };
    for retry in [false, true] {
        match hello(&mut conn, root)? {
            Response::HelloOk { ref proto, .. } if stale(proto) => {
                if !lazy {
                    return refuse_stale(&mut conn, req, proto);
                }
                ensure!(
                    !retry,
                    "stale daemon (proto {proto} vs {DAEMON_PROTO}) survived a shutdown round"
                );
                round_trip(&mut conn, &Request::Shutdown)?;
                std::thread::sleep(RETRY_DELAY); // let the old one die
                conn = connect_or_spawn(root)?;
            }
            Response::HelloOk { .. } => return round_trip(&mut conn, req),
            Response::Restart { .. } if lazy && !retry => conn = connect_or_spawn(root)?,
            Response::Error { ref message } if !retry && message.starts_with("unauthorized") => {
                std::thread::sleep(RETRY_DELAY); // let the fresh token land
                conn = BufReader::new(try_connect(root)?);
            }
            other => bail!("hello failed: {other:?}"),
        }
    }
    unreachable!("the second round returned or bailed")
}

/// The non-lazy path's answer to a HelloOk it will not trust. It
/// forwards the shutdown eject needs to retire the daemon — and,
/// for anything else, sends that shutdown ITSELF on the same
/// precedent: reporting alone left a daemon whose hello checks NO
/// credential (every pre-1.1.0 one) serving each later caller until
/// some lazy client happened by, and `ce doctor` — the diagnostic
/// that meets it first — is exactly this path. Same major only: a
/// cross-major daemon owes us `restart`, and its `shutdown` word is
/// not ours to assume. The shutdown's own outcome is moot; the bail
/// reports the staleness either way, so the caller still sees it.
fn refuse_stale(conn: &mut BufReader<Stream>, req: &Request, proto: &str) -> Result<Response> {
    if matches!(req, Request::Shutdown) {
        return round_trip(conn, req);
    }
    if major(proto) == major(DAEMON_PROTO) {
        let _ = round_trip(conn, &Request::Shutdown);
    }
    bail!("stale daemon (proto {proto} vs {DAEMON_PROTO}); any ce command replaces it")
}

fn hello(conn: &mut BufReader<Stream>, root: &Path) -> Result<Response> {
    round_trip(
        conn,
        &Request::Hello {
            proto: DAEMON_PROTO.into(),
            token: auth::read(root),
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
    crate::proc::command(exe)
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

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
