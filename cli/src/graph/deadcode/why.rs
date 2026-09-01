//! The liveness reason vocabulary (plan v2.25, O23). A dead row carries
//! its reason as a CODE; this table is the single owner of what each
//! code says, in both languages. The English half is the machine
//! faces' word (`why`, beside `whyCode` in every report), the Chinese
//! half is what the console prints under `--lang zh` — before this
//! table the sentence was minted in English at measurement time and
//! rode into the Chinese console and the GUI's Chinese face untranslated.

use super::DeadRow;

/// The two liveness reasons by code, English beside Chinese. Frozen
/// positions: 0 = the unreferenced verdicts, 1 = the unreachable ones.
pub const WHY_CODES: [(&str, &str); 2] = [
    (
        "no kept in-edge and no entry flag",
        "无保留入边且无入口标记",
    ),
    (
        "referenced only from dead code; no entry flag",
        "仅被死代码引用且无入口标记",
    ),
];

impl DeadRow {
    /// The English sentence — the machine faces' word for the code.
    pub fn why(&self) -> &'static str {
        WHY_CODES[self.why_code].0
    }

    /// The console's word for the code in the pinned language.
    pub fn why_line(&self) -> &'static str {
        let (en, zh) = WHY_CODES[self.why_code];
        crate::i18n::t(en, zh)
    }
}
