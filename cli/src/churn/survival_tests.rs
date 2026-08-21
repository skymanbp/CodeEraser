//! survival.rs's clock-basis pin: which blame date decides that a
//! line is still inside the window.

/// One `--line-porcelain` record for a REBASED line: old author
/// date, fresh committer date. `git log --since` puts its commit in
/// the window, so the survivor pass must keep it — the author-time
/// filter it replaced scored this record 0 and inflated churn.
const REBASED: &str = "\
a1b2c3 1 1 1
author A
author-mail <a@example>
author-time 1000
author-tz +0000
committer C
committer-mail <c@example>
committer-time 900000
committer-tz +0000
summary rebased forward
filename a.rs
\tlet x = 1;
";

/// (cutoff, survivors): inside the window `--since` selected, the
/// record survives; past the cutoff the SAME record must not. Driven
/// off one table on the spot — two pasted assert stanzas would have
/// minted a clone block against the only-shrink dedup budget.
#[test]
fn survivors_are_dated_by_committer_time() {
    for (cutoff, want) in [(800_000u64, 1usize), (950_000, 0)] {
        assert_eq!(
            super::surviving_lines(REBASED, cutoff),
            want,
            "cutoff {cutoff}"
        );
    }
}
