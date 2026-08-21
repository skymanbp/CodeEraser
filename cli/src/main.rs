//! ce — CodeEraser CLI frontend: dispatch only. The clap surface
//! lives in main_cli.rs (split at the 300-line dogfood gate when the
//! bilingual help landed, M8-G3b); subcommand bodies in main_cmds.rs
//! (the RG13 plan). The full subcommand roster is main_cli.rs's —
//! a milestone list here went stale nine subcommands deep.

mod main_cli;
mod main_cmds;
mod main_judge;
mod main_lang;
mod main_score;

use codeeraser::daemon;
use main_cli::{Cli, Cmd};
use main_cmds::{self as cmds, json};
use std::process::ExitCode;

fn main() -> ExitCode {
    // pin the language BEFORE clap builds the Command (the help text
    // switches too); the parsed field re-pins the same value, which
    // keeps the flag a real consumer of the declared arg
    codeeraser::i18n::init(main_cli::lang_from_argv().as_deref());
    let cli = parse_localized();
    codeeraser::i18n::init(cli.lang.as_deref());
    match analysis(cli.cmd) {
        Ok(code) => code,
        Err(cmd) => infra(*cmd),
    }
}

/// Parse through the (possibly localized) Command: zh swaps the help
/// strings off main_lang's tables; en is the derive's own untouched
/// surface, so the default help stays byte-identical by construction.
fn parse_localized() -> Cli {
    use clap::{CommandFactory, FromArgMatches};
    let mut cmd = Cli::command();
    if codeeraser::i18n::zh() {
        cmd = main_lang::localize(cmd);
    }
    let matches = match cmd.clone().try_get_matches() {
        Ok(m) => m,
        // A bare `ce` is a question, not a mistake (v0.7.3): the old
        // stderr + exit-2 usage error rendered as a red PowerShell
        // NativeCommandError wall. Overview help on stdout, exit 0;
        // real usage errors keep clap's own loud exit.
        Err(e) if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let _ = cmd.print_help();
            std::process::exit(0);
        }
        Err(e) => e.exit(),
    };
    Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())
}

/// Analysis-family dispatch (the metric/judgment subcommands); hands
/// back any command it does not own — boxed, because the handoff is
/// the Err lane and Cmd's largest variant crossed clippy's 128-byte
/// line when DedupArgs grew its --core (ADR-008 P2); one allocation
/// on a once-per-process parse path. The two-stage match keeps each
/// dispatcher under the repo's own complexity gate as families grow.
fn analysis(cmd: Cmd) -> Result<ExitCode, Box<Cmd>> {
    Ok(match cmd {
        Cmd::Scan { path, format, core } => cmds::scan_cmd(path, json(format), &core),
        Cmd::Churn { root, days, format } => {
            cmds::churn_cmd(&cmds::or_cwd(root), days, json(format))
        }
        Cmd::Graph {
            root,
            sites,
            format,
        } => cmds::graph_cmd(&cmds::or_cwd(root), sites, json(format)),
        Cmd::Deadcode {
            root,
            db,
            core,
            format,
            check,
        } => cmds::deadcode_cmd(&cmds::or_cwd(root), db, &core, json(format), check),
        Cmd::Clone(a) => main_judge::clone_cmd(a),
        Cmd::Docdup(a) => main_judge::docdup_cmd(a),
        Cmd::Join(a) => main_judge::join_cmd(a),
        Cmd::Structure(a) => main_judge::structure_cmd(a),
        Cmd::Trend(a) => main_judge::trend_cmd(a),
        Cmd::Check(a) => main_score::check_cmd(a),
        Cmd::Baseline(a) => main_score::baseline_cmd(a),
        Cmd::Dedup(a) => cmds::dedup_cmd(a),
        other => return Err(Box::new(other)),
    })
}

/// Infrastructure dispatch: health, daemon, hooks, servers.
fn infra(cmd: Cmd) -> ExitCode {
    match cmd {
        Cmd::Doctor { core, root } => cmds::doctor(&core, &cmds::or_cwd(root)),
        Cmd::Probe { hook } => cmds::hook_cmd(hook, "probe", codeeraser::guard::run_hook),
        Cmd::Audit { hook } => cmds::hook_cmd(hook, "audit", codeeraser::audit::run_hook),
        Cmd::Health { hook } => cmds::hook_cmd(hook, "health", codeeraser::health::run_hook),
        Cmd::Precommit { root } => codeeraser::audit::run_precommit(&cmds::or_cwd(root)),
        Cmd::Mcp { root } => cmds::serve_cmd("mcp", codeeraser::mcp::serve(&cmds::or_cwd(root))),
        Cmd::Eject { root, yes } => codeeraser::eject::run(&cmds::or_cwd(root), yes),
        Cmd::Daemon { root } => cmds::serve_cmd("daemon", daemon::server::serve(&root)),
        Cmd::Ping { root } => cmds::ping_cmd(root),
        _ => unreachable!("analysis() owns every other command"),
    }
}
