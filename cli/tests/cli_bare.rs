//! A question answers with help, not an error (v0.7.3, extended to
//! `ce graph` in K). Before: the usage error went to stderr with exit
//! 2, and PowerShell 5.1 wraps any native stderr in a
//! NativeCommandError wall — the first thing a fresh installer user
//! typed looked like a crash. A wrong subcommand must still be a real
//! error, which is the boundary the last leg guards.
//!
//! The LANGUAGE axis (K step 8) is the second half. This leg used to
//! scrub CE_LANG so it could only ever see the English road — and the
//! contract held there while `ce --lang zh` exited 2, because clap
//! raises DisplayHelpOnMissingArgumentOrSubcommand only for an EMPTY
//! argv. A gate blind to a selector cannot see a promise broken along
//! it, so every question shape now runs down all three roads the
//! i18n charter names: default, the `--lang` flag, and CE_LANG.

use std::process::Command;

/// One argument shape and the words its answer must contain.
type Question<'a> = (&'a [&'a str], &'a [&'a str]);

/// How the language gets selected — the three roads i18n.rs declares
/// (flag wins over env; env alone works; neither is the en default).
/// The exit-code and empty-stderr halves of the contract must hold on
/// every one of them; the WORDS are asserted on the default road
/// alone, because that surface is the byte contract.
const ROADS: &[(&str, &[&str], Option<&str>)] = &[
    ("default", &[], None),
    ("--lang flag", &["--lang", "zh"], None),
    ("CE_LANG env", &[], Some("zh")),
];

/// The real binary. CE_LANG is removed unless the road under test
/// sets it, so an operator's ambient CE_LANG cannot silently move
/// which surface the default case measures.
fn run(args: &[&str], env: Option<&str>) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ce"));
    c.env_remove("CE_LANG").args(args);
    if let Some(v) = env {
        c.env("CE_LANG", v);
    }
    c.output().expect("run ce")
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
        for (road, prefix, env) in ROADS {
            let argv: Vec<&str> = prefix.iter().copied().chain(args.iter().copied()).collect();
            let out = run(&argv, *env);
            assert!(
                out.status.success(),
                "ce {argv:?} on the {road} road exit {:?}\n{}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(
                out.stderr.is_empty(),
                "ce {argv:?} on the {road} road must leave stderr empty: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            if *road != "default" {
                continue;
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            for want in *must_contain {
                assert!(
                    stdout.contains(want),
                    "ce {args:?} missing {want:?}:\n{stdout}"
                );
            }
        }
    }
}

/// The zh road answers the question in Chinese, not merely without
/// erroring — an exit-0 that printed the English help would satisfy
/// the leg above while leaving the reader exactly where they started.
#[test]
fn the_zh_road_answers_in_chinese() {
    for (road, prefix, env) in ROADS.iter().filter(|(r, ..)| *r != "default") {
        for args in [&[][..], &["graph"][..]] {
            let argv: Vec<&str> = prefix.iter().copied().chain(args.iter().copied()).collect();
            let stdout = String::from_utf8_lossy(&run(&argv, *env).stdout).into_owned();
            assert!(
                stdout
                    .chars()
                    .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
                "ce {argv:?} on the {road} road answered without a Chinese \
                 character:\n{stdout}"
            );
        }
    }
}

#[test]
fn unknown_subcommand_still_fails_loudly() {
    // the widened arm above accepts MissingSubcommand as a question;
    // a TYPO is InvalidSubcommand, a third kind, and must stay loud
    // on every road — otherwise the widening bought silence
    for (road, prefix, env) in ROADS {
        let argv: Vec<&str> = prefix
            .iter()
            .copied()
            .chain(["no-such-subcommand"])
            .collect();
        let out = run(&argv, *env);
        assert!(
            !out.status.success(),
            "a typo must not exit 0 on the {road} road"
        );
        assert!(
            !out.stderr.is_empty(),
            "the error goes to stderr on the {road} road"
        );
    }
}
