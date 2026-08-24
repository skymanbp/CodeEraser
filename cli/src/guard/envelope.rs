//! The PreToolUse hook envelope this guard decodes — split to its
//! own leaf in the headroom sprint: budget.rs importing it THROUGH
//! the guard hub made the pair a module cycle the graph axis itself
//! billed on the self-scan.

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Envelope {
    #[serde(default)]
    pub(super) hook_event_name: String,
    #[serde(default)]
    pub(super) tool_name: String,
    #[serde(default)]
    pub(super) cwd: String,
    /// Claude Code stamps this on every hook event. Carried into the
    /// observe feed (schema: hookio::OBSERVE_SCHEMA) because the M4
    /// evaluation set is partitioned BY SESSION — both the D2-2 count
    /// and the D2-1 purity rule are unanswerable without it.
    #[serde(default)]
    pub(super) session_id: String,
    #[serde(default)]
    pub(super) tool_input: ToolInput,
}

#[derive(Deserialize, Default)]
pub(super) struct ToolInput {
    #[serde(default)]
    pub(super) file_path: String,
    /// Write payloads carry `content`; Edit payloads carry
    /// `new_string` (captured contract) — the added text either way.
    #[serde(default)]
    pub(super) content: String,
    #[serde(default)]
    pub(super) new_string: String,
    /// Edit-only (captured contract): what `new_string` replaces, and
    /// whether every occurrence is replaced — enough to apply the
    /// edit in memory for an exact post-write line count.
    #[serde(default)]
    pub(super) old_string: String,
    #[serde(default)]
    pub(super) replace_all: bool,
}
