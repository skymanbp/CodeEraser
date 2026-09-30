//! The PreToolUse hook envelope this guard decodes — split to its
//! own leaf in the headroom sprint: budget.rs importing it THROUGH
//! the guard hub made the pair a module cycle the graph axis itself
//! billed on the self-scan. Every field is optional under the
//! captured contract, so each struct says `#[serde(default)]` once,
//! at the container (the derived `Default` is what a field-level
//! attribute would fill in anyway): one attribute per field made the
//! two field lists a periodic pair the clone gate billed at two
//! alignments once the id joined (plan v2.30 step 5b-7).

use crate::config::Config;
use crate::scan::lang::Lang;
use serde::Deserialize;
use std::path::Path;

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

impl Envelope {
    /// The write's target as a text class measures it (the tombstone
    /// and the flow class alike): a judged language, inside the
    /// config's walk; and its text before this write — on disk through
    /// the bounded read, empty for a file that does not exist yet.
    /// None = not this class's write, or a before side it cannot read.
    pub(super) fn judged_pair(&self, root: &Path, cfg: Option<&Config>) -> Option<(Lang, String)> {
        let path = Path::new(&self.tool_input.file_path);
        let lang = Lang::judged_path(path)?;
        if cfg.is_some_and(|c| !crate::scan::walk::in_scope(root, path, &c.exclude)) {
            return None;
        }
        let before = crate::tombstone::texts::read_capped(path)
            .or_else(|| (!path.exists()).then(String::new))?;
        Some((lang, before))
    }
}
