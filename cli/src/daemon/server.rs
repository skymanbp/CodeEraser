//! Per-project daemon server (ADR-003 as amended, plan v1.7): lazy-
//! started by the client, ONE convergent writer among the CLI's own
//! (requests are handled serially within the daemon; cross-process
//! convergence is the idempotent-write contract concurrent_writers
//! proves), exits after 30 min idle or on a version-skew hello.
//! Wire contract: contracts/DAEMON.md + the daemon_proto goldens.

mod replies;

use super::auth;
use super::coldstart;
use super::judge::Judge;
use super::proto::{DAEMON_PROTO, Request, Response, major, socket_name};
use anyhow::{Context, Result};
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};
use replies::{dedup_reply, probe_reply};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 30 min idle → exit (ADR-003). Overridable for tests only.
fn idle_max() -> Duration {
    crate::config::env_secs("CE_DAEMON_IDLE_SECS", 30 * 60)
}

static LAST_ACTIVITY_MS: AtomicU64 = AtomicU64::new(0);

fn touch(start: Instant) {
    LAST_ACTIVITY_MS.store(start.elapsed().as_millis() as u64, Ordering::Relaxed);
}

/// Serve until idle timeout, shutdown request, or version skew.
pub fn serve(root: &Path) -> Result<()> {
    let root = std::fs::canonicalize(root).with_context(|| format!("root {}", root.display()))?;
    let name = socket_name(&root);
    let ns = name
        .clone()
        .to_ns_name::<GenericNamespaced>()
        .context("socket name")?;
    let listener = ListenerOptions::new()
        .name(ns)
        .create_sync()
        .with_context(|| format!("bind {name} (another daemon already serving this root?)"))?;
    // after the bind on purpose: the bind is the singleton race, and
    // a loser minting first would lock clients out of the winner
    let token = auth::establish(&root)?;
    let start = Instant::now();
    touch(start);
    spawn_idle_watchdog(start);
    coldstart::init(&root); // after bind: a lost race spawns no build
    eprintln!("ce daemon: serving {} on {name}", root.display());
    let mut judge = Judge::default();
    for conn in listener.incoming() {
        let stream = match conn {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ce daemon: accept: {e}");
                continue;
            }
        };
        touch(start);
        match handle(stream, &root, start, &mut judge, &token) {
            Ok(true) => {}
            Ok(false) => return Ok(()), // shutdown or skew exit
            Err(e) => eprintln!("ce daemon: connection: {e}"),
        }
        touch(start);
    }
    Ok(())
}

/// The watchdog only ever exits the whole process — a stuck request
/// cannot block it because it reads a lock-free timestamp.
fn spawn_idle_watchdog(start: Instant) {
    let max = idle_max();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(max.min(Duration::from_secs(60)));
            let last = LAST_ACTIVITY_MS.load(Ordering::Relaxed);
            let now = start.elapsed().as_millis() as u64;
            if now.saturating_sub(last) > max.as_millis() as u64 {
                eprintln!("ce daemon: idle {max:?} exceeded, exiting");
                std::process::exit(0);
            }
        }
    });
}

/// Returns Ok(false) when the daemon must stop (shutdown / skew).
/// Only a token-bearing hello unlocks a connection (1.1.0): before
/// it, every other line gets the unauthorized refusal and the
/// CONNECTION closes — the daemon itself lives on.
fn handle(
    stream: Stream,
    root: &Path,
    start: Instant,
    judge: &mut Judge,
    token: &str,
) -> Result<bool> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    let mut authed = false;
    loop {
        buf.clear();
        if reader.read_until(b'\n', &mut buf)? == 0 {
            return Ok(true); // client hung up
        }
        // bytes first, lossy second: a non-UTF-8 line is the same
        // 坏行 class as bad JSON and must get the error reply with
        // the connection surviving — read_line's Err tore the
        // connection down silently (clearance review vs DAEMON.md's
        // 「绝不崩连接」 promise)
        let line = String::from_utf8_lossy(&buf);
        let req: Request = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                reply(
                    reader.get_mut(),
                    &Response::Error {
                        message: format!("bad request: {e}"),
                    },
                )?;
                continue;
            }
        };
        touch(start);
        match gate(&mut reader, &req, token, &mut authed)? {
            Flow::Next => continue,
            Flow::CloseConn => return Ok(true), // connection only
            Flow::ExitDaemon => return Ok(false),
            Flow::Dispatch => {}
        }
        if !dispatch(&mut reader, root, start, judge, req)? {
            return Ok(false);
        }
    }
}

