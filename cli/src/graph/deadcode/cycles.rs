//! The graph reply's `cycles` table read once (v2.35 imports lane: the
//! screen's canvas and the verdict roads each decoded it): [[sccId,
//! [nodeIdx..]]] as it rides; the canvas flattens it, the score and join
//! roads keep its singletons (deadcode.rs re-exports `self_loop_nodes`).

use anyhow::{Context, Result};
use serde_json::Value;

/// The table as it rides; a malformed row is named.
pub fn cycle_table(reply: &Value) -> Result<Vec<(i64, Vec<i64>)>> {
    serde_json::from_value(reply["cycles"].clone()).context("cycle rows")
}

/// The node indices the graph reply lists as singleton cycles —
/// non-empty only at `sccFloor` 1, where a lone node is a cycle
/// exactly when it carries a self-arc (6.4.0, O59).
pub fn self_loop_nodes(reply: &Value) -> Result<Vec<i64>> {
    Ok(cycle_table(reply)?
        .into_iter()
        .filter(|(_, m)| m.len() == 1)
        .map(|(_, m)| m[0])
        .collect())
}
