//! Daemon end-to-end: lazy start via the client, ping round-trip,
//! dedup probe over the socket, clean shutdown. Uses the real `ce`
//! binary (CARGO_BIN_EXE) and a throwaway project root so parallel
//! test runs get distinct socket names.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn project_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// T2 pair long enough to clear t=50.
fn seed_clone_pair(dir: &Path) {
    let f = |seed: u32| {
        format!(
            "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
        )
    };
    std::fs::write(dir.join("a.rs"), f(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), f(2)).expect("b.rs");
}

/// The client spawns `ce daemon` via current_exe(), which in a test
/// harness is the TEST binary — point it at the real ce instead by
/// spawning the daemon ourselves before using the client.
fn spawn_daemon(root: &Path) -> std::process::Child {
    std::process::Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("daemon")
        .arg(root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn ce daemon")
}

#[test]
fn ping_dedup_shutdown_roundtrip() {
    let root = project_dir("daemon-e2e");
    seed_clone_pair(&root);
    let mut child = spawn_daemon(&root);

    // ping (retry while the daemon binds)
    let mut pong = None;
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Ok(r) = client::request(&root, &Request::Ping) {
            pong = Some(r);
            break;
        }
    }
    match pong.expect("daemon never came up") {
        Response::Pong { .. } => {}
        other => panic!("expected pong, got {other:?}"),
    }

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
    for _ in 0..50 {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    panic!("daemon did not exit after shutdown");
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
            assert_eq!(report["schema"], "ce.dedup-report/0.4.0");
        }
        other => panic!("expected report, got {other:?}"),
    }
}

/// Version skew: a wrong-major hello gets `restart` and the daemon
/// exits so the client can respawn a fresh binary.
#[test]
fn version_skew_restarts_daemon() {
    let root = project_dir("daemon-skew");
    let mut child = spawn_daemon(&root);
    let mut ready = false;
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request(&root, &Request::Ping).is_ok() {
            ready = true;
            break;
        }
    }
    assert!(ready, "daemon never came up");
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
    for _ in 0..50 {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    panic!("daemon did not exit on version skew");
}
