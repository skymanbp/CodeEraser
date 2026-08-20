//! Daemon connection authorization (proto 1.1.0). The socket name is
//! a hash of the project root — guessable — so before this gate ANY
//! local user could probe indexed content, run dedup queries, or
//! shut the daemon down. The capability is now READING
//! <root>/.ce/daemon.token (owner-only on Unix; the project dir's
//! ACL on Windows): a wrong token and a tokenless request both get
//! the unauthorized refusal with the CONNECTION closed and the
//! daemon alive — proven here over raw sockets, with the real
//! client's happy path alongside.

use codeeraser::daemon::proto::{DAEMON_PROTO, Request, Response};
use codeeraser::daemon::{auth, client};

mod common;
use common::{raw_daemon_line, tmp as project_dir};

fn is_unauthorized(resp: &Response) -> bool {
    matches!(resp, Response::Error { message } if message.starts_with("unauthorized"))
}

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
