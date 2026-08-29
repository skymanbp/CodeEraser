//! Per-project daemon server (ADR-003 as amended, plan v1.7): lazy-
//! started by the client, one thread per connection so a silent
//! client parks its own thread and never the accept loop (review
//! 2026-08-20 #3 — before this, a connection that sent no newline
//! stalled every other request). Requests are still handled one at a
//! time: dispatch.rs builds every reply under the judge lock and
//! releases it before conn.rs writes; cross-process convergence is
//! the idempotent-write contract concurrent_writers proves. Exits
//! after 30 min idle or on a version-skew hello. Wire contract:
//! contracts/DAEMON.md + the daemon_proto goldens.

mod bind;
mod conn;
mod dispatch;
mod idle;
mod replies;
mod state;

use super::auth;
use super::coldstart;
use super::judge::Judge;
use super::proto::socket_name;
use anyhow::{Context, Result};
use interprocess::local_socket::Stream;
use interprocess::local_socket::traits::ListenerExt;
use state::{Shared, touch};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 30 min idle → exit (ADR-003). Overridable for tests only.
fn idle_max() -> Duration {
    crate::config::env_secs("CE_DAEMON_IDLE_SECS", 30 * 60)
}

/// Live connection-thread ceiling: every parked connection costs one
/// thread, and an unauthed local prober must not grow that without
/// bound. At the cap a new connection is dropped on accept — an
/// honest connection error at the client, never a queue. A cap alone
/// was a hole and not a bound: silent peers held their slots for the
/// daemon's whole life and every later hook was the one dropped, so
/// idle.rs puts a deadline on the silence.
const MAX_CONNS: u32 = 64;

/// The daemon inherits the cwd of the hook that spawned it — the
/// project root — and a process's cwd pins its directory on Windows:
/// an idling (30 min) or exiting daemon then refuses the root's
/// deletion. The demo replay raced exactly that under the parallel
/// suite (eject's Bye is answered before the exit completes, and
/// Node's rimraf does not retry a top-level EBUSY). The root travels
/// as an absolute path and every child gets `-C root` or pure pipes,
/// so the daemon leaves for the system temp dir. A relative,
/// path-bearing CE_CORE_BIN is absolutized first (a bare name stays a
/// PATH lookup); called before any thread of this process exists.
fn leave_root() -> std::path::PathBuf {
    if let Ok(bin) = std::env::var("CE_CORE_BIN")
        && Path::new(&bin).is_relative()
        && Path::new(&bin)
            .parent()
            .is_some_and(|d| !d.as_os_str().is_empty())
        && let Ok(abs) = std::path::absolute(&bin)
    {
        // SAFETY: serve's first act, single-threaded by construction —
        // no other thread can read the environment concurrently.
        unsafe { std::env::set_var("CE_CORE_BIN", abs) };
    }
    let away = std::env::temp_dir();
    if let Err(e) = std::env::set_current_dir(&away) {
        eprintln!("ce daemon: cannot leave the root ({}): {e}", away.display());
    }
    std::env::current_dir().unwrap_or(away)
}

/// Serve until idle timeout, shutdown request, or version skew (the
/// latter two exit from the connection thread — dispatch::exit_daemon).
pub fn serve(root: &Path) -> Result<()> {
    let root = std::fs::canonicalize(root).with_context(|| format!("root {}", root.display()))?;
    let cwd = leave_root();
    let name = socket_name(&root);
    let listener = bind::bind(&name)?;
    // after the bind on purpose: the bind is the singleton race, and
    // a loser minting first would lock clients out of the winner
    let token = auth::establish(&root)?;
    let start = Instant::now();
    touch(start);
    spawn_idle_watchdog(start);
    coldstart::init(&root); // after bind: a lost race spawns no build
    eprintln!(
        "ce daemon: serving {} on {name} (cwd {})",
        root.display(),
        cwd.display()
    );
    let shared = Arc::new(Shared {
        root,
        start,
        token,
        judge: Mutex::new(Judge::default()),
    });
    let live = Arc::new(AtomicU32::new(0));
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                touch(start);
                spawn_handler(s, &shared, &live);
            }
            Err(e) => eprintln!("ce daemon: accept: {e}"),
        }
    }
    Ok(())
}

/// Decrements the live-connection count when its thread ends — Drop,
/// so a panicking handler cannot leak its slot.
struct Slot(Arc<AtomicU32>);
impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// One thread per connection, bounded by MAX_CONNS.
fn spawn_handler(stream: Stream, shared: &Arc<Shared>, live: &Arc<AtomicU32>) {
    if live.fetch_add(1, Ordering::Relaxed) >= MAX_CONNS {
        live.fetch_sub(1, Ordering::Relaxed);
        eprintln!("ce daemon: {MAX_CONNS} live connections — dropping a new one");
        return; // stream drops = connection closed
    }
    let slot = Slot(Arc::clone(live));
    let shared = Arc::clone(shared);
    let spawned = std::thread::Builder::new().spawn(move || {
        let _slot = slot; // freed even if handle panics
        if let Err(e) = conn::handle(stream, &shared) {
            eprintln!("ce daemon: connection: {e}");
        }
    });
    if let Err(e) = spawned {
        // the closure (and the slot in it) was dropped, count intact
        eprintln!("ce daemon: connection thread: {e}");
    }
}

/// The watchdog only ever exits the whole process — a stuck request
/// cannot block it because it reads a lock-free timestamp.
fn spawn_idle_watchdog(start: Instant) {
    let max = idle_max();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(max.min(Duration::from_secs(60)));
            let last = state::last_activity_ms();
            let now = start.elapsed().as_millis() as u64;
            if now.saturating_sub(last) > max.as_millis() as u64 {
                eprintln!("ce daemon: idle {max:?} exceeded, exiting");
                std::process::exit(0);
            }
        }
    });
}
