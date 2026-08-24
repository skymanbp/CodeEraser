//! Request dispatch: the reply for one parsed request, and the
//! whole-daemon exits that ride out on a connection thread. Split
//! from conn.rs (which keeps the front door) when the reply had to be
//! built strictly BEFORE the socket write — see `build`.

use super::state::Shared;
use crate::daemon::judge::Judge;
use crate::daemon::proto::{Request, Response};

/// The reply for one request, plus the bool = keep the daemon alive.
///
/// The judge lock spans the WHOLE match — dedup/probe write the
/// index, and two in-process writers would be a new concurrency class
/// the v1.7 convergence contract never had to cover — but it is
/// released HERE, before the caller writes anything. Held across that
/// blocking write, one client that stopped draining its end wedged
/// every other connection's request, `ce doctor`'s Ping included
/// (review 2026-08-20). `build` is handed no stream, so it cannot
/// regress into spanning the write again; `Response` owns its
/// payload, so nothing borrows the guard out of here.
pub(super) fn build(shared: &Shared, req: Request) -> (Response, bool) {
    let mut judge = lock_judge(shared);
    let root = &shared.root;
    match req {
        // every hello (first or repeated) is consumed by conn::handle;
        // this arm is unreachable and exists for match exhaustiveness
        Request::Hello { .. } => (
            Response::Error {
                message: "hello is negotiated per connection".into(),
            },
            true,
        ),
        Request::Ping => (
            Response::Pong {
                uptime_ms: shared.start.elapsed().as_millis() as u64,
            },
            true,
        ),
        Request::Dedup {
            min_tokens,
            min_distinct,
        } => (
            super::replies::dedup_reply(root, min_tokens, min_distinct),
            true,
        ),
        Request::Probe { file_path, content } => (
            super::replies::probe_reply(root, &file_path, &content),
            true,
        ),
        Request::FourClass { pairs } => (
            Response::FourClassReport {
                report: judge.classify(root, &pairs),
            },
            true,
        ),
        Request::Shutdown => (Response::Bye, false),
    }
}

/// Poison recovery: Judge degrades through its failure budget and
/// keeps no state a panic could half-write, so a panicked holder must
/// not brick every later request.
fn lock_judge(shared: &Shared) -> std::sync::MutexGuard<'_, Judge> {
    shared.judge.lock().unwrap_or_else(|e| e.into_inner())
}

/// The whole-daemon exits (shutdown / skew) happen on a connection
/// thread and cannot flow back to the blocked accept loop, so the
/// thread finishes the teardown itself: the judge lock is taken (no
/// request mid-flight), the core link's Drop runs (a bare exit would
/// orphan the ce-core child), then the process ends — the reply was
/// already flushed by conn.rs.
pub(super) fn exit_daemon(shared: &Shared) -> ! {
    lock_judge(shared).retire_link();
    std::process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::{Shared, build};
    use crate::daemon::judge::Judge;
    use crate::daemon::proto::{Request, Response};
    use std::sync::Mutex;
    use std::time::Instant;

    /// The judge must be free the moment the response exists: the
    /// caller still has a blocking socket write ahead of it, and
    /// while the guard lived across that write one client which
    /// stopped reading its end blocked every other connection.
    #[test]
    fn build_releases_the_judge_before_the_caller_writes() {
        let shared = Shared {
            root: std::env::temp_dir(),
            start: Instant::now(),
            token: String::new(),
            judge: Mutex::new(Judge::default()),
        };
        let (resp, keep) = build(&shared, Request::Ping);
        assert!(matches!(resp, Response::Pong { .. }), "got {resp:?}");
        assert!(keep, "a ping never retires the daemon");
        assert!(
            shared.judge.try_lock().is_ok(),
            "another connection must be able to take the judge now"
        );
    }
}
