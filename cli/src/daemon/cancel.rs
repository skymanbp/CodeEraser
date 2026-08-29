//! The client deadline's TEARDOWN (plan v2.18 step #14, O64). The
//! deadline (client.rs `bounded`) is a worker thread that owns the
//! connection and a main thread that stops waiting; until this
//! module the worker it stopped waiting for stayed parked on its
//! read until the process exited — every hook exits at once, but the
//! GUI's doctor probe is long-lived and kept one parked thread per
//! wedged-daemon event, which DAEMON.md recorded as a caveat. Now the
//! main thread CANCELS the read: Unix shuts the socket down through a
//! duplicated descriptor (shutdown acts on the socket, not the
//! descriptor, so the worker's blocked read returns 0 and it bails
//! "daemon closed the connection"); Windows aborts the in-flight
//! overlapped read on the pipe handle with `CancelIoEx` (interprocess
//! reads named pipes with `ReadFileEx` + an alertable wait, which is
//! why `CancelSynchronousIo` could not touch it), fired again every
//! tick because it only cancels I/O already in flight. A connect that
//! outlasts the deadline has nothing to cancel portably (a full
//! backlog parks `connect` in the kernel), so expiry DETACHES it by
//! name — the honest residue, bounded by the kernel's own connect
//! timeout, counted in the `PARKED` gauge `ce doctor` reports.
//!
//! Ordering closes the reconnect race: `fire` sets the flag FIRST and
//! then cancels the registered stream; `register` stores the new
//! stream FIRST and then reads the flag — a stream registered before
//! the flag is hit by the cancel, one registered after sees the flag.

use super::client::{RETRY_DELAY, SPAWN_RETRIES};
use super::proto::{Request, Response};
use anyhow::{Result, bail, ensure};
use interprocess::local_socket::Stream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering::SeqCst};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long an expired deadline waits for the cancelled worker to
/// come back before detaching it: the worker's own retry budget (a
/// lazy connect may still be inside the spawn loop, which honours
/// the cancel only between attempts) plus a margin for the read to
/// unwind. A reply landing in this window is stale and dropped.
pub(super) const GRACE: Duration =
    Duration::from_millis(RETRY_DELAY.as_millis() as u64 * SPAWN_RETRIES as u64 + 500);
/// `CancelIoEx` aborts only the I/O in flight, so the cancel is
/// re-fired at this tick until the worker returns.
const CANCEL_TICK: Duration = Duration::from_millis(20);

/// Workers the deadline gave up on that have not returned: the leak
/// as a number, read by the doctor document (`parkedWorkers`).
pub static PARKED: AtomicUsize = AtomicUsize::new(0);

const RUNNING: u8 = 0;
const DETACHED: u8 = 1;
const RETURNED: u8 = 2;

#[cfg(windows)]
type Handle = std::os::windows::io::OwnedHandle;
#[cfg(not(windows))]
type Handle = std::os::fd::OwnedFd;

enum Target {
    Starting,
    Connecting,
    Stream(Handle),
}

/// One conversation's cancel point, shared by its worker (which
/// registers what it is blocked on) and the main thread (which fires
/// at the deadline and, failing that, detaches).
pub(super) struct Canceller {
    cancelled: AtomicBool,
    fate: AtomicU8,
    inert: bool,
    target: Mutex<Target>,
}

impl Canceller {
    pub(super) fn new() -> Self {
        Self::with(false)
    }

    /// The counterfactual: a canceller whose `fire` does nothing —
    /// the pre-O64 behaviour, kept so the test can show the gauge
    /// seeing the leak that the live canceller clears.
    #[cfg(test)]
    pub(super) fn inert() -> Self {
        Self::with(true)
    }

