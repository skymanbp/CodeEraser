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
