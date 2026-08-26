//! The progress face (plan v2.16): a long measurement says so on
//! STDERR, and stdout does not move a byte for it. Both directions
//! are the gate — the counterfactual is not "progress appeared" but
//! "the report is still byte-identical while it does", which is what
//! leaves every machine consumer of these commands untouched.
//!
//! The fixture is the churn batteries' own three-commit history, not
//! the self window: the defect is measured in minutes (102 s for a
//! ONE-day self window before this landed) and a gate that took
//! minutes is a gate nobody runs. What this proves is the WIRING —
//! the phases fire, the words switch language, the two streams stay
//! apart. The wall-clock is PERF-BUDGET's to record.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo(name: &str) -> PathBuf {
    let root = common::tmp(name);
    common::seed_churn_history(&root);
    root
}

/// `CE_PROGRESS` is forced in BOTH directions here rather than left
/// to a terminal test: a TTY-only feature cannot be observed by CI,
/// and a feature nothing observes is one nobody notices breaking.
fn churn(root: &Path, progress: &str, lang: &str, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["churn", ".", "--days", "30", "--format", format])
        .current_dir(root)
        .env("CE_PROGRESS", progress)
        .env("CE_LANG", lang)
        .output()
        .expect("run ce churn")
}

fn stderr_of(out: Output) -> String {
    String::from_utf8(out.stderr).expect("utf8 stderr")
}

#[test]
fn progress_rides_stderr_and_leaves_the_report_alone() {
    let root = repo("progress-streams");
    for format in ["console", "json"] {
        let off = churn(&root, "0", "en", format);
        let on = churn(&root, "1", "en", format);
        assert!(off.status.success() && on.status.success(), "{format} run");
        assert_eq!(
            off.stdout, on.stdout,
            "{format}: the report must be the same bytes with progress armed"
        );
        assert!(
            off.stderr.is_empty(),
            "{format}: an unarmed face wrote {:?}",
            String::from_utf8_lossy(&off.stderr)
        );
        let e = stderr_of(on);
        for phase in ["commits ", "blame survivors"] {
            assert!(e.contains(phase), "{format}: no {phase} phase: {e:?}");
        }
        assert!(e.starts_with('\r'), "a frame opens with a carriage return");
        assert!(e.ends_with('\r'), "the span erases its line on the way out");
    }
}

/// The whole reason the words are not on the wire (plan v2.15): a
/// progress line cast in English at the measurement layer is one no
/// lookup switch could reach.
#[test]
fn the_zh_face_paints_chinese() {
    let e = stderr_of(churn(&repo("progress-zh"), "1", "zh", "console"));
    for phase in ["提交 ", "存活归因"] {
        assert!(e.contains(phase), "zh {phase} phase missing: {e:?}");
    }
    assert!(!e.contains("commits "), "an English frame leaked: {e:?}");
}

/// A shorter frame must blank what it replaces: the window phase is
/// 29 cells wide and `commits 0/3` is 11, so an unpadded repaint
/// would leave `he commit window` on screen behind it. The cell
/// arithmetic itself is unit-tested beside the function; this frame
/// is ASCII, so bytes and cells agree and the gate reads the stream
/// directly.
#[test]
fn a_repaint_blanks_the_frame_it_replaces() {
    let e = stderr_of(churn(&repo("progress-erase"), "1", "en", "console"));
    let frames: Vec<&str> = e.split('\r').filter(|f| !f.is_empty()).collect();
    let (i, wide) = frames
        .iter()
        .enumerate()
        .find(|(_, f)| f.starts_with("enumerating"))
        .map(|(i, f)| (i, f.len()))
        .expect("the window phase paints first");
    let next = frames.get(i + 1).expect("a phase follows the window");
    assert!(
        next.len() >= wide,
        "unpadded repaint leaves a tail: {next:?} under a {wide}-cell frame"
    );
    // without this the case goes vacuous the day the successor grows
    assert!(
        next.trim_end().len() < wide,
        "this gate only bites while the successor is the shorter frame"
    );
}
