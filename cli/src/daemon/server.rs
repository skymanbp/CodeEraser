//! Per-project daemon server (ADR-003): lazy-started by the client,
//! sole writer of the SQLite index (requests are handled serially —
//! the serialization IS the multi-session concurrency model), exits
//! after 30 min idle or on a version-skew hello.

use super::proto::{DAEMON_PROTO, Request, Response, major, socket_name};
use anyhow::{Context, Result};
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 30 min idle → exit (ADR-003). Overridable for tests only.
fn idle_max() -> Duration {
    std::env::var("CE_DAEMON_IDLE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(30 * 60))
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
    let start = Instant::now();
    touch(start);
    spawn_idle_watchdog(start);
    eprintln!("ce daemon: serving {} on {name}", root.display());
    for conn in listener.incoming() {
        let stream = match conn {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ce daemon: accept: {e}");
                continue;
            }
        };
        touch(start);
        match handle(stream, &root, start) {
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
fn handle(stream: Stream, root: &Path, start: Instant) -> Result<bool> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(true); // client hung up
        }
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
        if !dispatch(&mut reader, root, start, req)? {
            return Ok(false);
        }
    }
}

fn dispatch(
    reader: &mut BufReader<Stream>,
    root: &Path,
    start: Instant,
    req: Request,
) -> Result<bool> {
    match req {
        Request::Hello { proto } => {
            if major(&proto) != major(DAEMON_PROTO) {
                reply(
                    reader.get_mut(),
                    &Response::Restart {
                        reason: format!("proto {proto} vs daemon {DAEMON_PROTO}"),
                    },
                )?;
                return Ok(false); // exit → client respawns new binary
            }
            reply(
                reader.get_mut(),
                &Response::HelloOk {
                    proto: DAEMON_PROTO.into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                },
            )?;
        }
        Request::Ping => reply(
            reader.get_mut(),
            &Response::Pong {
                uptime_ms: start.elapsed().as_millis() as u64,
            },
        )?,
        Request::Dedup { min_tokens } => {
            let resp = match run_dedup(root, min_tokens) {
                Ok(report) => Response::DedupReport { report },
                Err(e) => Response::Error {
                    message: format!("{e:#}"),
                },
            };
            reply(reader.get_mut(), &resp)?;
        }
        Request::Shutdown => {
            reply(reader.get_mut(), &Response::Bye)?;
            return Ok(false);
        }
    }
    Ok(true)
}

fn run_dedup(root: &Path, min_tokens: Option<usize>) -> Result<serde_json::Value> {
    let db: Option<PathBuf> = None; // daemon always uses <root>/.ce/index.db
    let (found, summary) = crate::dedup::analyze(root, db, min_tokens)?;
    crate::dedup::report_json(&found, &summary)
}

fn reply(stream: &mut Stream, resp: &Response) -> Result<()> {
    let line = serde_json::to_string(resp)?;
    writeln!(stream, "{line}")?;
    stream.flush()?;
    Ok(())
}
