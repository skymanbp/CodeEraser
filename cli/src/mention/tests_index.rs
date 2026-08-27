//! The universe and storage legs (K39 determinism, K40 own gate, K42
//! privacy) on a scratch tree and a scratch index — the same
//! `Index::open` the product uses. Every leg opens with `seeded`: the
//! tree, its files, the index; the scratch scaffold clears a stale
//! tree on the next run, so no leg carries a cleanup tail. The cap and
//! census legs (K40/K41) live in tests_caps.rs on the same helpers.

use super::store;
use super::{Stats, refresh};
use crate::dedup::tokens::fnv1a;
use crate::dedup::{Params, index::Index};
use crate::testutil::scratch;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A scratch tree holding `files`, and an index opened beside it.
pub(super) fn seeded(tag: &str, files: &[(&str, &[u8])]) -> (PathBuf, Index) {
    let dir = scratch(&format!("mention-{tag}"));
    for (rel, bytes) in files {
        put(&dir, rel, bytes);
    }
    let idx = Index::open(&dir.join(".ce/index.db"), Params::default()).expect("open");
    (dir, idx)
}

pub(super) fn put(dir: &Path, rel: &str, bytes: &[u8]) {
    if let Some((sub, _)) = rel.rsplit_once('/') {
        std::fs::create_dir_all(dir.join(sub)).expect(sub);
    }
    std::fs::write(dir.join(rel), bytes).expect(rel);
}

fn universe(idx: &Index) -> BTreeSet<String> {
    store::indexed_paths(idx.raw()).expect("paths")
}

/// One scalar off the index — the legs' only raw read.
pub(super) fn query<T: rusqlite::types::FromSql>(
    idx: &Index,
    sql: &str,
    params: impl rusqlite::Params,
) -> T {
    idx.raw().query_row(sql, params, |r| r.get(0)).expect(sql)
}

pub(super) fn mentioned(idx: &Index, token: &str, other_than: &str) -> bool {
    store::mentioned_by_other(idx.raw(), fnv1a(token.as_bytes()) as i64, other_than).expect("query")
}

/// Whether ONE file's rows hold the token (the per-file half of the
/// other-file rule the store answers).
fn mentions_in(idx: &Index, rel: &str, token: &str) -> bool {
    query(
        idx,
        "SELECT EXISTS(SELECT 1 FROM mentions m JOIN mention_files f ON f.id = m.file_id
         WHERE f.path = ?1 AND m.ident_hash = ?2)",
        (rel, fnv1a(token.as_bytes()) as i64),
    )
}

pub(super) fn run(dir: &Path, idx: &Index) -> Stats {
    refresh(dir, idx).expect("refresh")
}

/// K39: one commit, one U — with or without `.git`, hidden files in,
/// `.gitignore` honoured without a repository, the walker's own
/// `.ignore` NOT honoured, a nested repository cut whole, and a second
/// unchanged run touching nothing.
#[test]
fn the_universe_is_the_same_set_with_and_without_a_git_directory() {
    let (dir, idx) = seeded(
        "k39-git",
        &[
            (".gitignore", b"ignored.txt\n"),
            (".ignore", b"keep.txt\n"),
            ("keep.txt", b"graph_report\n"),
            ("ignored.txt", b"graph_report\n"),
            (".github/wf.yml", b"run: graph_report\n"),
            ("sub/.git", b"gitdir: elsewhere\n"),
            ("sub/keep.txt", b"graph_report\n"),
        ],
    );
    let first = run(&dir, &idx);
    let want: BTreeSet<String> = [".github/wf.yml", ".gitignore", ".ignore", "keep.txt"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        universe(&idx),
        want,
        "hidden in, ignored out, .ignore not a source, nested repo cut"
    );
    assert_eq!((first.universe, first.run.refreshed), (4, 4));
    std::fs::create_dir_all(dir.join(".git")).expect("git");
    let second = run(&dir, &idx);
    assert_eq!(universe(&idx), want, "same set once .git exists");
    assert_eq!(
        (second.run.refreshed, second.run.removed, second.rows),
        (0, 0, first.rows)
    );
}

