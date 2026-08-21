//! Bare `ce` answers with help, not an error (v0.7.3). Before: the
//! arg_required_else_help usage error went to stderr with exit 2,
//! and PowerShell 5.1 wraps any native stderr in a NativeCommandError
//! wall — the first thing a fresh installer user typed looked like a
//! crash. A wrong subcommand must still be a real error.

use std::process::Command;

#[test]
fn bare_invocation_prints_overview_help_on_stdout_and_exits_zero() {
    let out = Command::new(env!("CARGO_BIN_EXE_ce"))
        .env_remove("CE_LANG") // the default (en) surface is the contract
        .output()
        .expect("run ce");
    assert!(out.status.success(), "exit {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Usage: ce"), "overview help:\n{stdout}");
    assert!(stdout.contains("doctor"), "subcommand roster:\n{stdout}");
    assert!(
        out.stderr.is_empty(),
        "stderr must stay empty: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn unknown_subcommand_still_fails_loudly() {
    let out = Command::new(env!("CARGO_BIN_EXE_ce"))
        .env_remove("CE_LANG")
        .arg("no-such-subcommand")
        .output()
        .expect("run ce");
    assert!(!out.status.success(), "a typo must not exit 0");
    assert!(!out.stderr.is_empty(), "the error goes to stderr");
}
