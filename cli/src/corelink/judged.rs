//! The one kit every judged family reads core replies with, once it has
//! a link: open behind the capability gate, ask, and read a reply's
//! degraded posture, tables, fields and counts by name. Promoted when
//! the twelfth family (similar/1) reminted tombstone/1's ask-and-consume
//! shell verbatim — the dedup gate's own promotion rule; the chunked
//! lockstep loop (lockstep.rs) reads its replies through here too (plan
//! v2.33 step 11a: the two kits were one). Policy never lives here:
//! which tables a family has and what a degraded reply means for it
//! stays in that family's wire.rs.

use super::Link;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// A reply this side cannot read as a judgment: a link that would not
/// open, a capability or key the core does not carry, a degraded reply,
/// a table that does not decode. Its words are the reason; a `String`
/// family and an `anyhow` family both take it by `?`.
#[derive(Debug)]
pub struct Unread(String);

impl std::fmt::Display for Unread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Unread {}

impl From<Unread> for String {
    fn from(u: Unread) -> String {
        u.0
    }
}

/// A link to `core` that carries `cap` — the shared head of every
/// family that opens its own link (the lockstep families, score, scan,
/// structure, trend, the audit verdict).
pub fn open(core: &str, cap: &str) -> Result<Link, Unread> {
    let (link, _hello) = Link::open(core).map_err(Unread)?;
    gate(&link, cap)?;
    Ok(link)
}

/// The capability gate: a core without `cap` is named as skew
/// (crate::corelink::skew), never read as an empty judgment (A9f).
fn gate(link: &Link, cap: &str) -> Result<(), Unread> {
    match link.has(cap) {
        true => Ok(()),
        false => Err(Unread(super::skew(cap))),
    }
}

/// One `{kind}.request` behind its capability gate.
pub fn ask(link: &mut Link, cap: &str, kind: &str, body: Value) -> Result<Value, Unread> {
    gate(link, cap)?;
    link.request(kind, body).map_err(Unread)
}

/// One request over the process's core session (plan v2.33 W2-text
/// stage Z1: taken for the request, parked again unless the request broke
/// it), behind the gate; a degraded reply is a named refusal, every error
/// prefixed by the capability. The families that ask from deep inside a
/// sweep (resolve/1, bags/1) go through here. Their request is whole like
/// a candidate pass's, so it meets the same line check (`ask_whole`): a
/// line the core would answer only with an id-less `too_large` is not
/// sent, the named reason is the error (plan v2.33 step 11b item 6).
pub fn session_ask((cap, kind): (&str, &str), body: Value) -> Result<Value, String> {
    let (mut link, _) = Link::open(crate::tables::core_flag())?;
    let reply = ask_whole(&mut link, (cap, kind), body).map_err(|e| format!("{cap}: {e}"))?;
    degraded(&reply).map_err(|e| format!("{cap}: {e}"))?;
    Ok(reply)
}

/// Why a whole request has no reply: its line passes the ceiling and was
/// not sent (`fits_line`'s named reason), or the link / the core said no.
enum Unsent {
    Line(String),
    Unread(Unread),
}

impl std::fmt::Display for Unsent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unsent::Line(why) => f.write_str(why),
            Unsent::Unread(u) => u.fmt(f),
        }
    }
}

/// `ask` for a request that cannot be split without changing its answer
/// — the two roads that send one (`whole_pass`, `session_ask`) share the
/// line check here, one owner: measured before it is built, refused by
/// name before it is sent.
fn ask_whole(link: &mut Link, (cap, kind): (&str, &str), body: Value) -> Result<Value, Unsent> {
    fits_line(kind, &body).map_err(Unsent::Line)?;
    ask(link, cap, kind, body).map_err(Unsent::Unread)
}