/// K39: the binary rule and the decoders end to end — an early NUL is
/// skipped and counted, a late NUL stays, UTF-16 and mixed-script
/// prose both mention the name.
#[test]
fn binary_rule_and_decoders_decide_membership() {
    let mut late = b"x y ".repeat(2100);
    late.extend_from_slice(b"graph_report\0");
    let u16: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain("see graph_report".encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    let (dir, idx) = seeded(
        "k39-bytes",
        &[
            ("bin.dat", b"graph\0_report graph_report"),
            ("late.txt", &late),
            ("u16.txt", &u16),
            ("zh.md", "调用graph_report函数".as_bytes()),
        ],
    );
    let s = run(&dir, &idx);
    assert_eq!((s.skipped.binary, s.universe), (1, 3));
    assert!(!universe(&idx).contains("bin.dat"));
    for file in ["late.txt", "u16.txt", "zh.md"] {
        assert!(mentions_in(&idx, file, "graph_report"), "{file}");
    }
}

/// K39: a `MENTION_REV` bump (simulated as a skewed meta row, the way
/// a sibling binary of another revision presents) rescans every file
/// with zero file changes — and the file gate alone would not have.
#[test]
fn a_revision_skew_rescans_everything_without_a_file_change() {
    let (dir, idx) = seeded(
        "k39-rev",
        &[("a.txt", b"alpha_one\n"), ("b.txt", b"beta_two\n")],
    );
    run(&dir, &idx);
    assert_eq!(run(&dir, &idx).run.refreshed, 0);
    idx.raw()
        .execute("UPDATE meta SET v = v + 1 WHERE k = 'mention_rev'", [])
        .expect("skew");
    let s = run(&dir, &idx);
    assert!(s.run.rescanned);
    assert_eq!((s.run.refreshed, s.universe, s.rows), (2, 2, 2));
    assert!(!run(&dir, &idx).run.rescanned, "stamped back");
}

/// K39 (platform-conditional): a file symlink whose target stays in
/// the root is in U once — the lexicographically first path of a
/// shared target wins — a link escaping the root is not, a directory
/// link is neither entered nor an error, and the walk-error counter
/// stays at zero for all of it.
#[test]
fn file_symlinks_enter_once_and_never_escape() {
    let (dir, idx) = seeded(
        "k39-link",
        &[
            ("target.txt", b"linked_name\n"),
            ("sub/inner.txt", b"inner_name\n"),
        ],
    );
    let outside = scratch("mention-k39-outside");
    put(&outside, "secret.txt", b"escaped_name\n");
    if link(&dir.join("target.txt"), &dir.join("link.txt"), false).is_err() {
        return; // no symlink privilege on this platform: the leg is conditional
    }
    link(&outside.join("secret.txt"), &dir.join("escape.txt"), false).expect("second link");
    link(&dir.join("sub"), &dir.join("dirlink"), true).expect("directory link");
    let s = run(&dir, &idx);
    let u = universe(&idx);
    assert!(u.contains("link.txt") && !u.contains("target.txt"), "{u:?}");
    assert!(
        u.contains("sub/inner.txt") && !u.contains("dirlink/inner.txt"),
        "{u:?}"
    );
    assert!(!u.contains("escape.txt"), "{u:?}");
    assert_eq!(
        s.skipped.walk_errors, 0,
        "a duplicate and a directory link are not errors"
    );
    assert!(!mentioned(&idx, "escaped_name", "decl.rs"));
}

/// A symlink to a file or a directory; Windows types the link, unix
/// does not.
fn link(target: &Path, at: &Path, dir: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let _ = dir;
        std::os::unix::fs::symlink(target, at)
    }
    #[cfg(windows)]
    if dir {
        std::os::windows::fs::symlink_dir(target, at)
    } else {
        std::os::windows::fs::symlink_file(target, at)
    }
}

/// K40: the pass owns its gate — a hand-deleted row returns with no
/// byte changed (a shared `files.content_hash` gate would never
/// rebuild it), and a vanished non-judged file loses its rows.
#[test]
fn the_own_hash_gate_rebuilds_a_deleted_row_and_reaps_a_vanished_file() {
    let (dir, idx) = seeded(
        "k40-gate",
        &[("keep.txt", b"alpha_one\n"), ("gone.txt", b"beta_two\n")],
    );
    run(&dir, &idx);
    idx.raw()
        .execute("DELETE FROM mention_files WHERE path = 'keep.txt'", [])
        .expect("hand delete");
    assert!(!mentioned(&idx, "alpha_one", "x"));
    let s = run(&dir, &idx);
    assert_eq!(
        s.run.refreshed, 1,
        "the row comes back under the pass's own gate"
    );
    assert!(mentioned(&idx, "alpha_one", "x"));
    std::fs::remove_file(dir.join("gone.txt")).expect("rm");
    let s = run(&dir, &idx);
    assert_eq!(s.run.removed, 1);
    assert!(!mentioned(&idx, "beta_two", "x"));
}

/// K42: no plaintext token reaches the database — both hash columns
/// are INTEGER and no cast of them carries the probe — while the two
/// EXISTING plaintext faces (`symbols.key`, `sites.spec`) are named
/// here as TEXT, the residue the criterion records rather than hides.
#[test]
fn mentions_hold_hashes_only_and_the_existing_plaintext_faces_are_named() {
    let (dir, idx) = seeded("k42", &[("notes.txt", b"plaintext_probe_token_9f3 here\n")]);
    run(&dir, &idx);
    assert!(mentioned(&idx, "plaintext_probe_token_9f3", "x"));
    let column_type = |table: &str, col: &str| -> String {
        query(
            &idx,
            &format!("SELECT type FROM pragma_table_info('{table}') WHERE name = '{col}'"),
            [],
        )
    };
    for col in ["file_id", "ident_hash", "folded_hash"] {
        assert_eq!(column_type("mentions", col), "INTEGER", "{col}");
    }
    let leaked: i64 = query(
        &idx,
        "SELECT COUNT(*) FROM mentions WHERE CAST(ident_hash AS TEXT) LIKE '%probe%'
         OR CAST(folded_hash AS TEXT) LIKE '%probe%'",
        [],
    );
    assert_eq!(leaked, 0);
    assert_eq!(
        column_type("symbols", "key"),
        "TEXT",
        "existing plaintext face"
    );
    assert_eq!(
        column_type("sites", "spec"),
        "TEXT",
        "existing plaintext face"
    );
}
