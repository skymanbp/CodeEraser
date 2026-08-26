//! M3 probe core tests: the content gate must find verified T2
//! matches against the index, exclude the file being edited (its
//! indexed version is pre-edit), stay silent below thresholds, and
//! round-trip over the daemon socket.

use codeeraser::dedup::{Params, index::Index, pairs, probe};
use codeeraser::scan::lang::Lang;
use std::path::Path;

use crate::common;
use crate::common::{rust_fn, tmp};

fn filter(p: Params) -> pairs::Filter {
    pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    }
}

/// Index a.rs on disk, then probe content that T2-clones it. The
/// completeness stamp is honest — a.rs IS the whole corpus — and it
/// is what keeps the daemon leg schedule-free: an unstamped index
/// sends the daemon into its ADR-003 first build, and a probe racing
/// that build gets the cold-start refusal (CI caught the window).
fn seeded_index(dir: &Path) -> Index {
    let p = Params::default();
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("write a.rs");
    let mut idx = Index::open(&dir.join(".ce/index.db"), p).expect("open");
    idx.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p)
        .expect("seed");
    idx.mark_full_build().expect("stamp");
    idx
}

/// One probe of `content` as `rel` through the seeded index — the
/// Target/probe stanza used to be pasted per test (three standing
/// dedup-ratchet blocks, repaid with the ADR-008 P3 batch).
fn probed(idx: &Index, dir: &Path, rel: &'static str, content: &[u8]) -> Vec<probe::Match> {
    let p = Params::default();
    let target = probe::Target {
        rel,
        content,
        lang: Lang::Rust,
    };
    probe::probe(idx, dir, target, p, filter(p)).expect("probe")
}

#[test]
fn probe_finds_t2_clone_of_indexed_file() {
    let dir = tmp("probe-hit");
    let idx = seeded_index(&dir);
    // new file whose content T2-clones a.rs (renamed ids, new literals)
    let m = probed(&idx, &dir, "b.rs", &rust_fn(2).into_bytes());
    assert!(!m.is_empty(), "T2 clone of a.rs must be flagged");
    assert_eq!(m[0].file, "a.rs");
    assert!(m[0].tokens >= Params::default().guarantee());
}

#[test]
fn probe_excludes_the_edited_file_itself() {
    let dir = tmp("probe-self");
    let idx = seeded_index(&dir);
    // re-writing a.rs with its own (slightly grown) content must not
    // self-flag against the pre-edit indexed version
    let grown = format!("{}\nfn extra() -> i64 {{ 42 }}\n", rust_fn(1));
    let m = probed(&idx, &dir, "a.rs", grown.as_bytes());
    assert!(m.is_empty(), "self-match must be excluded, got {m:?}");
}

#[test]
fn probe_ignores_short_and_unrelated_content() {
    let dir = tmp("probe-quiet");
    let idx = seeded_index(&dir);
    let m = probed(&idx, &dir, "c.rs", b"fn tiny() -> i64 { 1 }\n");
    assert!(m.is_empty(), "sub-threshold content stays silent");
    let unrelated = "fn other(a: u8) -> u8 { a.wrapping_add(3) }\n".repeat(8);
    let m2 = probed(&idx, &dir, "d.rs", unrelated.as_bytes());
    assert!(m2.is_empty(), "unrelated content stays silent, got {m2:?}");
}

#[test]
fn probe_over_daemon_socket() {
    use codeeraser::daemon::client;
    use codeeraser::daemon::proto::{Request, Response};
    let root = tmp("probe-daemon");
    let _ = seeded_index(&root);
    let child = common::spawn_daemon_ready(&root);
    let req = Request::Probe {
        file_path: root.join("b.rs").display().to_string(),
        content: rust_fn(9),
    };
    match client::request_if_running(&root, &req).expect("probe") {
        Response::ProbeReport {
            matches,
            elapsed_ms,
        } => {
            let arr = matches.as_array().expect("array");
            assert!(!arr.is_empty(), "socket probe must flag the T2 clone");
            assert_eq!(arr[0]["file"], "a.rs");
            println!("socket probe elapsed: {elapsed_ms} ms");
        }
        other => panic!("expected probe report, got {other:?}"),
    }
    // the shared explicit-daemon teardown, Bye asserted — this site
    // was the one weaker hand-rolled copy (batch 9 P13)
    common::shutdown_and_wait(&root, child, "daemon");
}
