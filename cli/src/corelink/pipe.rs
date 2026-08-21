//! The core link's READ side: a reader thread feeds a channel so the
//! reply wait can carry a deadline. `read_line` on the child's stdout
//! was unbounded — a wedged core held the daemon (and every hook
//! queued behind its serial accept loop) forever, and the idle
//! watchdog could not help because activity is stamped BEFORE
//! dispatch (2026-08-19 review, finding 20). The write side stays
//! direct: request lines are far below the pipe buffer, so a blocked
//! write needs a core that stopped draining mid-handshake — the read
//! deadline reaps exactly that core one reply later.

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// Hard ceiling for ONE reply line, not a budget: every measured
/// judgment answers in single-digit seconds even cold (PERF-BUDGET),
/// so anything past this is a wedge, never work.
pub(super) fn deadline() -> Duration {
    crate::config::env_secs("CE_CORE_DEADLINE_SECS", 60)
}

/// Byte ceiling for ONE reply line — a MEMORY bound, deliberately
/// above the core's own 32 MiB frame ceiling
/// (core/app/CE/Protocol.hs::maxLineBytes), so no in-contract reply
/// is ever cut short. The deadline alone did not bound the pump: a
/// core emitting an endless newline-free line grew the String for the
/// whole deadline window, unbounded, while `next_line` sat waiting
/// for a message that could not arrive (2026-08-20 protocol review).
const MAX_REPLY_BYTES: u64 = 64 * 1024 * 1024;

/// Pump the child's stdout into a channel, line by line. EOF ends the
/// thread and the hung-up channel tells the waiter; a read error is
/// forwarded once and ends it too. Detached on purpose: after a kill
/// the read returns 0 and the thread leaves by itself.
pub(super) fn reader(out: ChildStdout) -> Receiver<std::io::Result<String>> {
    pump(out, MAX_REPLY_BYTES)
}

/// The pump proper, over any reader and any cap so the ceiling is
/// testable without a 64 MiB fixture.
fn pump<R: Read + Send + 'static>(src: R, cap: u64) -> Receiver<std::io::Result<String>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let mut src = BufReader::new(src);
        loop {
            let mut line = String::new();
            // read through take(cap + 1): ONE byte past the ceiling
            // already proves over-length, so the String can never
            // outgrow it no matter what the core streams
            match src.by_ref().take(cap + 1).read_line(&mut line) {
                Ok(0) => break,
                // stopped at the limit with no terminator = over cap.
                // Exactly cap + 1 bytes ending in '\n' is a line OF
                // cap bytes, which is in contract and rides on.
                Ok(n) if n as u64 > cap && !line.ends_with('\n') => {
                    let _ = tx.send(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("core reply line exceeds {cap} bytes"),
                    )));
                    break;
                }
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    /// A newline-free flood is refused at the cap instead of being
    /// accumulated: the pump takes the existing Err road (the
    /// caller's degraded path, A9f) and hangs up, rather than growing
    /// one String for the whole deadline window.
    #[test]
    fn a_newline_free_flood_is_refused_not_accumulated() {
        let rx = super::pump(Cursor::new(vec![b'x'; 200]), 64);
        let first = rx.recv().expect("the pump answers once");
        assert!(first.is_err(), "over-cap line must not arrive as a reply");
        assert!(rx.recv().is_err(), "the pump hangs up after refusing");
    }

    /// …and a line AT the ceiling still rides — the cap detects a
    /// wedge, it does not budget legitimate replies (an off-by-one
    /// here would silently shrink the contract's frame size).
    #[test]
    fn a_line_at_the_ceiling_still_arrives() {
        let mut input = vec![b'y'; 64];
        input.push(b'\n');
        let rx = super::pump(Cursor::new(input), 64);
        assert_eq!(rx.recv().expect("delivered").expect("no error").len(), 64);
    }
}
