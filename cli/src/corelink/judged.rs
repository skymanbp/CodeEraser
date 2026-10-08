//! What every judged family's Rust leg does the same way once it has
//! a link: ask behind the capability gate, and read a reply's degraded
//! posture, tables and counts by name. Promoted when the twelfth family
//! (similar/1) reminted tombstone/1's ask-and-consume shell verbatim —
//! the dedup gate's own promotion rule. Policy never lives here: which
//! tables a family has and what skew means for them stays in that
//! family's wire.rs.

use super::Link;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// One `{kind}.request` behind its capability gate: a core without the
/// family is healthy and answers nothing here, and the absence is
/// NAMED with the proto that minted the family — never read as an
/// empty judgment (A9f).
pub fn ask(
    link: &mut Link,
    cap: &str,
    since: &str,
    kind: &str,
    body: Value,
) -> Result<Value, String> {
    if !link.has(cap) {
        return Err(format!("core offers no {cap} (pre-{since})"));
    }
    link.request(kind, body)
}

/// One request over the process's core session (plan v2.33 W2-text
/// stage Z1: taken for the request, parked again unless the request broke
/// it), behind the gate; a degraded reply is a named refusal, every error
/// prefixed by the capability. The families that ask from deep inside a
/// sweep (resolve/1, bags/1) go through here.
pub fn session_ask((cap, since, kind): (&str, &str, &str), body: Value) -> Result<Value, String> {
    let (mut link, _) = Link::open(crate::tables::core_flag())?;
    let reply = ask(&mut link, cap, since, kind, body).map_err(|e| format!("{cap}: {e}"))?;
    degraded(&reply).map_err(|e| format!("{cap}: {e}"))?;
    Ok(reply)
}

/// One candidate pass a caller consumes WHOLE (plan v2.33 W3): asked
/// behind the gate, a degraded reply refused by name (`what` names the
/// pass it starved), then the reply, its `pairs` table and the named
/// `counts.<key>` in `keys` order. What adds up stays the caller's.
/// A request whose line would pass the protocol's line ceiling is not
/// sent and meets the same named degradation (`fits_line`): the pass is
/// whole-corpus, so it cannot be split without changing its answer.
pub fn whole_pass<P: DeserializeOwned>(
    link: &mut Link,
    (cap, since, kind): (&str, &str, &str),
    body: Value,
    (what, keys): (&str, &[&str]),
) -> anyhow::Result<(Value, P, Vec<u64>)> {
    if let Err(why) = fits_line(kind, &body) {
        anyhow::bail!("{cap} degraded the {what} ({why})");
    }
    let reply = ask(link, cap, since, kind, body).map_err(anyhow::Error::msg)?;
    if let Err(why) = degraded(&reply) {
        anyhow::bail!("{cap} degraded the {what} ({why})");
    }
    let pairs = table(&reply, "pairs").map_err(anyhow::Error::msg)?;
    let counts = keys
        .iter()
        .map(|k| count(&reply, k).map(|n| n as u64))
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::msg)?;
    Ok((reply, pairs, counts))
}

/// Whether `body` as a `{kind}.request` line stays within the line
/// ceiling the core states (`limits.caps.line_bytes`, CE.Limits): the
/// core measures a line before it decodes it, so a longer one could only
/// be answered by a `too_large` that echoes no id (CE.Protocol.respond).
/// The line is counted as the session writes it (session.rs: the body
/// plus the `id` / `proto` / `type` it stamps, the id at its widest),
/// without building it; over the ceiling, the named reason.
fn fits_line(kind: &str, body: &Value) -> Result<(), String> {
    let max = crate::tables::get().limits.caps.line_bytes;
    let stamp = format!(
        r#","id":{},"proto":"{}","type":"{kind}.request""#,
        u64::MAX,
        super::PROTO
    );
    let mut line = Count(stamp.len());
    serde_json::to_writer(&mut line, body).map_err(|e| e.to_string())?;
    if line.0 > max {
        return Err(format!(
            "request line {} B over the {max} B line the core reads; not sent",
            line.0
        ));
    }
    Ok(())
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
pub fn degraded(reply: &Value) -> Result<(), String> {
    if reply["degraded"] == Value::Bool(true) {
        return Err(reply["reason"].as_str().unwrap_or("degraded").to_string());
    }
    Ok(())
}

/// One table of the reply under `key`, decoded; its absence or a
/// malformed element is named by key (a malformed reply is never a
/// healthy one).
pub fn table<T: DeserializeOwned>(reply: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(reply[key].clone()).map_err(|e| format!("{key}: {e}"))
}

/// One `counts.<key>` of the reply, or its named absence.
pub fn count(reply: &Value, key: &str) -> Result<usize, String> {
    reply["counts"][key]
        .as_u64()
        .map(|n| n as usize)
        .ok_or_else(|| format!("counts.{key} missing"))
}
