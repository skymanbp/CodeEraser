//! One core process past its accepted hello: the child, its reply
//! channel and what the hello said. `Link` (corelink.rs) is a handle on
//! one of these; the process slot (shared.rs) keeps one between links.
//! A session that lost its framing — a write or read that failed, a
//! deadline, a reply that does not parse, a desync — is `broken` and is
//! never handed out again; a core's refusal (an error reply echoing the
//! request's id, or the id-less `too_large` of a line the core would not
//! read) leaves it whole, the core being stateless per request.

use super::{Hello, HelloReply, PROTO, pipe};
use serde_json::Value;
use std::io::Write;
use std::process::Child;

pub(super) struct Session {
    child: Child,
    replies: std::sync::mpsc::Receiver<std::io::Result<String>>,
    deadline: std::time::Duration,
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
            deadline: pipe::deadline(),
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
        let parsed = serde_json::from_str(&session.exchange(&line)?)
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
        let reply: Value = serde_json::from_str(&self.exchange(&line)?).map_err(|e| {
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
    /// session.
    fn exchange(&mut self, line: &str) -> Result<String, String> {
        let got = self
            .send(line)
            .and_then(|()| pipe::next_line(&self.replies, self.deadline, &mut self.child));
        self.broken |= got.is_err();
        got
    }

    /// The core's process id (the slot's battery tells sessions apart).
    #[cfg(test)]
    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        let stdin = self.child.stdin.as_mut().ok_or("no stdin pipe")?;
        writeln!(stdin, "{line}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))
    }
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
