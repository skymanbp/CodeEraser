//! Daemon end-to-end: lazy start via the client, ping round-trip,
//! dedup probe over the socket, clean shutdown. Uses the real `ce`
//! binary (CARGO_BIN_EXE) and a throwaway project root so parallel
//! test runs get distinct socket names.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use std::path::Path;
use std::time::Instant;

mod common;
use common::{seed_clone_pair, tmp as project_dir};

#[test]
fn ping_dedup_shutdown_roundtrip() {
    let root = project_dir("daemon-e2e");
    seed_clone_pair(&root);
    let child = common::spawn_daemon_ready(&root);

    // warm probe latency (informational; the formal p95 budget is the
    // M2 acceptance run)
    let t0 = Instant::now();
    let r = client::request(&root, &Request::Ping).expect("warm ping");
    let warm_ms = t0.elapsed().as_millis();
    assert!(matches!(r, Response::Pong { .. }));
    println!("warm ping round-trip: {warm_ms} ms");

    assert_dedup_probe(&root);

    // clean shutdown; the process must exit
    match client::request(&root, &Request::Shutdown).expect("shutdown") {
        Response::Bye => {}
        other => panic!("expected bye, got {other:?}"),
    }
    common::wait_exit(child, "daemon after shutdown");
}

/// The socket-side dedup probe must find the seeded T2 clone.
fn assert_dedup_probe(root: &Path) {
    let req = Request::Dedup {
        min_tokens: None,
        min_distinct: None,
    };
    match client::request(root, &req).expect("dedup") {
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

/// Version skew: a wrong-major hello gets `restart` and the daemon
/// exits so the client can respawn a fresh binary.
#[test]
fn version_skew_restarts_daemon() {
    let root = project_dir("daemon-skew");
    let child = common::spawn_daemon_ready(&root);
    // raw connection with a bad-major hello (client::request would
    // auto-respawn; here we watch the exit itself)
    use interprocess::local_socket::traits::Stream as _;
    use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
    use std::io::{BufRead, BufReader, Write};
    let ns = codeeraser::daemon::proto::socket_name(&root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    let mut conn = BufReader::new(Stream::connect(ns).expect("connect"));
    writeln!(
        conn.get_mut(),
        "{}",
        serde_json::to_string(&Request::Hello {
            proto: "999.0.0".into()
        })
        .expect("ser")
    )
    .expect("write");
    conn.get_mut().flush().expect("flush");
    let mut reply = String::new();
    conn.read_line(&mut reply).expect("read");
    let resp: Response = serde_json::from_str(reply.trim()).expect("parse");
    assert!(matches!(resp, Response::Restart { .. }), "got {resp:?}");
    common::wait_exit(child, "daemon on version skew");
}
