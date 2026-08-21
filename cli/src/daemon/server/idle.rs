//! Per-connection silence deadline — the TIME bound whose byte twin
//! is conn.rs's PRE_AUTH_LINE_MAX. MAX_CONNS caps how many connection
//! threads may exist, but nothing capped how long one may sit mute:
//! 64 connected-and-silent local peers parked 64 threads forever,
//! pinned every slot, and every later hook was then dropped at accept
//! — the gate degraded to fail-open at the request of any local
//! process holding no token at all (review 2026-08-20).
//!
//! interprocess 2.4.3 exposes no ONE portable mechanism, so there are
//! two:
//!
//! * Unix — `Stream::set_recv_timeout` is SO_RCVTIMEO: kernel
//!   enforced, no thread. It bounds each READ rather than the whole
//!   line, so a peer dripping one byte per interval keeps its slot;
//!   PRE_AUTH_LINE_MAX bounds that to 4096 drips before the cap
//!   refuses the connection outright.
//! * Windows — the same call is a hard `ErrorKind::Unsupported`
//!   ("named pipes do not support I/O timeouts", interprocess-2.4.3
//!   src/os/windows/named_pipe/local_socket/stream.rs:53), and the
//!   read underneath is a `ReadFileEx` awaited in an alertable
//!   `SleepEx`, so the only way to end it from outside that thread is
//!   `CancelIoEx` on the same handle. ONE sweeper thread for the
//!   whole daemon walks the live connections: a watchdog per
//!   connection would double the very thread count MAX_CONNS exists
//!   to bound.

use interprocess::local_socket::Stream;
use std::sync::Arc;
use std::time::Duration;

/// A hello is one short line the client already holds when it
/// connects, so a peer that has not spoken in 5 s is not
/// mid-handshake — it is squatting on a slot.
const PRE_HELLO: Duration = Duration::from_secs(5);

/// Past the hello the peer proved it can read .ce/daemon.token, so
/// the budget is generous — still finite, because a wedged token
/// holder must not own a slot for the daemon's whole 30-min life.
const POST_HELLO: Duration = Duration::from_secs(60);

#[cfg(unix)]
pub(super) struct Watch(Arc<Stream>);

#[cfg(unix)]
impl Watch {
    pub(super) fn arm(stream: &Arc<Stream>) -> Self {
        let watch = Self(Arc::clone(stream));
        watch.budget(PRE_HELLO);
        watch
    }

    pub(super) fn relax(&self) {
        self.budget(POST_HELLO);
    }

    /// SO_RCVTIMEO already spans exactly the blocking read, so the
    /// bracket the Windows sweeper needs costs nothing here.
    pub(super) fn waiting(&self, _now: bool) {}

    fn budget(&self, span: Duration) {
        use interprocess::local_socket::traits::Stream as _;
        // best effort: a stream that refuses the option still has
        // MAX_CONNS and PRE_AUTH_LINE_MAX, just no time bound
        let _ = self.0.set_recv_timeout(Some(span));
    }
}

#[cfg(windows)]
pub(super) struct Watch(Arc<Entry>);

#[cfg(windows)]
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// How often the sweeper re-reads every live connection's clock.
#[cfg(windows)]
const TICK: Duration = Duration::from_millis(250);

#[cfg(windows)]
struct Entry {
    /// The very handle the read was issued on: CancelIoEx matches on
    /// the handle, and this Arc keeps it open for as long as the
    /// sweeper can still reach it.
    stream: Arc<Stream>,
    /// `now_ms()` at which the current wait for the peer began; 0 =
    /// not waiting, so the daemon's own compute between two requests
    /// never counts against the peer.
    since_ms: AtomicU64,
    budget_ms: AtomicU64,
}

/// Every live connection, weakly: the connection thread owns the
/// only strong Arc, so a thread that ends drops out of here on the
/// next tick with no unregister call to forget.
#[cfg(windows)]
static LIVE: std::sync::Mutex<Vec<std::sync::Weak<Entry>>> = std::sync::Mutex::new(Vec::new());

