//! Daemon end-to-end: lazy start via the client, ping round-trip,
//! dedup probe over the socket, clean shutdown. Uses the real `ce`
//! binary (CARGO_BIN_EXE) and a throwaway project root so parallel
//! test runs get distinct socket names.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use std::path::Path;
use std::time::{Duration, Instant};

mod common;
use common::{seed_clone_pair, tmp as project_dir};

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