/// One candidate pass a caller consumes WHOLE (plan v2.33 W3): asked
/// behind the gate, a degraded reply refused by name (`what` names the
/// pass it starved), then the reply, its `pairs` table and the named
/// `counts.<key>` in `keys` order. What adds up stays the caller's.
/// A request whose line would pass the protocol's line ceiling is not
/// sent and meets the same named degradation (`ask_whole`): the pass is
/// whole-corpus, so it cannot be split without changing its answer.
pub fn whole_pass<P: DeserializeOwned>(
    link: &mut Link,
    (cap, kind): (&str, &str),
    body: Value,
    (what, keys): (&str, &[&str]),
) -> anyhow::Result<(Value, P, Vec<u64>)> {
    let reply = match ask_whole(link, (cap, kind), body) {
        Ok(reply) => reply,
        Err(Unsent::Line(why)) => anyhow::bail!("{cap} degraded the {what} ({why})"),
        Err(Unsent::Unread(u)) => return Err(u.into()),
    };
    if let Err(why) = degraded(&reply) {
        anyhow::bail!("{cap} degraded the {what} ({why})");
    }
    let pairs = table(&reply, "pairs")?;
    let counts = keys
        .iter()
        .map(|k| count(&reply, k).map(|n| n as u64))
        .collect::<Result<_, _>>()?;
    Ok((reply, pairs, counts))
}

/// Whether `body` as a `{kind}.request` line stays within the line
/// ceiling the core states (`limits.caps.line_bytes`, CE.Limits): the
/// core measures a line before it decodes it, so a longer one could only
/// be answered by a `too_large` that echoes no id (CE.Protocol.respond).
/// The line is counted as the session writes it (session.rs: the body
/// plus the `id` / `proto` / `type` it stamps, the id at its widest),
/// without building it; over the ceiling, the named reason.
pub fn fits_line(kind: &str, body: &Value) -> Result<(), String> {
    let (line, max) = (line_len(kind, body), line_max());
    if line > max {
        return Err(format!(
            "request line {line} B over the {max} B line the core reads; not sent"
        ));
    }
    Ok(())
}

/// The ceiling `fits_line` holds a line to (`limits.caps.line_bytes`).
pub fn line_max() -> usize {
    crate::tables::get().limits.caps.line_bytes
}

/// The bytes of `body` as a `{kind}.request` line, counted the way
/// `fits_line` counts it — the planners that pack several items into one
/// body (fourclass::moves, lockstep) measure the empty body here and each
/// item with `json_len`, so a body they cut stays within the line.
pub fn line_len(kind: &str, body: &Value) -> usize {
    let stamp = format!(
        r#","id":{},"proto":"{}","type":"{kind}.request""#,
        u64::MAX,
        super::PROTO
    );
    stamp.len() + json_len(body)
}

/// The bytes of one value as serde writes it compactly (an item joins an
/// array at that many bytes plus one separator).
pub fn json_len(value: &Value) -> usize {
    let mut line = Count(0);
    // writing a `Value` to a writer that never fails cannot fail
    let _ = serde_json::to_writer(&mut line, value);
    line.0
}

/// A writer that keeps only the number of bytes written to it.
struct Count(usize);

impl std::io::Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A reply's degraded posture, read before any table: the core's named
/// reason (or the bare word) is a named non-judgment.
pub fn degraded(reply: &Value) -> Result<(), Unread> {
    if reply["degraded"] == Value::Bool(true) {
        let why = reply["reason"].as_str().unwrap_or("degraded");
        return Err(Unread(why.to_string()));
    }
    Ok(())
}

/// One table of the reply under `key`, decoded; its absence or a
/// malformed element is named by key (a malformed reply is never a
/// healthy one).
pub fn table<T: DeserializeOwned>(reply: &Value, key: &str) -> Result<T, Unread> {
    serde_json::from_value(reply[key].clone()).map_err(|e| Unread(format!("{key}: {e}")))
}

/// A required field of the reply as it stands, its absence named (a
/// `null` decodes as a table; a missing field is no field).
pub fn field(reply: &Value, key: &str) -> Result<Value, Unread> {
    reply
        .get(key)
        .cloned()
        .ok_or_else(|| Unread(format!("reply missing {key}")))
}

/// One `counts.<key>` of the reply, or its named absence.
pub fn count(reply: &Value, key: &str) -> Result<usize, Unread> {
    reply["counts"][key]
        .as_u64()
        .map(|n| n as usize)
        .ok_or_else(|| Unread(format!("counts.{key} missing")))
}
