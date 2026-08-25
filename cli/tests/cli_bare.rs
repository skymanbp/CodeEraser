//! A question answers with help, not an error (v0.7.3, extended to
//! `ce graph` in K). Before: the usage error went to stderr with exit
//! 2, and PowerShell 5.1 wraps any native stderr in a
//! NativeCommandError wall — the first thing a fresh installer user
//! typed looked like a crash. A wrong subcommand must still be a real
//! error, which is the boundary the last leg guards.

use std::process::Command;

/// One argument shape and the words its answer must contain.
type Question<'a> = (&'a [&'a str], &'a [&'a str]);

/// The real binary with the default (en) surface — that surface is
/// the contract, so CE_LANG never leaks in from the environment.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .env_remove("CE_LANG")
        .args(args)
        .output()
        .expect("run ce")
}

/// Both question shapes in one table: a bare `ce`, and a bare
/// `ce graph` whose subsystem has exactly one usable form. Written
/// as a table because a second copy of this body was an 85-token T2
/// clone by this repo's own gate — which said so on the first draft.
#[test]
fn a_question_answers_on_stdout_and_exits_zero() {
    let cases: &[Question] = &[
        (&[], &["Usage: ce", "doctor"]),
        (&["graph"], &["--sites", "deadcode"]),
    ];
    for (args, must_contain) in cases {
        let out = run(args);
        assert!(out.status.success(), "ce {args:?} exit {:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        for want in *must_contain {
            assert!(
                stdout.contains(want),
                "ce {args:?} missing {want:?}:\n{stdout}"
            );
        }
        assert!(
            out.stderr.is_empty(),
            "ce {args:?} must leave stderr empty: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn unknown_subcommand_still_fails_loudly() {
    let out = run(&["no-such-subcommand"]);
    assert!(!out.status.success(), "a typo must not exit 0");
    assert!(!out.stderr.is_empty(), "the error goes to stderr");
}
