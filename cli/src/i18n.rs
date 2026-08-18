//! CLI UI-string switch (M8-G3b, ruling ⑦: English is the default,
//! Chinese is a lookup switch). The HARD constraint: the English
//! path returns the exact literal at the call site — every test
//! assertion and every machine consumer of console lines runs under
//! the default and must see identical bytes. Chinese activates only
//! under CE_LANG=zh (env single-source for now; a ce.toml [ui] road
//! can join later without moving any call site). Report JSON is
//! never translated — schemas are the machine face.

use std::sync::OnceLock;

static ZH: OnceLock<bool> = OnceLock::new();

/// Pin the language from the CLI's `--lang` (G3 charter: the flag
/// and CE_LANG are the two selectors, flag wins). Must run before
/// any zh() read — main reads the flag straight off argv ahead of
/// clap so even `--help` renders in the chosen language; a pin that
/// arrives after the first lookup loses (OnceLock), which no caller
/// does today.
pub fn init(lang: Option<&str>) {
    if let Some(l) = lang {
        let _ = ZH.set(l == "zh");
    }
}

/// True exactly when the pinned --lang, else CE_LANG, says zh —
/// resolved once per process.
pub fn zh() -> bool {
    *ZH.get_or_init(|| std::env::var("CE_LANG").is_ok_and(|v| v == "zh"))
}

/// The one string switch: call sites keep their English literal in
/// place (byte-identical default) and carry the Chinese beside it.
pub fn t(en: &'static str, zh_s: &'static str) -> &'static str {
    if zh() { zh_s } else { en }
}

/// One bilingual console line: pick the template by language, fill
/// `{}` holes left to right. Templates are DATA (the DSL stance) —
/// the dual-`println!` branch form the first draft used was a
/// structural twin per line under T2 normalization and the dedup
/// ratchet caught it immediately; one call per line dissolves the
/// class. Sequential `{}` substitution is byte-equal to `format!`
/// for plain positional holes, which is all console lines use.
pub fn line(en: &'static str, zh_t: &'static str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::new();
    let mut rest = if zh() { zh_t } else { en };
    for a in args {
        match rest.split_once("{}") {
            Some((head, tail)) => {
                out.push_str(head);
                out.push_str(&a.to_string());
                rest = tail;
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}
