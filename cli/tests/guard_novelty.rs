//! The novel-duplication probe semantics (K step 11 root fix): the
//! guard denies duplication a write INTRODUCES, and stays silent on
//! duplication the replaced content already carried — Write replaces
//! the on-disk file, Edit replaces old_string. The FPR replay
//! measured the whole-content rule denying full-file rewrites of
//! files carrying budgeted blocks; the split-to-a-new-file write
//! still denies (no replaced content exists), and its reason now
//! teaches the safe ordering.

use std::path::Path;

mod common;
use common::fixtures::rust_fn;
use common::gates::write_all;
use common::hooks::{expect_decision, pretooluse_edit_envelope, pretooluse_envelope_at};
use common::{build_index, silent_hook_observe};

/// a.rs and b.rs are an indexed T2 pair with the guard at deny —
/// seed_sources owns the a.rs + ce.toml stanza, b.rs joins it.
fn seeded(tag: &str) -> std::path::PathBuf {
    let dir = common::tmp(tag);
    common::seed_sources(&dir, "deny");
    write_all(&dir, &[("b.rs", &rust_fn(2))]);
    build_index(&dir);
    dir
}

fn silent(dir: &Path, envelope: &str) -> serde_json::Value {
    silent_hook_observe(dir, &["probe", "--hook"], envelope, "probe")
}

/// A full rewrite of b.rs that KEEPS its twin: the baseline (the
/// on-disk file being replaced) already carries the a.rs match, so
/// nothing is new — silence, and the feed shows zero novel matches.
#[test]
fn a_rewrite_carrying_its_own_twin_is_not_new() {
    let dir = seeded("novelty-rewrite");
    let kept = format!("{}// trailing note\n", rust_fn(2));
    let line = silent(&dir, &pretooluse_envelope_at(&dir, "b.rs", "Write", &kept));
    assert_eq!(line["degraded"], false);
    assert_eq!(line["matches"], 0, "carried duplication is not novel");
}

/// The same rewrite ADDING a fresh copy of the twin family into a
/// third file is new duplication and still denies — the subtraction
/// must never launder an introduction.
#[test]
fn a_rewrite_adding_a_twin_still_denies() {
    let dir = seeded("novelty-adding");
    let lone = "fn lone(n: u8) -> u8 { n + 1 }\n";
    write_all(&dir, &[("c.rs", lone)]);
    build_index(&dir);
    let added = format!("{lone}{}", rust_fn(3));
    let deny = pretooluse_envelope_at(&dir, "c.rs", "Write", &added);
    let reason = expect_decision(&dir, &deny, "deny");
    assert!(
        reason.contains("a.rs") || reason.contains("b.rs"),
        "{reason}"
    );
}

/// An Edit whose old_string already carried the twin (editing inside
/// a budgeted block) is carried duplication, not an introduction.
#[test]
fn an_edit_replacing_the_twin_with_itself_is_not_new() {
    let dir = seeded("novelty-edit");
    let old = rust_fn(2);
    let new = format!("{old}// touched\n");
    let line = silent(&dir, &pretooluse_edit_envelope(&dir, "b.rs", &old, &new));
    assert_eq!(line["matches"], 0, "the replaced span already matched");
}

/// The split-to-a-new-file write has no replaced content and still
/// denies — and the reason teaches the ordering that passes (trim
/// the source first; the probe verifies against the current tree).
#[test]
fn a_new_file_split_still_denies_and_teaches_the_order() {
    let dir = seeded("novelty-split");
    let split = pretooluse_envelope_at(&dir, "leaf.rs", "Write", &rust_fn(2));
    let reason = expect_decision(&dir, &split, "deny");
    assert!(
        reason.contains("Trim the source region first"),
        "the move guidance rides the deny: {reason}"
    );
}
