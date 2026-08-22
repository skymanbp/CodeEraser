//! The bind IS the singleton arbiter (ADR-003): one daemon per root.
//! The stale-corpse reclaim (server/bind.rs — macOS pseudo-namespace
//! socket files left behind by process::exit) discriminates live from
//! dead by CONNECTING, never by unlinking first, so a live listener's
//! name is not a loser's to take. The corpse arm itself is
//! macOS-only and rides the observe-feed golden on the tag/nightly
//! macOS leg; the live arm is provable everywhere and is pinned here.

mod common;

/// With a live daemon serving the root, a second `ce daemon` must
/// refuse and exit — and the winner must stay untouched.
#[test]
fn a_second_daemon_refuses_a_live_root() {
    let root = common::tmp("daemon-second");
    let child = common::spawn_daemon_ready(&root);
    let out = common::run_ce(&root, &["daemon", "."]);
    assert!(!out.status.success(), "second daemon must refuse the bind");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("already serving"), "stderr: {err}");
    common::assert_alive_then_shutdown(&root, child, "daemon after second-bind refusal");
}
