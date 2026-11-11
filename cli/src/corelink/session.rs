//! One core process past its accepted hello: the child, its reply
//! channel and what the hello said. `Link` (corelink.rs) is a handle on
//! one of these; the process slot (shared.rs) keeps one between links.
//! A session that lost its framing — a write or read that failed, a
//! deadline, a reply that does not parse, a desync — is `broken` and is
//! never handed out again; a core's refusal (an error reply echoing the
//! request's id, or the id-less `too_large` of a line the core would not
//! read) leaves it whole, the core being stateless per request.

use super::watch::{self, Sample, Verdict, Watch};
use super::{Hello, HelloReply, PROTO, cputime, pipe};
use serde_json::Value;
use std::io::Write;
use std::process::{Child, ChildStdin};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::{Duration, Instant};

/// How often an open exchange is read by the rule (watch.rs). A reply or
/// the writer's end wakes the wait at once; the tick only bounds how late
/// a stall or an overspend is seen.
const TICK: Duration = Duration::from_millis(250);

/// Request bytes per write, each counted once it is handed to the pipe:
/// a core that drains a large request shows progress while it reads.
const CHUNK: usize = 64 * 1024;

pub(super) struct Session {
    child: Child,
    replies: Receiver<std::io::Result<String>>,
    /// The rule's S and B (watch.rs), read once per session.
    stall: Duration,
    budget: Duration,
    /// The hello the core answered: its capabilities and its own proto
    /// (what a verdict replayed from a cache was judged under,
    /// dedup/t3/cache.rs), handed to every link that takes the session.
    pub(super) hello: HelloReply,
    /// The core it runs, as the process slot keys it (shared::key).
    pub(super) key: String,
    pub(super) broken: bool,
    next_id: u64,
}

impl Session {
    /// Spawn `core` and perform the handshake.
    pub(super) fn open(core: &str, key: String) -> Result<Session, String> {
        let mut child = super::spawn(core)?;
        let replies = pipe::reader(child.stdout.take().ok_or("no stdout pipe")?);
        let mut session = Session {
            child,
            replies,
            stall: watch::stall(),
            budget: watch::budget(),
            hello: HelloReply::default(),
            key,
            broken: false,
            next_id: 0,
        };
        let hello = Hello {
            proto: PROTO,
            r#type: "hello",
            client: "ce",
            version: env!("CARGO_PKG_VERSION"),
        };
        let line = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        let parsed = serde_json::from_str(&session.exchange(&line, "hello")?)
            .map_err(|e| format!("bad hello reply: {e}"))?;
        session.hello = super::validate(parsed)?;
        crate::tables::check_hello(core, session.hello.tables_digest)?;
        Ok(session)
    }

    /// One `{kind}.request` line out, one `{kind}.result` line in.
    /// Stamps proto/type/id; a reply that does not echo the id or
    /// carry the expected type is a desync — the caller falls back to
    /// L1, visibly (A9f).
    pub(super) fn request(&mut self, kind: &str, mut body: Value) -> Result<Value, String> {
        self.next_id += 1;
        let obj = body
            .as_object_mut()
            .ok_or("request body must be an object")?;
        obj.insert("proto".into(), PROTO.into());
        obj.insert("type".into(), format!("{kind}.request").into());
        obj.insert("id".into(), self.next_id.into());
        let line = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        let waiting = format!("{kind}.request ({} bytes)", line.len());
        let reply: Value = serde_json::from_str(&self.exchange(&line, &waiting)?).map_err(|e| {
            self.broken = true;
            format!("bad reply: {e}")
        })?;
        let expected = format!("{kind}.result");
        // an error reply that echoes our id is a REFUSAL, not a
        // desync: surface the core's named reason (review C4 — the
        // knob roads put ce.toml values behind these messages, and
        // "desync" hid every one of them). So is `too_large` with no id:
        // the core refuses a line over its byte ceiling before decoding
        // it (CE.Protocol.respond), so it has no id to echo, answers
        // that line with exactly one line and keeps no state — the
        // framing holds and the session stays whole
        let unread = reply["id"].is_null() && reply["code"] == "too_large";
        if reply["type"] == "error" && (reply["id"] == self.next_id || unread) {
            return Err(format!(
                "core refused {kind}.request: {}: {}",
                reply["code"].as_str().unwrap_or("?"),
                reply["message"].as_str().unwrap_or("?")
            ));
        }
        if reply["type"] != expected.as_str() || reply["id"] != self.next_id {
            self.broken = true;
            return Err(format!("desync: expected {expected} id {}", self.next_id));
        }
        Ok(reply)
    }

    /// One line out and one line back; a failure either way breaks the
    /// session. `waiting` names the line for the rule's message.
    fn exchange(&mut self, line: &str, waiting: &str) -> Result<String, String> {
        // the asking thread's tally, here and not in the writer: the
        // writer is a thread of its own whose count would die with it
        SENT.with(|n| n.set(n.get() + 1));
        let got = self.watched(line, waiting);
        self.broken |= got.is_err();
        got
    }

