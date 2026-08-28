//! The veto end to end on a real index (K17 comment, K18 string, K22
//! hidden-directory yaml, K31 fold gate both ways — `scan_row_cap` by
//! its `_` segments and `PyProject` by its camel rise are saved by
//! `scanRowCap` / `pyproject`, `Level` and `RULES` are not — K20 self
//! fence, the no-bit-0-prefilter rule, the `(file, name)` fold across
//! `nth`, and the name-half bits riding the key): the tree is indexed
//! the way `ce deadcode` indexes it, the mention pass runs, and the
//! table is read off `wire_of` under `Advisory::Yes`.

use crate::graph::deadcode::{Advisory, wire_of};
use crate::testutil::{scratch, write_tree};
use std::collections::BTreeMap;

const TREE: &[(&str, &str)] = &[
    ("Cargo.toml", "[package]\nname = \"fx\"\n"),
    (
        "src/lib.rs",
        "pub fn only_in_comment() {}\npub fn only_in_string() {}\npub fn only_in_yaml() {}\n\
         pub fn scan_row_cap() {}\npub struct Level;\npub const RULES: u8 = 0;\n\
         pub fn nobody() {}\nfn hidden() {}\n\
         /// ```\n/// self_fenced();\n/// ```\npub fn self_fenced() {}\n\
         struct A;\nstruct B;\nimpl A {\n    pub fn add(&self) {}\n}\nimpl B {\n    pub fn add(&self) {}\n}\n\
         pub struct PyProject;\n",
    ),
    (
        "src/notes.rs",
        "// only_in_comment is reached by reflection\n// pyproject.toml is read at build time\n",
    ),
    ("app.py", "import m\ngetattr(m, \"only_in_string\")\n"),
    (".github/wf.yml", "run: only_in_yaml\n"),
    ("Core.hs", "scanRowCap = 1\nlevel = 2\nrules = 3\n"),
    (
        "src/allowed.rs",
        "// ce:allow(unmentioned) -- reached by name\npub fn quiet() {}\n",
    ),
    ("tests/t.rs", "pub fn helper() {}\n"),
];

/// `symbol → (path, vis, conv, line)` over the whole table.
fn table(root: &std::path::Path) -> BTreeMap<String, (String, i64, i64, i64)> {
    let (idx, db) = crate::dedup::refreshed_index(root, None).expect("index");
    let w = wire_of(root, &idx, &db, Advisory::Yes).expect("wire");
    let names = w
        .unmentioned
        .as_ref()
        .expect("the advisory road carries the table");
    assert!(w.mounts.as_ref().is_some_and(|m| m.len() == w.nodes.len()));
    let mut out = BTreeMap::new();
    for (&[node, vis, conv], entries) in names {
        assert!(!entries.is_empty(), "a key without names");
        for n in entries {
            let path = w.nodes[node as usize].path.clone();
            assert!(
                out.insert(n.symbol.clone(), (path, vis, conv, n.line))
                    .is_none()
            );
        }
    }
    out
}

#[test]
fn the_veto_reads_every_other_file_and_the_files_own_exceptions() {
    let root = scratch("candidates");
    write_tree(&root, TREE);
    let got = table(&root);
    let want: BTreeMap<String, (String, i64, i64, i64)> = [
        ("nobody", ("src/lib.rs", 3, 0, 7)),
        ("hidden", ("src/lib.rs", 0, 0, 8)),
        ("Level", ("src/lib.rs", 3, 0, 5)),
        ("RULES", ("src/lib.rs", 3, 0, 6)),
        ("A", ("src/lib.rs", 0, 0, 13)),
        ("B", ("src/lib.rs", 0, 0, 14)),
        ("add", ("src/lib.rs", 3, 0, 16)),
        ("quiet", ("src/allowed.rs", 3, 1 << 10, 2)),
        ("helper", ("tests/t.rs", 3, 1 << 1, 1)),
        // the Haskell mentioners are declarations too, and nothing
        // spells them: the veto has no favourites
        ("scanRowCap", ("Core.hs", 3, 0, 1)),
        ("level", ("Core.hs", 3, 0, 2)),
        ("rules", ("Core.hs", 3, 0, 3)),
    ]
    .into_iter()
    .map(|(s, (p, v, c, l))| (s.to_string(), (p.to_string(), v, c, l)))
    .collect();
    assert_eq!(got, want);
    std::fs::remove_dir_all(&root).ok();
}
