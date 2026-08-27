//! Unit-test scratch directories (compiled under cfg(test) only):
//! the one tmp-dir scaffold the in-crate test modules share — the
//! dedup ratchet caught join::churn_unit growing a second copy of
//! deadcode's observe-test setup, so the scaffold became a throat.

use std::path::PathBuf;

/// Fresh empty scratch dir keyed by tag + pid (parallel-test safe);
/// callers remove it themselves when the assertion needs a clean end.
pub fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ce-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// One per-declaration word case as one line: the source, then
/// ` ⇒`, then every unit `measure` must find, in source order, as
/// `name:letters` with `bit_of` reading each letter (`-` = none); an
/// empty right side asserts that `measure` finds nothing at all. A
/// string per case rather than a (source, table) tuple: a run of such
/// tuples is this repo's most-rhyming token shape, and its own clone
/// gate said so when the visibility tables were drafted — the mention
/// category word measures its tables through this same throat.
pub fn check_word_case(
    lang: crate::scan::lang::Lang,
    case: &str,
    measure: impl Fn(&str, crate::scan::lang::Lang) -> Vec<(String, i64)>,
    bit_of: impl Fn(char) -> i64,
) {
    let (src, want) = case.split_once(" ⇒").expect("case has ` ⇒`");
    let want: Vec<(String, i64)> = want
        .split_whitespace()
        .map(|unit| {
            let (name, letters) = unit.rsplit_once(':').expect("name:letters");
            (name.to_string(), letters.chars().map(&bit_of).sum())
        })
        .collect();
    assert_eq!(measure(src, lang), want, "{src:?}");
}
