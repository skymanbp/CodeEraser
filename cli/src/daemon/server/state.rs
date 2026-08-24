//! What every connection thread shares, plus the one lock-free
//! activity clock. Split to its own leaf in the headroom sprint:
//! conn.rs and dispatch.rs importing the struct THROUGH server.rs
//! made the daemon a module cycle the graph axis itself billed.

use crate::daemon::judge::Judge;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// What every connection thread shares. The judge mutex doubles as
/// the request-serialization lock (dispatch::build).
pub(super) struct Shared {
    pub(super) root: PathBuf,
    pub(super) start: Instant,
    pub(super) token: String,
    pub(super) judge: Mutex<Judge>,
}

static LAST_ACTIVITY_MS: AtomicU64 = AtomicU64::new(0);

pub(super) fn touch(start: Instant) {
    LAST_ACTIVITY_MS.store(start.elapsed().as_millis() as u64, Ordering::Relaxed);
}

/// The idle watchdog's read half — lock-free, so a stuck request can
/// never block the exit decision.
pub(super) fn last_activity_ms() -> u64 {
    LAST_ACTIVITY_MS.load(Ordering::Relaxed)
}
