//! A wedged core is reaped, not waited on. `read_line` on the core's
//! stdout was unbounded, so a core that stopped answering held the
//! daemon — and every hook queued behind its serial accept loop —
//! forever (2026-08-19 review, finding 20). The deadline turns that
//! wedge into the ordinary degraded path one reply later.
//!
//! Alone in its binary: CE_CORE_DEADLINE_SECS is process-global, and
//! integration test files each get their own process.

/// `sort` is the canonical read-everything-answer-at-EOF program on
/// both Windows (System32) and Unix: it swallows the hello line and
/// answers nothing, which is exactly a wedge.
#[test]
fn a_core_that_never_answers_is_reaped_within_the_deadline() {
    // SAFETY: single-threaded at this point — this test is alone in
    // its file, so no other test races the env (the corelink unit
    // test's e2e caveat, honoured by isolation instead of avoidance).
    unsafe { std::env::set_var("CE_CORE_DEADLINE_SECS", "1") };
    let t0 = std::time::Instant::now();
    let err = codeeraser::corelink::Link::open("sort")
        .map(|_| ())
        .expect_err("a silent core must not hand back a live link");
    let waited = t0.elapsed();
    assert!(err.contains("deadline"), "names the deadline: {err}");
    assert!(
        waited < std::time::Duration::from_secs(30),
        "reaped in bounded time, not a hung read: {waited:?}"
    );
}