/// What the front door decided about one parsed line.
enum Flow {
    Dispatch,
    Next,
    CloseConn,
    ExitDaemon,
}

/// The 1.1.0 front door: a hello negotiates (token before major) and
/// flips `authed`; any other line before an authed hello is refused
/// with the connection closed — the daemon itself lives on.
fn gate(
    reader: &mut BufReader<Stream>,
    req: &Request,
    expected: &str,
    authed: &mut bool,
) -> Result<Flow> {
    if let Request::Hello { proto, token: sent } = req {
        let (resp, verdict) = hello_reply(proto, sent, expected);
        reply(reader.get_mut(), &resp)?;
        return Ok(match verdict {
            Hello::Authed => {
                *authed = true;
                Flow::Next
            }
            Hello::Refused => Flow::CloseConn,
            Hello::Skew => Flow::ExitDaemon,
        });
    }
    if *authed {
        return Ok(Flow::Dispatch);
    }
    reply(
        reader.get_mut(),
        &Response::Error {
            message: auth::UNAUTHORIZED.into(),
        },
    )?;
    Ok(Flow::CloseConn)
}

/// Build the reply per request; the bool = keep serving.
fn dispatch(
    reader: &mut BufReader<Stream>,
    root: &Path,
    start: Instant,
    judge: &mut Judge,
    req: Request,
) -> Result<bool> {
    let (resp, keep) = match req {
        // every hello (first or repeated) is consumed by handle();
        // this arm is unreachable and exists for match exhaustiveness
        Request::Hello { .. } => (
            Response::Error {
                message: "hello is negotiated per connection".into(),
            },
            true,
        ),
        Request::Ping => (
            Response::Pong {
                uptime_ms: start.elapsed().as_millis() as u64,
            },
            true,
        ),
        Request::Dedup {
            min_tokens,
            min_distinct,
        } => (dedup_reply(root, min_tokens, min_distinct), true),
        Request::Probe { file_path, content } => (probe_reply(root, &file_path, &content), true),
        Request::FourClass { pairs } => (
            Response::FourClassReport {
                report: judge.classify(root, &pairs),
            },
            true,
        ),
        Request::Shutdown => (Response::Bye, false),
    };
    reply(reader.get_mut(), &resp)?;
    Ok(keep)
}

/// What one hello line did to the connection.
enum Hello {
    Authed,
    Refused,
    Skew,
}

/// Token BEFORE major: a tokenless line must not be able to exit the
/// daemon through a faked skew — that is an unauthenticated kill. A
/// real newer-major client reads the token file first, so the
/// restart-respawn chain still runs; on skew the daemon exits and
/// the client respawns one from its own (newer) binary.
fn hello_reply(proto: &str, sent: &str, expected: &str) -> (Response, Hello) {
    if sent != expected {
        return (
            Response::Error {
                message: auth::UNAUTHORIZED.into(),
            },
            Hello::Refused,
        );
    }
    if major(proto) != major(DAEMON_PROTO) {
        return (
            Response::Restart {
                reason: format!("proto {proto} vs daemon {DAEMON_PROTO}"),
            },
            Hello::Skew,
        );
    }
    (
        Response::HelloOk {
            proto: DAEMON_PROTO.into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        Hello::Authed,
    )
}

// Reply builders (dedup/probe) live in replies.rs — dogfood split.

fn reply(stream: &mut Stream, resp: &Response) -> Result<()> {
    let line = serde_json::to_string(resp)?;
    writeln!(stream, "{line}")?;
    stream.flush()?;
    Ok(())
}
