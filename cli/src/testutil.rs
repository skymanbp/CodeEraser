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
