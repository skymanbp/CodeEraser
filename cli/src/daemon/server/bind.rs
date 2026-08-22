//! The socket bind and its stale-corpse fork (split from server.rs so
//! the reclaim contract has one home).

use anyhow::{Context, Result, bail};
use interprocess::local_socket::traits::Stream as _;
use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, Stream, ToNsName};
use std::io::ErrorKind;

/// "The name is held by SOMETHING": Unix answers AddrInUse; a Windows
/// named pipe with a live FIRST_PIPE_INSTANCE holder answers
/// ERROR_ACCESS_DENIED (PermissionDenied) instead — same fork.
fn collision(e: &std::io::Error) -> bool {
    matches!(e.kind(), ErrorKind::AddrInUse | ErrorKind::PermissionDenied)
}

/// Bind the root's socket. A collision forks on one question: is a
/// listener ALIVE behind the name? A successful connect proves one —
/// that is the singleton race, and this loser leaves. But on macOS
/// GenericNamespaced is a FILE-backed pseudo-namespace (no abstract
/// sockets there), and every daemon exit path is process::exit — the
/// listener's reclaim Drop never runs, so the socket file outlives
/// its daemon, refuses connects, and blocks every rebind: ONE
/// shutdown bricked all later daemons for that root, degrading every
/// probe fail-open and silently (the v1.0.0 tag leg's observe golden
/// caught it as three degraded probes on macOS only). A REFUSED
/// connect proves no listener, so the retry overwrites: try_overwrite
/// unlinks the corpse between bind attempts. Linux (abstract
/// namespace) and Windows (named pipe) names die with their process,
/// so the corpse arm never fires there. A racer binding between the
/// check and the overwrite loses its file to the unlink — two
/// daemons then serve convergently (the v1.7 idempotent-write
/// contract) and the extra one idles out.
pub(super) fn bind(name: &str) -> Result<Listener> {
    let ns = || {
        name.to_owned()
            .to_ns_name::<GenericNamespaced>()
            .context("socket name")
    };
    match ListenerOptions::new().name(ns()?).create_sync() {
        Ok(l) => Ok(l),
        Err(e) if collision(&e) => {
            if Stream::connect(ns()?).is_ok() {
                bail!("bind {name}: another daemon already serving this root");
            }
            ListenerOptions::new()
                .name(ns()?)
                .try_overwrite(true)
                .create_sync()
                .with_context(|| format!("bind {name}: reclaim of a stale socket file"))
        }
        Err(e) => Err(e).with_context(|| format!("bind {name}")),
    }
}
