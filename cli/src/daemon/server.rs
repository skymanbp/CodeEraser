//! Per-project daemon server (ADR-003): lazy-started by the client,
//! sole writer of the SQLite index (requests are handled serially —
//! the serialization IS the multi-session concurrency model), exits
//! after 30 min idle or on a version-skew hello.

use super::coldstart;
use super::judge::Judge;
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
        match handle(stream, &root, start, &mut judge) {
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
fn handle(stream: Stream, root: &Path, start: Instant, judge: &mut Judge) -> Result<bool> {
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
        if !dispatch(&mut reader, root, start, judge, req)? {
            return Ok(false);
        }
    }
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
        Request::Hello { proto } => hello_reply(&proto),
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

/// Version skew → restart reply and stop; the client respawns a
/// daemon from its own (newer) binary.
fn hello_reply(proto: &str) -> (Response, bool) {
    if major(proto) != major(DAEMON_PROTO) {
        return (
            Response::Restart {
                reason: format!("proto {proto} vs daemon {DAEMON_PROTO}"),
            },
            false,
        );
    }
    (
        Response::HelloOk {
            proto: DAEMON_PROTO.into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        true,
    )
}

fn dedup_reply(root: &Path, min_tokens: Option<usize>, min_distinct: Option<usize>) -> Response {
    match run_dedup(root, min_tokens, min_distinct) {
        Ok(report) => {
            coldstart::mark_ready();
            Response::DedupReport { report }
        }
        Err(e) => Response::Error {
            message: format!("{e:#}"),
        },
    }
}

fn probe_reply(root: &Path, file_path: &str, content: &str) -> Response {
    if !coldstart::index_ready(root) {
        return Response::Error {
            message: "index cold start: first build in progress".into(),
        };
    }
    let t0 = Instant::now();
    match run_probe(root, file_path, content) {
        Ok(matches) => Response::ProbeReport {
            matches,
            elapsed_ms: t0.elapsed().as_millis() as u64,
        },
        Err(e) => Response::Error {
            message: format!("{e:#}"),
        },
    }
}

/// Cheap by design (ADR-004): opens the index read-path only, never
/// refreshes; unknown language probes report no matches.
fn run_probe(root: &Path, file_path: &str, content: &str) -> Result<serde_json::Value> {
    use crate::dedup::{Params, index::Index, pairs, probe};
    use crate::scan::lang::Lang;
    let path = Path::new(file_path);
    let Some(lang) = Lang::from_path(path) else {
        return Ok(serde_json::json!([]));
    };
    if lang.grammar().is_none() {
        return Ok(serde_json::json!([]));
    }
    // canonicalize before stripping: `root` is canonical (serve()),
    // and on Windows that is the \\?\ verbatim form — a plain
    // absolute file_path never strip-matches it, which silently
    // killed probe self-exclusion (caught by the observe-feed golden
    // diverging across CI platforms). A not-yet-existing file can't
    // canonicalize — and can't be in the index either, so the raw
    // fallback is safe. The SPELLING then goes through the one
    // rel_str throat (M5-close review: this was the third copy).
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let rel = crate::scan::walk::rel_str(root, &canon);
    let p = Params::default();
    let idx = Index::open(&root.join(".ce/index.db"), p)?;
    let f = pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    };
    let target = probe::Target {
        rel: &rel,
        content: content.as_bytes(),
        lang,
    };
    let matches = probe::probe(&idx, root, target, p, f)?;
    Ok(serde_json::to_value(matches)?)
}

fn run_dedup(
    root: &Path,
    min_tokens: Option<usize>,
    min_distinct: Option<usize>,
) -> Result<serde_json::Value> {
    let db: Option<PathBuf> = None; // daemon always uses <root>/.ce/index.db
    let (found, summary) = crate::dedup::analyze(root, db, min_tokens, min_distinct)?;
    crate::dedup::report_json(&found, &summary)
}

fn reply(stream: &mut Stream, resp: &Response) -> Result<()> {
    let line = serde_json::to_string(resp)?;
    writeln!(stream, "{line}")?;
    stream.flush()?;
    Ok(())
}
