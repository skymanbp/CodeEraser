//! Per-connection front door (split from server.rs with the
//! per-connection threads): the bounded pre-hello read, the silence
//! deadline that keeps a mute peer from owning its slot forever, and
//! the 1.1.0 credential gate. Connections are concurrent threads, but
//! REQUESTS are not — dispatch::build serializes on the judge lock,
//! preserving the ADR-003 one-at-a-time discipline the serial accept
//! loop used to provide.

use super::{Shared, dispatch, idle, touch};
use crate::daemon::auth;
use crate::daemon::proto::{DAEMON_PROTO, Request, Response, major};
use anyhow::Result;
use interprocess::local_socket::Stream;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Arc;

/// Longest line an UNAUTHED connection may send: a hello is ~200
/// bytes, so 4 KiB is generous — without the cap a tokenless local
/// prober could stream an endless newline-free line into the buffer
/// (review 2026-08-20 #3). Authed reads stay unbounded: probe and
/// four_class legitimately carry whole file contents, and the token
/// holder already reads everything those bytes could be.
const PRE_AUTH_LINE_MAX: u64 = 4096;

/// One connection's reader. `&Stream`, not `Stream`: the Arc behind
/// it is shared with idle::Watch, which can only end an overdue read
/// through the very handle that read was issued on.
type Conn<'a> = BufReader<&'a Stream>;

/// Serve one connection to completion. Ok = the connection ended
/// (hang-up, refusal, or silence past its deadline); the whole-daemon
/// exits never return.
pub(super) fn handle(stream: Stream, shared: &Shared) -> Result<()> {
    let stream = Arc::new(stream);
    let watch = idle::Watch::arm(&stream);
    let mut reader = BufReader::new(&*stream);
    let mut buf = Vec::new();
    let mut authed = false;
    loop {
        buf.clear();
        if !read_line_capped(&mut reader, &mut buf, authed, &watch)? {
            return Ok(()); // hung up, or refused at the cap
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
        touch(shared.start);
        match gate(&mut reader, &req, &shared.token, &mut authed, &watch)? {
            Flow::Next => continue,
            Flow::CloseConn => return Ok(()),
            Flow::ExitDaemon => dispatch::exit_daemon(shared),
            Flow::Dispatch => {}
        }
        let (resp, keep) = dispatch::build(shared, req);
        reply(reader.get_mut(), &resp)?;
        if !keep {
            dispatch::exit_daemon(shared);
        }
        touch(shared.start);
    }
}

/// One line into `buf`; pre-auth reads stop AT the cap and refuse —
/// the bytes beyond it are never buffered or parsed. False = close
/// the connection. The read is bracketed so the deadline measures
/// time spent waiting for the PEER and never the daemon's own compute
/// between two requests.
fn read_line_capped(
    reader: &mut Conn<'_>,
    buf: &mut Vec<u8>,
    authed: bool,
    watch: &idle::Watch,
) -> Result<bool> {
    watch.waiting(true);
    let read = if authed {
        reader.read_until(b'\n', buf)
    } else {
        (&mut *reader)
            .take(PRE_AUTH_LINE_MAX)
            .read_until(b'\n', buf)
    };
    watch.waiting(false);
    let n = read?;
    if n == 0 {
        return Ok(false); // client hung up
    }
    if !buf.ends_with(b"\n") && !authed && n as u64 == PRE_AUTH_LINE_MAX {
        reply(
            reader.get_mut(),
            &Response::Error {
                message: format!("line exceeds the {PRE_AUTH_LINE_MAX}-byte pre-hello cap"),
            },
        )?;
        return Ok(false);
    }
    Ok(true)
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
/// with the connection closed — the daemon itself lives on. Proving
/// the token is also what widens the silence deadline: everything
/// before it waits on the short pre-hello budget.
fn gate(
    reader: &mut Conn<'_>,
    req: &Request,
    expected: &str,
    authed: &mut bool,
    watch: &idle::Watch,
) -> Result<Flow> {
    if let Request::Hello { proto, token: sent } = req {
        let (resp, verdict) = hello_reply(proto, sent, expected);
        reply(reader.get_mut(), &resp)?;
        return Ok(match verdict {
            Hello::Authed => {
                *authed = true;
                watch.relax();
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
    if !token_eq(sent, expected) {
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

/// Constant-time equality: a short-circuit `==` lets a local prober
/// time its way toward the token byte by byte. The fold's cost
/// depends only on the length — which is public (64 hex chars) —
/// and black_box keeps the accumulator from being short-circuited
/// back in by the optimizer.
fn token_eq(sent: &str, expected: &str) -> bool {
    let (a, b) = (sent.as_bytes(), expected.as_bytes());
    a.len() == b.len()
        && std::hint::black_box(a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y))) == 0
}

fn reply(stream: &mut &Stream, resp: &Response) -> Result<()> {
    let line = serde_json::to_string(resp)?;
    writeln!(stream, "{line}")?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::token_eq;

    /// The fold must still be an equality — timing is unobservable
    /// in a unit test, so this pins the truth table.
    #[test]
    fn token_eq_matches_equality() {
        assert!(token_eq("abc123", "abc123"));
        assert!(!token_eq("abc123", "abc124"), "last byte differs");
        assert!(!token_eq("abc123", "abc12"), "length differs");
        assert!(!token_eq("", "x"), "empty vs non-empty");
        assert!(token_eq("", ""), "both empty");
    }
}
