//! Daemon end-to-end: lazy start via the client, ping round-trip,
//! dedup probe over the socket, clean shutdown, version skew — and
//! the 1.1.0 connection authorization (folded in from
//! daemon_auth.rs at the v0.5.0 test consolidation). Uses the real
//! `ce` binary (CARGO_BIN_EXE) and a throwaway project root so
//! parallel test runs get distinct socket names.

use codeeraser::daemon::proto::{DAEMON_PROTO, Request, Response};
use codeeraser::daemon::{auth, client};
use std::path::Path;
use std::time::{Duration, Instant};

mod common;
use common::{raw_daemon_line, seed_clone_pair, tmp as project_dir};

/// ADR-003 cold start: a repo whose index was never built must never
/// get a silent empty ProbeReport — that reply is indistinguishable
/// from a genuine clean probe (the §5.9 silent-failure class). The
/// daemon answers Error (client maps it to degraded, fail-open) until
/// its background first build lands, then serves real matches with no
/// dedup run or Stop audit ever having touched the repo.
#[test]
fn cold_start_probe_degrades_then_serves_matches() {
    let root = project_dir("daemon-cold");
    seed_clone_pair(&root);
    let child = common::spawn_daemon_ready(&root);
    let req = Request::Probe {
        file_path: root.join("new.rs").display().to_string().replace('\\', "/"),
        content: common::rust_fn(9),
    };
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match client::request_if_running(&root, &req).expect("probe") {
            Response::ProbeReport { matches, .. } => {
                let n = matches.as_array().expect("matches array").len();
                assert!(n > 0, "silent empty report from a never-built index");
                break; // build landed; the seeded clones are visible
            }
            Response::Error { .. } => {} // degraded: honest cold start
            other => panic!("unexpected reply: {other:?}"),
        }
        assert!(Instant::now() < deadline, "first index build never landed");
        std::thread::sleep(Duration::from_millis(100)); // poll the async build
    }
    common::shutdown_and_wait(&root, child, "daemon after cold-start test");
}

#[test]
fn ping_dedup_shutdown_roundtrip() {
    let root = project_dir("daemon-e2e");
    seed_clone_pair(&root);
    let child = common::spawn_daemon_ready(&root);

    // warm probe latency (informational; the formal p95 budget is the
    // M2 acceptance run)
    let t0 = Instant::now();
    let r = client::request_if_running(&root, &Request::Ping).expect("warm ping");
    let warm_ms = t0.elapsed().as_millis();
    assert!(matches!(r, Response::Pong { .. }));
    println!("warm ping round-trip: {warm_ms} ms");

    assert_dedup_probe(&root);
    assert_probe_excludes_self(&root);

    common::shutdown_and_wait(&root, child, "daemon after shutdown");
}

/// Probing content FOR an indexed file must not report that file as
/// its own duplicate source. Cross-platform regression: the daemon
/// canonicalizes its root (\\?\ form on Windows), so a plain
/// absolute file_path never strip-matched it and self-exclusion was
/// silently dead on Windows — first exposed by the observe-feed
/// golden diverging between CI platforms.
fn assert_probe_excludes_self(root: &Path) {
    // the daemon indexed a.rs + b.rs via assert_dedup_probe's run
    let req = Request::Probe {
        file_path: root.join("a.rs").display().to_string().replace('\\', "/"),
        content: common::rust_fn(9),
    };
    match client::request_if_running(root, &req).expect("probe") {
        Response::ProbeReport { matches, .. } => {
            let files: Vec<&str> = matches
                .as_array()
                .expect("matches array")
                .iter()
                .map(|m| m["file"].as_str().expect("file"))
                .collect();
            assert_eq!(files, ["b.rs"], "a.rs must be excluded as self");
        }
        other => panic!("expected probe report, got {other:?}"),
    }
}

/// The socket-side dedup probe must find the seeded T2 clone.
fn assert_dedup_probe(root: &Path) {
    let req = Request::Dedup {
        min_tokens: None,
        min_distinct: None,
    };
    match client::request_if_running(root, &req).expect("dedup") {
        Response::DedupReport { report } => {
            let blocks = report["blocks"].as_array().expect("blocks array");
            assert!(!blocks.is_empty(), "seeded clone must be found");
            // transport check only — the schema SHAPE is pinned by the
            // report_schema golden, so no literal id duplicated here
            assert_eq!(report["schema"], codeeraser::dedup::SCHEMA_ID);
        }
        other => panic!("expected report, got {other:?}"),
    }
}

