//! The daemon client's test half, split from client.rs at the
//! 300-line dogfood wall when the 2.0.0 staleness legs grew it past
//! its ceiling (the candidates_tests.rs precedent): the scripted
//! stale daemons and the trust ordering they exercise.

use super::{Request, bounded, request_if_running, socket_name, stale};
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// A scripted pre-1.1.0 daemon: it answers ONE hello with `proto`
/// (checking no credential, as such a daemon does), then records
/// every further line and answers each with a bye. The handle
/// yields the recorded lines once the client drops the socket.
fn stub_daemon(root: &Path, proto: &str) -> std::thread::JoinHandle<Vec<String>> {
    let ns = socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    let listener = ListenerOptions::new().name(ns).create_sync().expect("bind");
    let hello_ok = format!(r#"{{"type":"hello_ok","proto":"{proto}"}}"#);
    std::thread::spawn(move || {
        let stream = listener.incoming().next().expect("conn").expect("accept");
        let mut conn = BufReader::new(stream);
        let mut lines: Vec<String> = Vec::new();
        let mut first = true;
        loop {
            let mut got = String::new();
            if conn.read_line(&mut got).unwrap_or(0) == 0 {
                return lines;
            }
            let reply = if first {
                hello_ok.as_str()
            } else {
                r#"{"type":"bye"}"#
            };
            writeln!(conn.get_mut(), "{reply}").expect("write");
            conn.get_mut().flush().expect("flush");
            if !first {
                lines.push(got.trim().to_string()); // post-hello only
            }
            first = false;
        }
    })
}

/// The non-lazy path must RETIRE the daemon whose HelloOk it
/// refuses to trust, not just report it: `ce doctor` rides this
/// path, and reporting alone left an unauthenticated pre-1.1.0
/// daemon serving until a lazy client happened by (review
/// 2026-08-20 #1). The staleness still reaches the caller.
/// Two legs by major: a CROSS-major stale hello (1.x since the
/// 2.0.0 field cut) is reported and NOT sent a shutdown — that
/// daemon owes us `restart` and its shutdown word is not ours to
/// assume; a SAME-major older minor is asked to leave. The
/// same-major leg has no specimen while the current proto sits
/// at an x.0.0 floor (no older minor exists to be stale), and
/// re-arms by itself at the first 2.1.0.
#[test]
fn non_lazy_retires_the_stale_daemon_it_refuses_to_trust() {
    let root = crate::testutil::scratch("client-stale");
    let stub = stub_daemon(&root, "1.0.0");
    let err = request_if_running(&root, &Request::Ping).expect_err("stale is refused");
    assert!(
        err.to_string().contains("stale daemon"),
        "the caller still sees the staleness: {err}"
    );
    let sent = stub.join().expect("stub thread");
    assert!(
        sent.is_empty(),
        "a cross-major daemon is not sent our shutdown word: {sent:?}"
    );
    let Some(older) = same_major_older(super::DAEMON_PROTO) else {
        return; // x.0.0: no same-major stale minor can exist yet
    };
    let root = crate::testutil::scratch("client-stale-minor");
    let stub = stub_daemon(&root, &older);
    request_if_running(&root, &Request::Ping).expect_err("stale is refused");
    let sent = stub.join().expect("stub thread");
    assert_eq!(sent.len(), 1, "one line follows the hello: {sent:?}");
    assert!(
        sent[0].contains("shutdown"),
        "the same-major stale daemon is asked to leave: {sent:?}"
    );
}

/// `major.(minor-1).0` when the minor is above 0 — the newest
/// proto a same-major stale daemon could still speak.
fn same_major_older(proto: &str) -> Option<String> {
    let (major, minor) = super::super::proto::major_minor(proto)?;
    (minor > 0).then(|| format!("{major}.{}.0", minor - 1))
}

/// The staleness ordering the HelloOk trust check rides on: an
/// older major (1.x predates the field cut, 1.0 the credential
/// gate) and garbage are stale; the current proto and NEWER
/// same-major minors (additive by the versioning contract) are
/// not.
#[test]
fn stale_orders_minors_and_distrusts_garbage() {
    assert!(stale("1.0.0"), "pre-credential daemon");
    assert!(stale("1.1.0"), "pre-cut daemon, older major");
    assert!(stale("not-a-version"), "unparseable proto");
    assert!(!stale(super::DAEMON_PROTO), "current");
    assert!(!stale("2.99.0"), "newer same-major minor is additive");
}

/// The #85 close, measured: a daemon that accepts and then says
/// NOTHING no longer parks the client forever. The deadline is
/// passed straight to `bounded` — an env pin here would race the
/// parallel tests reading the same process environment.
#[test]
fn a_silent_daemon_meets_the_deadline_not_forever() {
    let root = crate::testutil::scratch("client-silent");
    let ns = socket_name(&root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    let listener = ListenerOptions::new().name(ns).create_sync().expect("bind");
    // accept, then hold the connection open without one byte back
    let hold = std::thread::spawn(move || {
        let stream = listener.incoming().next().expect("conn").expect("accept");
        std::thread::sleep(std::time::Duration::from_secs(5));
        drop(stream);
    });
    let started = std::time::Instant::now();
    let err = bounded(
        &root,
        &Request::Ping,
        false,
        std::time::Duration::from_millis(300),
    )
    .expect_err("silence must not be an answer");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "the deadline bounded the wait: {:?}",
        started.elapsed()
    );
    assert!(
        err.to_string().contains("did not answer within"),
        "the refusal names the deadline: {err}"
    );
    hold.join().expect("stub");
}
