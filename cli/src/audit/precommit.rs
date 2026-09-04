//! `ce precommit`: the pre-commit face of the audit — STAGED changes
//! only, printed for a person (it runs in a terminal, not a hook), and
//! the commit refused (exit 1) when a deny-tier verdict holds: `[guard]
//! mode` deny over touched duplicates, or `[tombstone] tier` deny over
//! the class's budget. The count is the JUDGED staged set
//! (`changes::diff`'s universe): a staged `.css` can never match a
//! dedup block either.

use super::{gather, reason, tombstone};
use std::path::Path;
use std::process::ExitCode;

/// The face: one gather over `--cached`, the tombstone line once, the
/// duplicate verdict or its degraded summary, the exit code.
pub fn run_precommit(root: &Path) -> ExitCode {
    // session = None honestly: `ce precommit` runs in a terminal, no
    // session owns it — the M4 sampler excludes non-session events.
    // `--cached` needs no base rev: git compares the index against an
    // unborn HEAD without complaint, unlike the Stop leg.
    let Some((mode, net_loc, changed, dups, tomb)) = gather(root, &["--cached"], "precommit", None)
    else {
        eprintln!(
            "{}",
            crate::i18n::line(
                "ce precommit: not a git repo (skipped)",
                "ce precommit：不是 git 仓库（跳过）",
                &[],
            )
        );
        return ExitCode::SUCCESS;
    };
    // the tombstone line rides every exit: the person sees it once,
    // and a deny-tier `over` blocks the commit on its own
    if let Some(line) = tomb.as_ref().and_then(tombstone::summary) {
        println!("{line}");
    }
    let tomb_blocks = tomb.as_ref().is_some_and(tombstone::Leg::blocks);
    if let Some(t) = tomb.as_ref().filter(|t| t.blocks()) {
        println!("{}", tombstone::reason(t));
    }
    let Some(v) = dups.as_ref() else {
        // A9f: fail open but never silently — the human still gets
        // the staged summary the healthy path prints
        println!("{}", staged_summary(changed.len(), net_loc, true));
        return exit_code(tomb_blocks);
    };
    if v.dups == 0 {
        println!("{}", staged_summary(changed.len(), net_loc, false));
        return exit_code(tomb_blocks);
    }
    println!("{}", reason(net_loc, v));
    exit_code(tomb_blocks || (mode == "deny" && v.fail))
}

/// The commit is refused (1) when any deny-tier verdict holds.
fn exit_code(blocked: bool) -> ExitCode {
    if blocked {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// The staged-summary line both healthy exits print. One body, two
/// tails: written apart they were a token twin by this repo's own
/// gate. The `+` sign flag is pre-rendered because the bilingual
/// switch fills plain `{}` holes only — the en bytes are unchanged.
fn staged_summary(changed: usize, net_loc: i64, degraded: bool) -> String {
    let net = format!("{net_loc:+}");
    if degraded {
        return crate::i18n::line(
            "ce precommit: {} staged file(s), net {} LOC — duplicate \
             verdict unavailable (DEGRADED: duplicate check skipped)",
            "ce precommit：{} 个暂存文件，净 {} 行 — 重复判决不可用\
             （已降级：重复检查已跳过）",
            &[&changed, &net],
        );
    }
    crate::i18n::line(
        "ce precommit: {} staged file(s), net {} LOC, no touched duplicates",
        "ce precommit：{} 个暂存文件，净 {} 行，未触及重复块",
        &[&changed, &net],
    )
}
