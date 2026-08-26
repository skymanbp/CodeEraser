//! The daemon's connection slots are a bound, not a hole: MAX_CONNS
//! caps how many connection threads may exist, and idle.rs caps how
//! long any one of them may sit waiting on a silent peer. Without the
//! second cap, local processes holding no token at all could pin
//! every slot for the daemon's whole life, every real hook was then
//! dropped at accept, and the gate degraded to fail-open (review
//! 2026-08-20).
//!
//! Alone in its binary (the corelink_deadline reasoning, wall-clock
//! flavor): the cap test needs all 64 silent peers PARKED AT ONCE,
//! and a peer only stays parked for the 5 s pre-hello budget — so the
//! whole connect storm must land inside one deadline window. Inside
//! the merged `it` crate the storm co-runs with 200 threads spawning
//! daemons and indexing repos, the storm stretched past 5 s, early
//! peers were swept before late ones connected, and the cap was never
//! observably full (2/80 loop failures, 2026-08-26). Cargo runs test
//! binaries sequentially, so a root binary is a quiet machine slice.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use std::path::Path;
use std::time::Duration;

#[path = "it/common/mod.rs"]
mod common;
use common::tmp as project_dir;

/// server.rs's MAX_CONNS. Private there and unreachable from an
/// integration test, so the capping loop below refuses to proceed if
/// this copy has drifted low enough to leave a slot free.
const MAX_CONNS: usize = 64;

/// Whether the daemon answers a Ping right now, and why not when it
/// does not. BOTH ends of the deadline ask through this one predicate
/// — the cap engaging is this call failing, the sweeper handing a slot
/// back is the same call succeeding — so the battery states its ping
/// tail once instead of re-pasting daemon_e2e's parked-connection one
/// (dedup gate, this batch). The spawn and shutdown halves already
/// come from common/daemon.rs.
fn pong(root: &Path) -> Result<(), String> {
    match client::request_if_running(root, &Request::Ping) {
        Ok(Response::Pong { .. }) => Ok(()),
        Ok(other) => Err(format!("got {other:?}")),
        Err(e) => Err(format!("{e:#}")),
    }
}

/// Peers that connect and then say nothing must lose their slots on
/// the pre-hello deadline. Before it, these mute peers parked
/// MAX_CONNS threads for the daemon's whole life and the second ping
/// below never got a slot to be answered on.
#[test]
fn silent_peers_stop_pinning_the_connection_cap() {
    let root = project_dir("daemon-silence");
    let child = common::spawn_daemon_ready(&root);
    // A few more than the cap: a slot still winding down from the
    // readiness ping would otherwise leave one free and make the
    // assertions vacuous. Over-cap connections are dropped at accept
    // and cost the daemon nothing.
    let parked: Vec<_> = (0..MAX_CONNS + 4)
        .map(|_| common::raw_daemon_connect(&root))
        .collect();

    // The accept loop counts each connection on its own thread, so
    // give it a moment to fill — an un-engaged cap would prove
    // nothing about the deadline.
    let mut capped = false;
    for _ in 0..20 {
        if pong(&root).is_err() {
            capped = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(capped, "{MAX_CONNS} silent peers never filled the cap");

    // comfortably past the 5 s pre-hello budget and the sweeper tick
    std::thread::sleep(Duration::from_secs(8));
    if let Err(e) = pong(&root) {
        panic!("the deadline must have freed the silent peers' slots: {e}");
    }

    drop(parked);
    common::shutdown_and_wait(&root, child, "daemon after the silence test");
}
