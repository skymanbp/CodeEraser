//! The core link's READ side: a reader thread feeds a channel so the
//! reply wait can carry a deadline. `read_line` on the child's stdout
//! was unbounded — a wedged core held the daemon (and every hook
//! queued behind its serial accept loop) forever, and the idle
//! watchdog could not help because activity is stamped BEFORE
//! dispatch (2026-08-19 review, finding 20). The write side stays
//! direct: request lines are far below the pipe buffer, so a blocked
//! write needs a core that stopped draining mid-handshake — the read
//! deadline reaps exactly that core one reply later.

use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// Hard ceiling for ONE reply line, not a budget: every measured
/// judgment answers in single-digit seconds even cold (PERF-BUDGET),
/// so anything past this is a wedge, never work.
pub(super) fn deadline() -> Duration {
    crate::config::env_secs("CE_CORE_DEADLINE_SECS", 60)
}

/// Pump the child's stdout into a channel, line by line. EOF ends the
/// thread and the hung-up channel tells the waiter; a read error is
/// forwarded once and ends it too. Detached on purpose: after a kill
/// the read returns 0 and the thread leaves by itself.
pub(super) fn reader(out: ChildStdout) -> Receiver<std::io::Result<String>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let mut lines = BufReader::new(out);
        loop {
            let mut line = String::new();
            match lines.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(Ok(line.trim().to_string())).is_err() {
                        break; // the Link is gone; stop pumping
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    break;
                }
            }
        }
    });
    rx
}

/// One reply line within the deadline. On timeout the core is REAPED
/// here — kill then wait, so a wedge cannot hold the exe (Windows
/// file lock) or leak a process — and the caller sees an Err, which
/// every family already treats as its degraded path (A9f).
pub(super) fn next_line(
    rx: &Receiver<std::io::Result<String>>,
    deadline: Duration,
    child: &mut Child,
) -> Result<String, String> {
    match rx.recv_timeout(deadline) {
        Ok(Ok(line)) => Ok(line),
        Ok(Err(e)) => Err(format!("read: {e}")),
        Err(RecvTimeoutError::Disconnected) => Err("core closed the pipe".into()),
        Err(RecvTimeoutError::Timeout) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(format!(
                "core reply deadline exceeded ({deadline:?}) — core killed"
            ))
        }
    }
}
