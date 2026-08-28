//! `ce deadcode` end to end (M5-2h exit row): the walk builds the
//! index, the ladder judges the edges in, the Haskell core judges
//! liveness, and the verdicts come back with names. Pins: the exact
//! dead SET (a phantom dead row and a missed one fail the same
//! assert), the entry conventions (entry_globs / doc entries / doc
//! links keep their targets alive), the empty-index explicit error
//! (a wrong root must never yield a silent empty graph), and G11
//! determinism — a second run and a from-scratch rebuild produce the
//! identical report.

use crate::common;
use codeeraser::graph::deadcode;

fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

fn dead_set(r: &deadcode::Report) -> Vec<(String, &'static str)> {
    r.dead.iter().map(|d| (d.path.clone(), d.verdict)).collect()
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
    // dies" is the design stance, pinned here.
    //
    // The CODES split since 4.1.0, and this pair is why the export
    // surface earns its place: `# Lost` is a heading, and a heading
    // is an anchor any document may link to, so the file offers a
    // public surface nothing consumes — unref_public, the verdict
    // that was structurally unreachable until the symbols table gave
    // bit 0 a producer. `export {};` names nothing, so orphan.ts has
    // no surface and stays private. Note what did NOT move: the same
    // two files are dead, and the gate's exit is unchanged — an
    // export surface is a verdict axis, never an entry claim (RG10).
    let want = vec![
        ("docs/lost.md".to_string(), "unref_public"),
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

/// The v2.16 K9 hint, both directions: a convention-loaded tree
/// (nothing imports anything, no entry declared) names the
/// entry_globs knob beside its dead list; a tree where death is the
/// exception must NOT — one or two stray dead files are deletion
/// candidates, and hinting an exemption knob at them would teach
/// masking over deleting. Same binary, same console face, only the
/// dead share differs.
#[test]
fn the_entry_globs_hint_follows_the_dead_share() {
    let core = core_bin();
    let plugin = common::fixture(
        "deadcode-plugin-shape",
        &[
            ("cmd-a.ts", "export {};\n"),
            ("cmd-b.ts", "export {};\n"),
            ("cmd-c.ts", "export {};\n"),
        ],
    );
    let out = common::run_ce(&plugin.dir, &["deadcode", ".", "--core", &core]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        text.contains("entry_globs") && text.contains("3 of 3 files"),
        "a fully-dead tree must name the knob: {text}"
    );
    // minority death: one orphan among entry-kept siblings
    let mixed = common::fixture(
        "deadcode-minority-dead",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"root.ts\"]\n"),
            ("root.ts", "import './used';\nimport './also';\n"),
            ("used.ts", "export {};\n"),
            ("also.ts", "export {};\n"),
            ("orphan.ts", "export {};\n"),
        ],
    );
    let out = common::run_ce(&mixed.dir, &["deadcode", ".", "--core", &core]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("orphan.ts"), "the orphan still dies: {text}");
    assert!(
        !text.contains("entry_globs"),
        "a minority dead set must not hint the exemption knob: {text}"
    );
}

