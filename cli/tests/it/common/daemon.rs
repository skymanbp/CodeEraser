//! Daemon e2e machinery (split from common/mod.rs when the 1.1.0
//! auth battery's raw-line helper pushed it past the 300-line
//! dogfood wall — the gates.rs/gitio.rs submodule pattern).

use std::path::Path;
use std::process::{Child, Command, Stdio};

/// The spawned daemon, KILLED ON DROP unless a shutdown helper
/// disarms it first. A test that panicked between spawn and shutdown
/// used to leak its daemon; the leaked process kept serving the
/// wiped fixture root (its auth token gone with the wipe), and the
/// next run's spawn starved deterministically on a socket it could
/// never bind nor authenticate to — hit for real during the merge
/// loops (2026-08-26). Tests never touch the inner Child: they pass
/// the guard back into shutdown_and_wait / wait_exit, which disarm.
pub struct DaemonGuard {
    child: Option<Child>,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait(); // reap — no zombie on the panic path
        }
    }
}

/// Spawn the real `ce daemon` for `root` and wait until it answers a
/// ping. Tests must NEVER go through the lazy-spawning
/// `client::request`: it respawns current_exe(), which inside a test
/// harness is the TEST binary — libtest then treats the `daemon` arg
/// as a name filter and runs `*_daemon` tests NESTED, wiping shared
/// tmp dirs and double-serving sockets mid-test (the Windows-CI
/// cold-start flake class). `request_if_running` never spawns; a
/// connect failure is a loud test failure, not silent process spray.
/// Daemon stderr is inherited so CI logs show its cold-start lines.
pub fn spawn_daemon_ready(root: &Path) -> DaemonGuard {
    use codeeraser::daemon::{client, proto::Request};
    let child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("daemon")
        .arg(root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn ce daemon");
    let guard = DaemonGuard { child: Some(child) };
    // 30 s, not 5: readiness is a liveness wait, not a latency claim,
    // and inside the merged `it` crate a cold daemon start co-runs
    // with the whole suite's thread pool — the old 5 s window blew
    // once in 80 amplified loops (2026-08-26). A dead daemon still
    // fails loudly, just later. The panic path needs no manual kill:
    // the guard's Drop reaps.
    for _ in 0..300 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request_if_running(root, &Request::Ping).is_ok() {
            return guard;
        }
    }
    panic!("daemon never came up");
}

/// A raw connection to the daemon for `root`, bypassing the client
/// (its hello/token/respawn behaviors are exactly what the auth,
/// skew, stall, and oversize batteries need to step around).
pub fn raw_daemon_connect(root: &Path) -> std::io::BufReader<interprocess::local_socket::Stream> {
    use interprocess::local_socket::traits::Stream as _;
    use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
    let ns = codeeraser::daemon::proto::socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    std::io::BufReader::new(Stream::connect(ns).expect("connect"))
}

/// One raw NDJSON line → the daemon's one reply line.
pub fn raw_daemon_line(
    root: &Path,
    req: &codeeraser::daemon::proto::Request,
) -> codeeraser::daemon::proto::Response {
    use std::io::{BufRead, Write};
    let mut conn = raw_daemon_connect(root);
    writeln!(
        conn.get_mut(),
        "{}",
        serde_json::to_string(req).expect("ser")
    )
    .expect("write");
    conn.get_mut().flush().expect("flush");
    let mut reply = String::new();
    conn.read_line(&mut reply).expect("read");
    serde_json::from_str(reply.trim()).expect("parse")
}

/// Ask the daemon for `dir` to shut down (ignore errors — may be
/// gone). Never the lazy-spawning path: spawning a daemon in order to
/// shut it down would be absurd, and in a test harness it sprays
/// nested test-binary processes (see spawn_daemon_ready).
pub fn shutdown_daemon(dir: &Path) {
    use codeeraser::daemon::{client, proto::Request};
    let _ = client::request_if_running(dir, &Request::Shutdown);
}

/// The daemon must still ANSWER (it survived whatever the test threw
/// at it), then take the clean shutdown — the shared tail of the
/// refusal-and-survival cases (auth refusals, second-bind refusal).
pub fn assert_alive_then_shutdown(root: &Path, guard: DaemonGuard, what: &str) {
    use codeeraser::daemon::{client, proto::Request, proto::Response};
    let r = client::request_if_running(root, &Request::Ping).expect("ping the survivor");
    assert!(matches!(r, Response::Pong { .. }), "got {r:?}");
    shutdown_and_wait(root, guard, what);
}

/// Clean daemon shutdown (Bye asserted) then reaped exit — the
/// shared tail of every daemon e2e case.
pub fn shutdown_and_wait(root: &Path, guard: DaemonGuard, what: &str) {
    use codeeraser::daemon::{client, proto::Request, proto::Response};
    match client::request_if_running(root, &Request::Shutdown).expect("shutdown") {
        Response::Bye => {}
        other => panic!("expected bye, got {other:?}"),
    }
    wait_exit(guard, what);
}

/// Wait ~5s for the daemon to exit; kill it and panic on timeout
/// (the disarmed guard's Drop is inert, so the timeout path reaps by
/// hand — and a panic BEFORE the wait still reaps through the guard).
pub fn wait_exit(mut guard: DaemonGuard, what: &str) {
    let mut child = guard.child.take().expect("guard already disarmed");
    for _ in 0..50 {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait(); // reap — no zombie on the panic path
    panic!("{what} did not exit");
}
