//! `ce deadcode` end to end (M5-2h exit row): the walk builds the
//! index, the ladder judges the edges in, the Haskell core judges
//! liveness, and the verdicts come back with names. Pins: the exact
//! dead SET (a phantom dead row and a missed one fail the same
//! assert), the entry conventions (entry_globs / doc entries / doc
//! links keep their targets alive), the empty-index explicit error
//! (a wrong root must never yield a silent empty graph), and G11
//! determinism — a second run and a from-scratch rebuild produce the
//! identical report.

use codeeraser::graph::deadcode;

mod common;

fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

fn dead_set(r: &deadcode::Report) -> Vec<(String, &'static str)> {
    r.dead.iter().map(|(n, v, _)| (n.clone(), *v)).collect()
}

#[test]
fn verdicts_come_back_with_names() {
    // habitat notes, row by row: root.ts lives by entry glob and
    // keeps used.ts alive; the README doc entry links docs/note.md
    // AND the lib/ DIRECTORY — whose synthetic containment arc is
    // what keeps inner.ts alive; orphan.ts and the unlinked doc die
    let fx = common::fixture(
        "deadcode-e2e",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"root.ts\"]\n"),
            ("README.md", "# T\n[note](./docs/note.md)\n[lib](./lib/)\n"),
            ("root.ts", "import './used';\n"),
            ("docs/note.md", "# Note\n"),
            ("used.ts", "export {};\n"),
            ("docs/lost.md", "# Lost\n"),
            ("lib/inner.ts", "export {};\n"),
            ("orphan.ts", "export {};\n"),
        ],
    );
    let (dir, core) = (fx.dir.clone(), core_bin());
    let first = deadcode::run(&dir, None, &core).expect("first run");
    // root.ts (entry glob), used.ts (imported), README.md (doc
    // entry), docs/note.md (doc-linked) live; the orphan module and
    // the unlinked doc die — "no entry rule = every doc trivially
    // dies" is the design stance, pinned here
    let want = vec![
        ("docs/lost.md".to_string(), "unref_private"),
        ("orphan.ts".to_string(), "unref_private"),
    ];
    assert_eq!(dead_set(&first), want, "the exact dead set");
    assert!(first.degraded.is_none(), "healthy run");
    assert!(first.reported.is_empty(), "no aggregate targets die here");
    // G11 determinism: an incremental re-run and a from-scratch
    // rebuild both reproduce the report exactly
    let second = deadcode::run(&dir, None, &core).expect("second run");
    assert_eq!(dead_set(&second), want, "incremental re-run identical");
    std::fs::remove_dir_all(dir.join(".ce")).expect("wipe index");
    let rebuilt = deadcode::run(&dir, None, &core).expect("rebuild run");
    assert_eq!(dead_set(&rebuilt), want, "from-scratch rebuild identical");
}

/// A tree with nothing walkable must be an explicit error — a wrong
/// root yielding a silent empty graph is the failure mode the exit
/// row names.
#[test]
fn empty_index_is_an_explicit_error() {
    let dir = common::tmp("deadcode-empty");
    std::fs::write(dir.join("notes.txt"), "no lang files here\n").expect("write");
    let err = deadcode::run(&dir, None, &core_bin()).expect_err("must refuse");
    assert!(
        err.to_string().contains("empty index"),
        "explicit empty-index error, got: {err:#}"
    );
}

/// The CI gate flag fires BOTH ways (M5-close: a gate that never
/// demonstrated red is unproven): a dead orphan makes `--check` exit
/// 1; dispositioning it through entry_globs turns the same tree
/// green. Arrange rides the shared fixture throat — hand-written
/// write stanzas were the ratchet's catch here.
#[test]
fn check_flag_reds_on_dead_and_greens_on_disposition() {
    let fx = common::fixture(
        "deadcode-checkflag",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"root.ts\"]\n"),
            ("root.ts", "import './used';\n"),
            ("used.ts", "export {};\n"),
            ("orphan.ts", "export {};\n"),
        ],
    );
    let core = core_bin();
    let gate = |why: &str, ok: bool| {
        let out = common::run_ce(&fx.dir, &["deadcode", ".", "--core", &core, "--check"]);
        assert_eq!(out.status.success(), ok, "{why}");
    };
    gate("an undispositioned orphan must red", false);
    std::fs::write(
        fx.dir.join("ce.toml"),
        "[graph]\nentry_globs = [\"root.ts\", \"orphan.ts\"]\n",
    )
    .expect("toml");
    gate("a dispositioned tree must pass", true);
}
