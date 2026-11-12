//! The core link's READ side: a reader thread feeds a channel so the
//! reply wait can be watched. `read_line` on the child's stdout was
//! unbounded — a wedged core held the daemon (and every hook queued
//! behind its serial accept loop) forever, and the idle watchdog could
//! not help because activity is stamped BEFORE dispatch (2026-08-19
//! review, finding 20). The write side is watched too: a request line
//! can be far larger than the pipe buffer (18,052,044 B measured, plan
//! v2.35 phase D), so a core that stops draining holds the writer, and
//! session.rs writes from its own thread under the same rule (watch.rs).

use std::io::{BufRead, BufReader, Read};
use std::process::ChildStdout;
use std::sync::mpsc::{Receiver, channel};

/// Byte ceiling for ONE reply line — a MEMORY bound, deliberately
/// twice the protocol's line (core/app/CE/Protocol/Version.hs::maxLineBytes,
/// the `limits.caps.line_bytes` the core states), so no in-contract reply
/// is ever cut short; a unit leg reads the tables golden and holds the
/// two together. The liveness rule alone did not bound the pump: a
/// core emitting an endless newline-free line grew the String for the
/// whole wait, unbounded, while the waiter sat waiting for a message
/// that could not arrive (2026-08-20 protocol review).
const MAX_REPLY_BYTES: u64 = 256 * 1024 * 1024;

/// Pump the child's stdout into a channel, line by line. EOF ends the
/// thread and the hung-up channel tells the waiter; a read error is
/// forwarded once and ends it too. Detached on purpose: after a kill
/// the read returns 0 and the thread leaves by itself.
pub(super) fn reader(out: ChildStdout) -> Receiver<std::io::Result<String>> {
    pump(out, MAX_REPLY_BYTES)
}

/// The pump proper, over any reader and any cap so the ceiling is
/// testable without a 256 MiB fixture.
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

#[cfg(test)]
#[path = "../../tests/unit/corelink/pipe.rs"]
mod tests;