    /// The line is written on a scoped writer thread, chunk by chunk,
    /// while this thread waits for the reply and then for the writer's
    /// end, reading the rule (watch.rs) every tick over both. Every way
    /// out but a reply kills the core (reaped, so a wedge holds neither
    /// the exe nor a process): its pipe closes, the writer's blocked
    /// write fails and the scope joins it. A core that answers before it
    /// read the whole line is watched by the same rule until it does.
    fn watched(&mut self, line: &str, waiting: &str) -> Result<String, String> {
        let mut stdin = self.child.stdin.take().ok_or("no stdin pipe")?;
        let written = AtomicU64::new(0);
        let total = line.len() as u64 + 1;
        let (done_tx, done) = channel();
        let Session {
            child,
            replies,
            stall,
            budget,
            ..
        } = self;
        let first = Sample {
            at: Duration::ZERO,
            bytes: 0,
            cpu: cputime::cpu_time(child),
        };
        let mut open = Open {
            child,
            watch: Watch::new(*stall, *budget, first),
            start: Instant::now(),
            written: &written,
            total,
            waiting,
        };
        let (pipe_in, count) = (&mut stdin, &written);
        let got = std::thread::scope(|scope| {
            // the sender moves in, so the writer's end hangs the channel up
            scope.spawn(move || done_tx.send(write_counted(pipe_in, line, count)));
            let got = open
                .until(|| reply(replies, &done))
                .and_then(|line| open.until(|| written_out(&done)).map(|()| line));
            if got.is_err() {
                open.kill();
            }
            got
        });
        self.child.stdin = Some(stdin);
        got
    }

    /// The core's process id (the slot's battery tells sessions apart).
    #[cfg(test)]
    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }
}

/// One exchange in flight: the core, the rule over it and what it reads.
struct Open<'a> {
    child: &'a mut Child,
    watch: Watch,
    start: Instant,
    written: &'a AtomicU64,
    total: u64,
    waiting: &'a str,
}

impl Open<'_> {
    /// Poll until `poll` answers, reading the rule after every tick it
    /// does not; a verdict that is not `Wait` kills the core and names
    /// its clause.
    fn until<T>(
        &mut self,
        mut poll: impl FnMut() -> Option<Result<T, String>>,
    ) -> Result<T, String> {
        loop {
            if let Some(got) = poll() {
                return got;
            }
            let now = Sample {
                at: self.start.elapsed(),
                bytes: self.written.load(Ordering::Relaxed),
                cpu: cputime::cpu_time(self.child),
            };
            match self.watch.step(now) {
                Verdict::Wait => {}
                verdict => {
                    self.kill();
                    return Err(self.watch.explain(verdict, self.total, self.waiting));
                }
            }
        }
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The reply, within one tick: a line, a read error, the core gone, or a
/// write that failed first (the writer's end, read without waiting).
fn reply(
    replies: &Receiver<std::io::Result<String>>,
    done: &Receiver<Result<(), String>>,
) -> Option<Result<String, String>> {
    match replies.recv_timeout(TICK) {
        Ok(Ok(line)) => Some(Ok(line)),
        Ok(Err(e)) => Some(Err(format!("read: {e}"))),
        Err(RecvTimeoutError::Disconnected) => Some(Err("core closed the pipe".into())),
        Err(RecvTimeoutError::Timeout) => match done.try_recv() {
            Ok(Err(e)) => Some(Err(e)),
            _ => None,
        },
    }
}

/// The writer's end, within one tick; a writer whose end was already
/// read in `reply` has hung up, which is its end too.
fn written_out(done: &Receiver<Result<(), String>>) -> Option<Result<(), String>> {
    match done.recv_timeout(TICK) {
        Ok(end) => Some(end),
        Err(RecvTimeoutError::Disconnected) => Some(Ok(())),
        Err(RecvTimeoutError::Timeout) => None,
    }
}

/// The line and its newline, `CHUNK` bytes per write, each counted once
/// the pipe took it.
fn write_counted(stdin: &mut ChildStdin, line: &str, written: &AtomicU64) -> Result<(), String> {
    let newline: &[u8] = b"\n";
    for chunk in line.as_bytes().chunks(CHUNK).chain([newline]) {
        stdin.write_all(chunk).map_err(|e| format!("write: {e}"))?;
        written.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }
    stdin.flush().map_err(|e| format!("flush: {e}"))
}

thread_local! {
    /// Lines the thread has asked a core to read (`exchange`, every one).
    static SENT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The count of lines the calling thread has asked a core to read —
/// every request and every hello, on any session — counted on the asking
/// thread, though the bytes leave on the exchange's writer thread. Read
/// by legs that hold "this path asks no core" or "this plan sends n
/// requests"; per thread, so another thread's judgments never move it.
/// No per-family counter.
pub fn sent_on_this_thread() -> usize {
    SENT.with(std::cell::Cell::get)
}

impl Drop for Session {
    fn drop(&mut self) {
        drop(self.child.stdin.take()); // EOF: the polite exit
        // then make exit unconditional — a bare wait() on a wedged
        // core blocked Drop forever, and the core keeps no state a
        // kill could corrupt (pure stdin/stdout judge)
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
