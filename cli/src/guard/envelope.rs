//! The PreToolUse hook envelope this guard decodes — split to its
//! own leaf in the headroom sprint: budget.rs importing it THROUGH
//! the guard hub made the pair a module cycle the graph axis itself
//! billed on the self-scan. Every field is optional under the
//! captured contract, so each struct says `#[serde(default)]` once,
//! at the container (the derived `Default` is what a field-level
//! attribute would fill in anyway): one attribute per field made the
//! two field lists a periodic pair the clone gate billed at two
//! alignments once the id joined (plan v2.30 step 5b-7).

use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct Envelope {
    pub(super) hook_event_name: String,
    pub(super) tool_name: String,
    pub(super) cwd: String,
    /// Claude Code stamps this on every hook event. Carried into the
    /// observe feed (schema: hookio::OBSERVE_SCHEMA) because the M4
    /// evaluation set is partitioned BY SESSION — both the D2-2 count
    /// and the D2-1 purity rule are unanswerable without it.
    pub(super) session_id: String,
    /// The tool call's identity, stamped on every tool event by Claude
    /// Code (the captured contract; the hooks reference has PostToolUse
    /// carry the same value): every feed line of the event names it,
    /// and the PostToolUse leg's `settled` line joins them by it (plan
    /// v2.30 step 5b). Empty when the envelope carries none.
    pub(super) tool_use_id: String,
    pub(super) tool_input: ToolInput,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct ToolInput {
    pub(super) file_path: String,
    /// Write payloads carry `content`; Edit payloads carry
    /// `new_string` (captured contract) — the added text either way.
    pub(super) content: String,
    pub(super) new_string: String,
    /// Edit-only (captured contract): what `new_string` replaces, and
    /// whether every occurrence is replaced — enough to apply the
    /// edit in memory for an exact post-write line count.
    pub(super) old_string: String,
    pub(super) replace_all: bool,
}