/// A daemon that never sees a connection never starts the sweeper.
#[cfg(windows)]
static SWEEPER: std::sync::Once = std::sync::Once::new();

#[cfg(windows)]
impl Watch {
    pub(super) fn arm(stream: &Arc<Stream>) -> Self {
        let entry = Arc::new(Entry {
            stream: Arc::clone(stream),
            since_ms: AtomicU64::new(0),
            budget_ms: AtomicU64::new(PRE_HELLO.as_millis() as u64),
        });
        LIVE.lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Arc::downgrade(&entry));
        SWEEPER.call_once(|| {
            std::thread::spawn(sweep);
        });
        Self(entry)
    }

    pub(super) fn relax(&self) {
        let long = POST_HELLO.as_millis() as u64;
        self.0.budget_ms.store(long, Relaxed);
    }

    /// Brackets the blocking read: the deadline runs only while the
    /// daemon waits for the peer's next byte, so a slow dedup between
    /// two requests can never get its own reply write cancelled.
    pub(super) fn waiting(&self, now: bool) {
        let mark = if now { now_ms() } else { 0 };
        self.0.since_ms.store(mark, Relaxed);
    }
}

/// Monotonic millis, never 0 — a wait that begins in the first
/// millisecond must not read as the not-waiting sentinel.
#[cfg(windows)]
fn now_ms() -> u64 {
    static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let origin = ORIGIN.get_or_init(std::time::Instant::now);
    origin.elapsed().as_millis() as u64 + 1
}

/// Cancels overdue reads and reaps the entries whose connection
/// thread has ended. Dropped connections lose their last Arc, so the
/// Weak fails to upgrade and the registry stays bounded by MAX_CONNS.
#[cfg(windows)]
fn sweep() {
    loop {
        std::thread::sleep(TICK);
        let now = now_ms();
        let mut live = LIVE.lock().unwrap_or_else(|p| p.into_inner());
        live.retain(|weak| match weak.upgrade() {
            None => false, // the connection thread ended; reap it
            Some(entry) => {
                let since = entry.since_ms.load(Relaxed);
                if since != 0 && now.saturating_sub(since) > entry.budget_ms.load(Relaxed) {
                    cancel(&entry.stream);
                }
                true
            }
        });
    }
}

/// Ends whatever read the connection thread is parked in. Repeated
/// every tick until the entry is reaped: the thread may have been
/// between operations when the deadline passed, and a cancel with
/// nothing pending is a no-op (ERROR_NOT_FOUND), not a mistake.
///
/// The reverse — the peer's byte landing in the microseconds between
/// the sweeper reading the clock and calling this — aborts that
/// connection's next write instead of its read. It costs the peer a
/// connection it had already overstayed, and costs no other
/// connection anything, which is why the read path stays lock-free.
#[cfg(windows)]
fn cancel(stream: &Stream) {
    use std::os::windows::io::{AsHandle, AsRawHandle};
    let handle = match stream {
        Stream::NamedPipe(pipe) => pipe.as_handle().as_raw_handle(),
    };
    // SAFETY: `stream` is borrowed from an Arc held across this call,
    // so `handle` is open; CancelIoEx only marks I/O this process
    // issued on it and writes nothing back through the null
    // OVERLAPPED.
    unsafe { CancelIoEx(handle, std::ptr::null_mut()) };
}

// Declared here rather than imported: cli/Cargo.toml asks windows-sys
// for Win32_Foundation and Win32_System_Console only, and the
// Win32_System_IO feature carrying CancelIoEx is on the graph purely
// because interprocess happens to request it — an invisible
// cross-crate coupling that would break this file the day that crate
// changes its feature list.
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CancelIoEx(handle: *mut std::ffi::c_void, overlapped: *mut std::ffi::c_void) -> i32;
}
