//! The silent-bypass battery: every shape where the Stop guard used
//! to pass QUIETLY while duplication sat in the tree. Each case was
//! confirmed against the shipped v0.4.0 binary in the 2026-08-19
//! review; the thesis they share is that a guard may fail open but
//! must never fail SILENT, and that git's answer is not automatically
//! spoken in ce's own path vocabulary.

use std::path::Path;

mod common;
use common::{assert_stop_blocks as blocks, committed_then, staged_repo, stop_observe};

/// A `ce.toml` anchored in a monorepo package: `git -C pkg/app diff`
/// prints REPO-root-relative paths (`pkg/app/b.rs`) while
/// `dedup::analyze` names them from the CE root (`b.rs`), so
/// `changed.contains(&b.a_file)` compared two spellings of one file
/// and never matched — a 100% bypass for every nested package, caused
/// by the very feature hookio advertises (ce.toml is a first-class
/// anchor). `--relative` rebases the diff leg onto the CE root.
#[test]
fn nested_ce_toml_anchor_still_blocks() {
    blocks(&staged_repo("bypass-nested", "pkg/app", true), "b.rs");
}

/// A brand-new project has no HEAD until its first commit, and `git
/// diff HEAD` then fails outright — the whole audit took the
/// fail-open path with NO feed entry, so the guard was entirely off
/// and left no trace, in exactly the scenario this product leads
/// with. Staged rather than untracked, so the DIFF leg is what has to
/// survive (git's empty tree becomes the base).
#[test]
fn unborn_head_still_blocks() {
    blocks(&staged_repo("bypass-unborn", "", false), "b.rs");
}

/// The ASCII twin of this case lives in audit_stop.rs; here the NAME
/// is the whole point. core.quotePath (git's DEFAULT) C-quotes
/// non-ASCII paths, so the changed set held
/// `"\346\265\213\350\257\225.rs"`, matched no dedup block, and left
/// every repo with a CJK / accented / Cyrillic source filename
/// unguarded — while the identical file named `c.rs` blocked.
#[test]
fn non_ascii_filename_still_blocks() {
    committed_then(
        "bypass-non-ascii",
        |d: &Path| std::fs::write(d.join("测试.rs"), common::rust_fn(3)).expect("cjk clone"),
        "测试.rs",
    );
}

/// A rename is ONE numstat column (`old => new`); that phantom path
/// entered the changed set and matched nothing, so re-homing a clone
/// with `git mv` slipped through while the identical end state via
/// delete-then-create was caught.
#[test]
fn renamed_clone_still_blocks() {
    committed_then(
        "bypass-rename",
        |d: &Path| common::git(d, &["mv", "b.rs", "moved.rs"]),
        "moved.rs",
    );
}

/// `hookio::project_root` accepted ANY `.git` — including a plain
/// FILE of that name, which one Write creates and the PreToolUse
/// probe cannot object to (`.git` has no language). The subtree then
/// anchored to itself: a fresh root with no ce.toml, so the guard
/// fell to its unset default and went quiet, writing its feed
/// somewhere else entirely. Broken submodule checkouts leave `.git`
/// FILES routinely, so this needs no attacker.
#[test]
fn a_plain_dot_git_file_cannot_re_root_the_guard() {
    let dir = common::tmp("bypass-anchor");
    common::seed_git_clone_repo(&dir, "deny");
    blocks(&common::fake_anchor_subdir(&dir, "sub"), "c.rs");
}

/// `mode = "Deny"` used to come back from `tier()` verbatim: the
/// audit compares `== "deny"` and stayed silent, while SessionStart
/// printed `guard: Deny` as though it were armed.
#[test]
fn unknown_guard_mode_degrades_and_names_itself() {
    let dir = common::tmp("bypass-bad-mode");
    common::seed_git_clone_repo(&dir, "Deny");
    let mode = stop_observe(&dir)["mode"]
        .as_str()
        .expect("mode")
        .to_string();
    assert!(mode.starts_with("observe ("), "degrades to observe: {mode}");
    assert!(mode.contains("Deny"), "names the offending value: {mode}");
}
