//! The liveness reason in the console's Chinese (plan v2.25, O23). A
//! dead row carries its reason as a code beside the English sentence
//! the core's document names it by (CE.Graph.Document.whyCodes, the
//! machine faces' word since plan v2.32 step 4); this table holds the
//! Chinese the console prints under `--lang zh`, until the console's
//! own words move into the core (plan v2.32 step 5).

use super::DeadRow;

/// The two liveness reasons by code in Chinese. Frozen positions:
/// 0 = the unreferenced verdicts, 1 = the unreachable ones.
pub const WHY_ZH: [&str; 2] = ["无保留入边且无入口标记", "仅被死代码引用且无入口标记"];

impl DeadRow {
    /// The console's word for the code in the pinned language.
    pub fn why_line(&self) -> &str {
        if crate::i18n::zh() {
            WHY_ZH.get(self.why_code).copied().unwrap_or(&self.why)
        } else {
            &self.why
        }
    }
}
