//! The deadcode family's own table, `[deadcode]` (plan v2.35, the
//! dead-file amendment, phase 5B). One key:
//!
//! - `block` — the root states that fail `ce deadcode --check` and
//!   `ce check` (`dead_added`), out of `"orphaned"` (a file the graph
//!   `ce-baseline.json` records held and did not call unreferenced) and
//!   `"born"` (a file that baseline's graph did not hold). Default empty:
//!   an unreferenced file is listed and blocks nothing. `"standing"` is
//!   refused with its reason — the inventory never blocks: declare an
//!   entry or delete the file. The states and the comparison are the
//!   core's (CE.Graph.Point); this side reads the list and sends its
//!   mask (bit i = the i-th of `BLOCKABLE`, the core's `blockStates`).
//!
//! A knob of the canonical form like every other (config/canonical.rs):
//! the set serializes sorted and unique, so order and repeats move
//! nothing, the default (an empty list) is silence to `knobs_digest`,
//! and a declared list moves it by name.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The blockable states by mask bit (the core's
/// CE.Graph.Cost.blockStates; a unit leg holds the two lists equal).
pub const BLOCKABLE: [&str; 2] = ["orphaned", "born"];

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeadcodeCfg {
    pub block: BTreeSet<String>,
}

impl DeadcodeCfg {
    /// The load-throat refusal: `standing` with its reason, any other
    /// word outside the two by name.
    pub(crate) fn fault(&self) -> Option<String> {
        self.block.iter().find_map(|s| match s.as_str() {
            "standing" => Some(
                "ce.toml [deadcode] block: \"standing\" cannot be blocked — the inventory never \
                 blocks: declare an entry or delete the file"
                    .to_string(),
            ),
            s if !BLOCKABLE.contains(&s) => Some(format!(
                "ce.toml [deadcode] block: unknown state {s:?} (the blockable states are \
                 \"orphaned\" and \"born\")"
            )),
            _ => None,
        })
    }

    /// The `deadBlock` mask graph/1 reads: bit i = BLOCKABLE[i]. Valid
    /// by load (`fault`), so a bare fold.
    pub fn mask(&self) -> u8 {
        BLOCKABLE
            .iter()
            .enumerate()
            .filter(|(_, s)| self.block.contains(**s))
            .fold(0, |m, (i, _)| m | (1 << i))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/deadcode.rs"]
mod tests;