/// M4 judgment forwarding: the daemon owns the ce-core link and a
/// FourClass request over a real cross-file function move comes back
/// with the relocation named on both ends. Requires CE_CORE_BIN like
/// core_wire — this gate must not silently skip.
#[test]
fn fourclass_reports_cross_file_relocation() {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    );
    use codeeraser::daemon::client;
    let root = project_dir("daemon-fourclass");
    let write_libs = |a: String, b: String| {
        std::fs::write(root.join("lib_a.rs"), a).expect("lib_a");
        std::fs::write(root.join("lib_b.rs"), b).expect("lib_b");
    };
    write_libs(
        format!("{}\n{}", common::rust_fn(1), common::rust_fn(2)),
        common::rust_fn(3),
    );
    common::git(&root, &["init", "-q"]);
    common::git(&root, &["add", "."]);
    common::git(&root, &["commit", "-qm", "seed"]);
    // the working tree moves work_2 from lib_a.rs into lib_b.rs
    write_libs(
        common::rust_fn(1),
        format!("{}\n{}", common::rust_fn(3), common::rust_fn(2)),
    );
    let child = common::spawn_daemon_ready(&root);
    let pairs = codeeraser::fourclass::session::head_pairs(&root).expect("pairs");
    let req = Request::FourClass { pairs };
    match client::request_if_running(&root, &req).expect("fourclass") {
        Response::FourClassReport { report } => {
            assert_eq!(report["degraded"], serde_json::Value::Null, "{report}");
            assert!(report["added_moved"].as_u64().expect("in") >= 2, "{report}");
            assert!(
                report["removed_moved"].as_u64().expect("out") >= 2,
                "{report}"
            );
            let rel = &report["relocations"][0];
            assert_eq!(rel["from"], "lib_a.rs", "{report}");
            assert_eq!(rel["to"], "lib_b.rs", "{report}");
            assert_eq!(rel["from_unit"], "work_2/2", "{report}");
        }
        other => panic!("expected report, got {other:?}"),
    }
    common::shutdown_and_wait(&root, child, "daemon after fourclass test");
}

/// Version skew: a wrong-major hello gets `restart` and the daemon
/// exits so the client can respawn a fresh binary. The hello carries
/// the REAL token — since 1.1.0 the token gates the restart path
/// too, or any local user could exit the daemon with a faked skew
/// (that refusal is daemon_auth.rs's battery).
#[test]
fn version_skew_restarts_daemon() {
    let root = project_dir("daemon-skew");
    let child = common::spawn_daemon_ready(&root);
    // raw line with a bad-major hello (client::request would
    // auto-respawn; here we watch the exit itself)
    let resp = raw_daemon_line(
        &root,
        &Request::Hello {
            proto: "999.0.0".into(),
            token: auth::read(&root),
        },
    );
    assert!(matches!(resp, Response::Restart { .. }), "got {resp:?}");
    common::wait_exit(child, "daemon on version skew");
}

/// A connection that never sends a byte must park ITS OWN thread,
/// not the daemon: before the per-connection threads (review
/// 2026-08-20 #3) this ping blocked forever behind the parked
/// connection in the serial accept loop.
#[test]
fn a_silent_connection_does_not_stall_the_next_client() {
    let root = project_dir("daemon-stall");
    let child = common::spawn_daemon_ready(&root);
    let parked = common::raw_daemon_connect(&root);
    let r = client::request_if_running(&root, &Request::Ping).expect("ping past parked conn");
    assert!(matches!(r, Response::Pong { .. }), "got {r:?}");
    drop(parked);
    common::shutdown_and_wait(&root, child, "daemon after stall test");
}

/// An unauthed line longer than the pre-hello cap is refused AT the
/// cap — the bytes beyond it are never buffered or parsed, so a
/// tokenless prober cannot feed the daemon an unbounded line.
#[test]
fn an_unauthed_oversize_line_is_refused_at_the_cap() {
    use std::io::{BufRead, Write};
    let root = project_dir("daemon-oversize");
    let child = common::spawn_daemon_ready(&root);
    let mut conn = common::raw_daemon_connect(&root);
    let noise = vec![b'x'; 8192]; // twice the cap, no newline
    conn.get_mut().write_all(&noise).expect("write");
    conn.get_mut().flush().expect("flush");
    let mut reply = String::new();
    conn.read_line(&mut reply).expect("read");
    let resp: Response = serde_json::from_str(reply.trim()).expect("parse");
    assert!(
        matches!(&resp, Response::Error { message } if message.contains("pre-hello cap")),
        "got {resp:?}"
    );
    // then the connection closes: EOF, or a broken pipe on Windows
    // (the daemon dropped its end with our excess bytes unread)
    match conn.read_line(&mut reply) {
        Ok(0) | Err(_) => {}
        Ok(n) => panic!("connection stayed open: {n} more bytes"),
    }
    common::shutdown_and_wait(&root, child, "daemon after oversize test");
}

fn is_unauthorized(resp: &Response) -> bool {
    matches!(resp, Response::Error { message } if message.starts_with("unauthorized"))
}

/// The 1.1.0 gate: the socket name is a hash of the project root —
/// guessable — so the capability is READING .ce/daemon.token
/// (owner-only on Unix; the project dir's ACL on Windows). A wrong
/// token and a tokenless request both get the refusal with the
/// CONNECTION closed and the daemon alive.
#[test]
fn the_token_gates_the_connection_and_the_daemon_survives_refusals() {
    let root = project_dir("daemon-auth");
    let child = common::spawn_daemon_ready(&root);

    // a request BEFORE any hello: refused, connection closed
    let r = raw_daemon_line(&root, &Request::Ping);
    assert!(is_unauthorized(&r), "tokenless ping must be refused: {r:?}");

    // a hello with the WRONG token: refused
    let r = raw_daemon_line(
        &root,
        &Request::Hello {
            proto: DAEMON_PROTO.into(),
            token: "not-the-token".into(),
        },
    );
    assert!(is_unauthorized(&r), "wrong token must be refused: {r:?}");

    // the minted token is on disk, fresh for this serve
    assert_eq!(auth::read(&root).len(), 64, "32 random bytes, hex");

    // the real client reads the same file and gets through — the
    // refusals above closed CONNECTIONS, never the daemon
    let r = client::request_if_running(&root, &Request::Ping).expect("authed ping");
    assert!(matches!(r, Response::Pong { .. }), "got {r:?}");

    common::shutdown_and_wait(&root, child, "daemon after auth battery");
}