    fn with(inert: bool) -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            fate: AtomicU8::new(RUNNING),
            inert,
            target: Mutex::new(Target::Starting),
        }
    }

    fn set(&self, target: Target) {
        *self.target.lock().unwrap_or_else(|e| e.into_inner()) = target;
    }

    /// The worker is about to `connect`: nothing to cancel yet.
    pub(super) fn connecting(&self) {
        self.set(Target::Connecting);
    }

    /// The worker holds `stream` now; a deadline that already passed
    /// refuses here rather than letting the worker read on.
    pub(super) fn register(&self, stream: &Stream) -> Result<()> {
        self.set(Target::Stream(dup(stream)?));
        ensure!(
            !self.cancelled.load(SeqCst),
            "daemon deadline passed while connecting"
        );
        Ok(())
    }

    /// Has the deadline passed? The spawn loop asks between attempts.
    pub(super) fn fired(&self) -> bool {
        self.cancelled.load(SeqCst)
    }

    /// The deadline: flag first, then cancel whatever is registered.
    /// Returns the stage the worker is in when nothing could be
    /// cancelled, for the caller's message.
    pub(super) fn fire(&self) -> Option<&'static str> {
        self.cancelled.store(true, SeqCst);
        if self.inert {
            return None;
        }
        let target = self.target.lock().unwrap_or_else(|e| e.into_inner());
        match &*target {
            Target::Stream(handle) => {
                cancel_io(handle);
                None
            }
            Target::Connecting => Some("connecting"),
            Target::Starting => Some("starting"),
        }
    }

    /// The caller stops waiting for a worker that is still alive: it
    /// counts in `PARKED` until it returns.
    pub(super) fn detach(&self) {
        if self
            .fate
            .compare_exchange(RUNNING, DETACHED, SeqCst, SeqCst)
            .is_ok()
        {
            PARKED.fetch_add(1, SeqCst);
        }
    }

    /// The worker's last act: a detached worker leaves the gauge.
    pub(super) fn returned(&self) {
        if self.fate.swap(RETURNED, SeqCst) == DETACHED {
            PARKED.fetch_sub(1, SeqCst);
        }
    }
}

/// The deadline itself: one negotiate on a worker thread that OWNS
/// the connection, a bounded wait on the main thread, and on expiry
/// the teardown above — cancel, grace, detach. `client::bounded` is
/// the door; the test's counterfactual comes in here with an inert
/// canceller.
pub(super) fn bounded_with(
    root: &Path,
    req: &Request,
    lazy: bool,
    deadline: Duration,
    canceller: Canceller,
) -> Result<Response> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (root, req) = (root.to_path_buf(), req.clone());
    let cancel = Arc::new(canceller);
    let worker = Arc::clone(&cancel);
    std::thread::spawn(move || {
        let _ = tx.send(super::client::negotiate(&root, &req, lazy, &worker));
        worker.returned();
    });
    if let Ok(reply) = rx.recv_timeout(deadline) {
        return reply;
    }
    // the deadline: cancel whatever the worker is blocked on, then
    // give it its own budget to come back — re-firing each tick,
    // since a cancel only reaches the read already in flight
    let mut stage = cancel.fire();
    let until = Instant::now() + GRACE;
    let mut alive = true;
    while alive && Instant::now() < until {
        match rx.recv_timeout(CANCEL_TICK) {
            Err(RecvTimeoutError::Timeout) => stage = cancel.fire(),
            _ => alive = false, // back (its reply is stale), or gone
        }
    }
    let residue = if alive {
        cancel.detach();
        format!(
            " (worker still {} after the {:.1}s grace — detached, see `ce doctor`)",
            stage.unwrap_or("reading"),
            GRACE.as_secs_f64()
        )
    } else {
        String::new()
    };
    bail!(
        "daemon did not answer within {}s{residue} — a wedged daemon is replaced by \
         `ce ping` after it exits, or killed by hand; CE_CLIENT_DEADLINE_SECS overrides",
        deadline.as_secs()
    )
}

/// A duplicate of the worker's handle: the platform enum has one
/// variant per platform, and the inner pipe / socket is what the
/// cancel addresses.
#[cfg(windows)]
fn dup(stream: &Stream) -> Result<Handle> {
    use std::os::windows::io::AsHandle;
    let Stream::NamedPipe(pipe) = stream;
    Ok(pipe.inner().as_handle().try_clone_to_owned()?)
}

#[cfg(not(windows))]
fn dup(stream: &Stream) -> Result<Handle> {
    use std::os::fd::AsFd;
    let Stream::UdSocket(socket) = stream;
    Ok(socket.inner().as_fd().try_clone_to_owned()?)
}

/// Abort the worker's blocked read. Windows: cancel the overlapped
/// I/O in flight on the pipe (the worker's `ReadFileEx` completes
/// with ERROR_OPERATION_ABORTED and `read_line` fails). Unix: shut
/// the socket down — the read returns 0.
#[cfg(windows)]
fn cancel_io(handle: &Handle) {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::IO::CancelIoEx;
    // SAFETY: a live handle we own a duplicate of; a null OVERLAPPED
    // means "every request on this file", and the call touches no
    // memory of ours.
    unsafe {
        CancelIoEx(handle.as_raw_handle(), std::ptr::null());
    }
}

#[cfg(not(windows))]
fn cancel_io(handle: &Handle) {
    if let Ok(fd) = handle.try_clone() {
        let socket = std::os::unix::net::UnixStream::from(fd);
        let _ = socket.shutdown(std::net::Shutdown::Both);
    }
}
