//! CLI UI-string switch (M8-G3b, ruling ⑦: English is the default,
//! Chinese is a lookup switch). The HARD constraint: the English
//! path returns the exact literal at the call site — every test
//! assertion and every machine consumer of console lines runs under
//! the default and must see identical bytes. Chinese activates by
//! three selectors in a fixed order — the global `--lang zh` flag,
//! else CE_LANG=zh, else the project's ce.toml `[ui] lang = "zh"`
//! (plan v2.29 step 9, O61 built the third; the v2.22 close-out had
//! ruled it not built). Report JSON is never translated — schemas are
//! the machine face.
//!
//! The state is three-valued, not a OnceLock: clap's own parse asks
//! zh() before any project is known, and a first read that PINNED
//! English would have made the third selector unreachable for the
//! command that follows. Unset with no CE_LANG answers English
//! without pinning; the flag and CE_LANG pin on sight; the config
//! pins only what is still unset, and only in a console face
//! (progress::armed) — the GUI, the MCP server, the daemon and every
//! test load the same configs and keep the language they had.

use std::sync::atomic::{AtomicU8, Ordering};

const UNSET: u8 = 0;
const EN: u8 = 1;
const ZH: u8 = 2;

static LANG: AtomicU8 = AtomicU8::new(UNSET);

fn code(lang: &str) -> u8 {
    if lang == "zh" { ZH } else { EN }
}

/// Pin the language from the CLI's `--lang` — the first selector, so
/// it pins unconditionally. main reads the flag straight off argv
/// ahead of clap so even `--help` renders in the chosen language.
pub fn init(lang: Option<&str>) {
    if let Some(l) = lang {
        LANG.store(code(l), Ordering::Relaxed);
    }
}

/// The third selector, offered by `Config::load` with the validated
/// `[ui] lang`. Takes effect only in a console face and only while
/// nothing pinned yet — the flag pinned at startup, CE_LANG pins on
/// its first read and is checked here too, so a project's declaration
/// never overrides the two above it.
pub fn init_from_config(lang: Option<&str>) {
    let Some(l) = lang else { return };
    if crate::progress::armed()
        && LANG.load(Ordering::Relaxed) == UNSET
        && std::env::var_os("CE_LANG").is_none()
    {
        LANG.store(code(l), Ordering::Relaxed);
    }
}

/// Let the project at `root` speak before a command's first console
/// line: `Config::load`'s side effect is the whole point, nothing of
/// the config is kept, and a config the command cannot load is that
/// command's own error to report — silence here.
pub fn pin_project(root: &std::path::Path) {
    let _ = crate::config::Config::load(root);
}

/// True exactly when the pinned --lang, else CE_LANG, else the
/// project's `[ui] lang` says zh. CE_LANG present pins on its first
/// read (a value other than zh is English, as it always was); with
/// nothing set the answer is English and NOT pinned, so a config
/// loaded later can still speak.
pub fn zh() -> bool {
    match LANG.load(Ordering::Relaxed) {
        UNSET => match std::env::var_os("CE_LANG") {
            Some(v) => {
                let zh = v == "zh";
                LANG.store(if zh { ZH } else { EN }, Ordering::Relaxed);
                zh
            }
            None => false,
        },
        state => state == ZH,
    }
}

/// The one string switch: call sites keep their English literal in
/// place (byte-identical default) and carry the Chinese beside it.
pub fn t(en: &'static str, zh_s: &'static str) -> &'static str {
    if zh() { zh_s } else { en }
}

/// One bilingual line chosen BY CODE — the shape plan v2.15 makes a
/// standing pattern: measurement emits a frozen code and each face
/// keeps a table of words for it. Written once here because the
/// second instance of it (index states beside daemon states) was
/// already a structural twin, and every future coded field would have
/// been a third.
///
/// An out-of-range code takes the LAST row, which every table writes
/// as its worst / unknown case: a state this table cannot name must
/// never render as the healthy row at index 0.
pub fn coded(
    state: i64,
    table: &[(&'static str, &'static str)],
    args: &[&dyn std::fmt::Display],
) -> String {
    let (en, zh_t) = usize::try_from(state)
        .ok()
        .and_then(|i| table.get(i))
        .or_else(|| table.last())
        .copied()
        .unwrap_or_default();
    line(en, zh_t, args)
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
