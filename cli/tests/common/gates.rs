//! Gate e2e machinery: the core-binary env read and the red/green
//! stanza every --check/--core gate test walks — split from
//! common/mod.rs when the review-repair helpers pushed it past the
//! 300-line dogfood ceiling (the gitio/hooks/ladder submodule
//! pattern).

use std::path::Path;

/// The core binary every judgment e2e drives — ONE env read (the
/// self-ratchet caught the third pasted expect when the scan gate
/// landed; baseline_bridge and the dedup gate carried the first
/// two).
pub fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

/// Red-then-green gate e2e through the real binaries: run expecting
/// exit 1 with `needle` on the chosen stream, apply `repair`, run
/// again expecting exit 0 — the stanza every --check/--core gate
/// test walks (the self-ratchet caught the second pasted copy when
/// the scan gate landed next to the dedup gate).
pub fn gate_red_green(
    dir: &Path,
    run: &dyn Fn(&Path) -> std::process::Output,
    needle: &str,
    on_stderr: bool,
    repair: &dyn Fn(),
) {
    let red = run(dir);
    assert!(!red.status.success(), "red half must exit 1: {red:?}");
    let hay =
        String::from_utf8_lossy(if on_stderr { &red.stderr } else { &red.stdout }).into_owned();
    assert!(hay.contains(needle), "the gate names itself: {hay}");
    repair();
    let green = run(dir);
    assert!(green.status.success(), "green half passes: {green:?}");
}
