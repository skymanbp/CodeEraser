//! The relocation stage of the duplicate-write class (v2.35 guard lane,
//! GD4): the measuring half's pure pieces — the occurrence count of one
//! matched token run in one token stream, and the relocate/1 wire. The
//! hook (guard/relocate.rs) gathers the streams; the core judges.
//!
//! A write the probe would refuse may be MOVING code: its source was
//! removed first, and the moved function still matches its other
//! surviving twins. The wall the hook mirrors (`ce dedup --check`)
//! counts blocks, and that count did not rise. For each match m with
//! matched run R_m, the hook counts R_m's occurrences in HEAD and in the
//! tree the write leaves, over the files that differ from HEAD (and the
//! target): unchanged files count alike on both sides, so the two sums
//! are the tree-wide counts. `occNow ≤ occHead` (CE.Relocate.Cost) is a
//! relocation; a write all of whose matches are relocations is not
//! refused by the duplicate class.

use crate::corelink::judged;
use serde_json::{Value, json};

pub const CAP: &str = "relocate/1";
pub const KIND: &str = "relocate";

/// The occurrences of `run` (a hash sequence) as a contiguous run in
/// `stream`, every start position counted — overlapping occurrences
/// included, both sides of the comparison counted alike.
pub fn occurrences(stream: &[u64], run: &[u64]) -> u64 {
    if run.is_empty() || run.len() > stream.len() {
        return 0;
    }
    stream.windows(run.len()).filter(|w| *w == run).count() as u64
}

/// The request body: one `[occHead, occNow]` row per match.
pub fn body(rows: &[[u64; 2]]) -> Value {
    json!({ "rows": rows })
}

/// The core's answer: which rows are relocations (ascending request
/// indices) and whether every one is (`all`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub moved: Vec<usize>,
    pub all: bool,
}

/// The reply, consumed: a degraded reply is a named non-judgment; a
/// relocation table that is not an ascending subsequence of the rows
/// sent, or an `all` that disagrees with it, is wire skew.
pub fn consume(reply: &Value, sent: usize) -> Result<Verdict, String> {
    judged::degraded(reply)?;
    let moved: Vec<usize> = judged::table(reply, "relocated")?;
    if !moved.windows(2).all(|w| w[0] < w[1]) || moved.iter().any(|&i| i >= sent) {
        return Err("wire skew: relocated must ascend within the rows sent".into());
    }
    let all = reply["all"]
        .as_bool()
        .ok_or_else(|| "all missing or not a boolean".to_string())?;
    if all != (sent > 0 && moved.len() == sent) {
        return Err("wire skew: all disagrees with the relocation table".into());
    }
    Ok(Verdict { moved, all })
}

#[cfg(test)]
#[path = "../../tests/unit/dedup/relocate.rs"]
mod tests;