/// K43: the 0.3.0 document carries the advisory keys exactly when the
/// road was asked — `ce deadcode`'s own run (Advisory::Yes) has
/// `unmentioned` rows and the drop flag; a report projected from a
/// wire built without the advisory has neither key, so "not asked" and
/// "asked and clean" stay distinct in the document. The same fixture
/// as verdicts_come_back_with_names: `orphan.ts` exports nothing, so
/// its dead verdict stands beside an advisory table that names the
/// unmentioned declarations of the live files.
#[test]
fn the_report_document_carries_the_advisory_only_when_asked() {
    let dir = common::fixtures::tmp("deadcode-report-030");
    common::fixtures::write_doc(
        &dir,
        "--- ce.toml\n[graph]\nentry_globs = [\"root.ts\"]\n\
         --- root.ts\nimport './used';\n\
         --- used.ts\nexport function usedButUnspoken() {}\n\
         --- orphan.ts\nexport {};\n",
    );
    let core = core_bin();
    let report = deadcode::run(&dir, None, &core).expect("run");
    assert_eq!(
        dead_set(&report).len(),
        1,
        "the orphan dies beside the advisory"
    );
    let asked = codeeraser::report::deadcode_json(&report);
    assert_eq!(asked["schema"], "ce.deadcode-report/0.3.0");
    assert_eq!(asked["unmentioned_dropped"], false);
    let rows = asked["unmentioned"].as_array().expect("advisory rows");
    assert_eq!(rows.len(), 1, "{asked}");
    let cell = |k: &str| rows[0][k].as_str().map(str::to_string);
    assert_eq!(
        (cell("name"), cell("symbol"), cell("code")),
        (
            Some("used.ts".into()),
            Some("usedButUnspoken".into()),
            Some("public_unmentioned".into())
        )
    );
    // the same tree judged off a wire built without the advisory
    let (_idx, w) = common::graph_wire(&dir, deadcode::Advisory::No);
    let quiet = codeeraser::report::deadcode_json(
        &deadcode::judge_report(&dir, &core, &w).expect("judged"),
    );
    assert_eq!(quiet["schema"], "ce.deadcode-report/0.3.0");
    assert!(
        quiet.get("unmentioned").is_none() && quiet.get("unmentioned_dropped").is_none(),
        "{quiet}"
    );
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

/// Shared red→green harness for the `--check` gate: build the tree,
/// expect exit 1, apply the one-file rewrite, expect exit 0 (a gate
/// that never demonstrated red is unproven — M5-close stance; a
/// second copy of this stanza was the ratchet's catch, twice).
fn check_flips_on(tag: &str, tree: &[(&str, &str)], rewrite: (&str, &str), why: [&str; 2]) {
    let fx = common::fixture(tag, tree);
    let core = core_bin();
    let gate = |msg: &str, ok: bool| {
        let out = common::run_ce(&fx.dir, &["deadcode", ".", "--core", &core, "--check"]);
        assert_eq!(out.status.success(), ok, "{msg}");
    };
    gate(why[0], false);
    std::fs::write(fx.dir.join(rewrite.0), rewrite.1).expect("rewrite");
    gate(why[1], true);
}

/// One red→green flip case of the --check gate.
type FlipCase = (
    &'static str,
    &'static [(&'static str, &'static str)],
    (&'static str, &'static str),
    [&'static str; 2],
);

/// The gate's red→green flips as ONE table (the ratchet caught the
/// fourth check_flips_on stanza chaining against the third — the
/// stanzas became rows, the P6 discipline): a dead orphan
/// dispositioned through entry_globs; the slice-3 defect fix
/// (2.28.0) — a tool binary the manifest declares via [[bin]] path
/// is a root, where undeclared it is exactly the orphan the gate
/// reds on; and a `#[path]`-mounted file alive with no entry_globs
/// at all (GRAPH_REV 5 — the gate that keeps the ce.toml exemption
/// retirement safe against a ladder regression).
const FLIP_CASES: [FlipCase; 4] = [
    (
        "deadcode-inertdef",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"main.md\"]\n"),
            ("main.md", "# M\n\n[def]: ./note.md\n"),
            ("note.md", "# Note\n"),
        ],
        (
            "main.md",
            "# M\n\nSee [the note][def].\n\n[def]: ./note.md\n",
        ),
        [
            "an unused reference definition must not keep its target alive",
            "using the definition must revive the target",
        ],
    ),
    (
        "deadcode-checkflag",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"root.ts\"]\n"),
            ("root.ts", "import './used';\n"),
            ("used.ts", "export {};\n"),
            ("orphan.ts", "export {};\n"),
        ],
        (
            "ce.toml",
            "[graph]\nentry_globs = [\"root.ts\", \"orphan.ts\"]\n",
        ),
        [
            "an undispositioned orphan must red",
            "a dispositioned tree must pass",
        ],
    ),
    (
        "deadcode-declaredbin",
        &[
            ("Cargo.toml", "[package]\nname='db'"),
            ("src/main.rs", "fn main() {}\n"),
            ("src/tools/gen.rs", "pub fn g() {}\n"),
        ],
        (
            "Cargo.toml",
            "[package]\nname='db'\n[[bin]]\nname='gen'\npath='src/tools/gen.rs'\n",
        ),
        [
            "an undeclared tool binary must red as dead",
            "the declared [[bin]] path alone must green it",
        ],
    ),
    (
        "deadcode-pathmount",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"src/lib.rs\"]\n"),
            ("Cargo.toml", "[package]\nname='pm'"),
            ("src/lib.rs", "mod helper;\n"),
            ("src/helper_impl.rs", "pub fn h() {}\n"),
        ],
        ("src/lib.rs", "#[path = \"helper_impl.rs\"]\nmod helper;\n"),
        [
            "without the attribute the target is an orphan and must red",
            "the #[path] mount alone must keep the target alive",
        ],
    ),
];

#[test]
fn check_gate_flips_red_to_green_across_dispositions() {
    for (tag, tree, rewrite, why) in FLIP_CASES {
        check_flips_on(tag, tree, rewrite, why);
    }
}
