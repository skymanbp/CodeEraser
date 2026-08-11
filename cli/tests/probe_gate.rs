//! M3 probe core tests: the content gate must find verified T2
//! matches against the index, exclude the file being edited (its
//! indexed version is pre-edit), stay silent below thresholds, and
//! round-trip over the daemon socket.

use codeeraser::dedup::{Params, index::Index, pairs, probe};
use codeeraser::scan::lang::Lang;
use std::path::Path;

mod common;
use common::{rust_fn, tmp};

fn filter(p: Params) -> pairs::Filter {
    pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    }
}

/// Index a.rs on disk, then probe content that T2-clones it.
fn seeded_index(dir: &Path) -> Index {
    let p = Params::default();
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("write a.rs");
    let mut idx = Index::open(&dir.join(".ce/index.db"), p).expect("open");
    idx.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p)
        .expect("seed");
    idx
}

#[test]
fn probe_finds_t2_clone_of_indexed_file() {
    let dir = tmp("probe-hit");
    let idx = seeded_index(&dir);
    let p = Params::default();
    // new file whose content T2-clones a.rs (renamed ids, new literals)
    let target = probe::Target {
        rel: "b.rs",
        content: &rust_fn(2).into_bytes(),
        lang: Lang::Rust,
    };
    let m = probe::probe(&idx, &dir, target, p, filter(p)).expect("probe");
    assert!(!m.is_empty(), "T2 clone of a.rs must be flagged");
    assert_eq!(m[0].file, "a.rs");
    assert!(m[0].tokens >= p.guarantee());
}

#[test]
fn probe_excludes_the_edited_file_itself() {
    let dir = tmp("probe-self");
    let idx = seeded_index(&dir);
    let p = Params::default();
    // re-writing a.rs with its own (slightly grown) content must not
    // self-flag against the pre-edit indexed version
    let grown = format!("{}\nfn extra() -> i64 {{ 42 }}\n", rust_fn(1));
    let target = probe::Target {
        rel: "a.rs",
        content: grown.as_bytes(),
        lang: Lang::Rust,
    };
    let m = probe::probe(&idx, &dir, target, p, filter(p)).expect("probe");
    assert!(m.is_empty(), "self-match must be excluded, got {m:?}");
}

#[test]
fn probe_ignores_short_and_unrelated_content() {
    let dir = tmp("probe-quiet");
    let idx = seeded_index(&dir);
    let p = Params::default();
    let short = "fn tiny() -> i64 { 1 }\n";
    let target = probe::Target {
        rel: "c.rs",
        content: short.as_bytes(),
        lang: Lang::Rust,
    };
    let m = probe::probe(&idx, &dir, target, p, filter(p)).expect("probe short");
    assert!(m.is_empty(), "sub-threshold content stays silent");
    let unrelated = "fn other(a: u8) -> u8 { a.wrapping_add(3) }\n".repeat(8);
    let target2 = probe::Target {
        rel: "d.rs",
        content: unrelated.as_bytes(),
        lang: Lang::Rust,
    };
    let m2 = probe::probe(&idx, &dir, target2, p, filter(p)).expect("probe unrelated");
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
    common::shutdown_daemon(&root);
    common::wait_exit(child, "daemon");
}
