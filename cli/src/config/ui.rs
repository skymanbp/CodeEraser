//! `[ui]` — the project's console language (plan v2.29 step 9, O61):
//! the third selector, read where every face reads its config. The
//! order is fixed and small: `--lang` wins, then CE_LANG, then this
//! key; `--help` renders before a project is known and never sees it.
//! Not a knob: the canonical fingerprint drops the table whole
//! (config/canonical.rs rule 6) — which language a sentence is spoken
//! in moves no line.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiCfg {
    /// `en` or `zh`; absent = the flag, then CE_LANG, then English.
    pub lang: Option<String>,
}

impl UiCfg {
    /// A value that is not a console language is refused at load by
    /// name — a mistyped `lang` must never look armed (the tier lesson).
    pub fn fault(&self) -> Option<String> {
        match self.lang.as_deref() {
            None | Some("en") | Some("zh") => None,
            Some(other) => Some(format!(
                "[ui] lang = {other:?} is not a console language — en or zh"
            )),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/ui.rs"]
mod tests;
