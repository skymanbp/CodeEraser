//! ce — CodeEraser CLI frontend: the clap surface only. Subcommand
//! bodies live in main_cmds.rs (split at the 300-line dogfood gate,
//! the RG13 plan). `doctor` (M0) + `scan` (M1) + `dedup`/`daemon`/
//! `ping` (M2) + hooks (M3) + `churn` (M4) + `graph`/`deadcode`
//! (M5-2).

mod main_cmds;
mod main_judge;

use clap::{Parser, Subcommand};
use codeeraser::daemon;
use main_cmds::{self as cmds, DedupArgs, OutFormat, json};
use main_judge::{CloneArgs, DocdupArgs};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "ce",
    version,
    about = "CodeEraser — erase LLM-induced code & document entropy",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Environment + project health: ce-core handshake, project
    /// status line, degradation counter (never starts the daemon)
    Doctor {
        /// Path to the ce-core executable
        #[arg(long, default_value = "ce-core")]
        core: String,
        /// Project root to report on (default: current directory)
        root: Option<PathBuf>,
    },
    /// Measure size / complexity / readability metrics (M1 modules)
    Scan {
        /// Directory to scan (default: current directory)
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
    /// Time-dimension metrics: append vs rewrite, windowed churn,
    /// co-change pairs (M4; report-only, feeds the M5 join)
    Churn {
        /// Repository root (default: current directory)
        root: Option<PathBuf>,
        /// History window in days
        #[arg(long, default_value_t = 14)]
        days: u32,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
    /// Dependency-graph subsystem: --sites lists reference sites
    /// (resolution-free); liveness lives under `ce deadcode`
    Graph {
        /// Directory to analyze (default: current directory)
        root: Option<PathBuf>,
        /// List reference sites
        #[arg(long)]
        sites: bool,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
    /// Judge liveness over the cached reference graph (M5-2h): the
    /// ladder's edges, the Haskell core's four-way verdicts
    Deadcode {
        /// Directory to judge (default: current directory)
        root: Option<PathBuf>,
        /// Index database path (default: <root>/.ce/index.db)
        #[arg(long)]
        db: Option<PathBuf>,
        /// Path to the ce-core executable
        #[arg(long, default_value = "ce-core")]
        core: String,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
    /// T3 near-miss clone judgment (M5-3): TED via the core's
    /// clone/1; --units lists the cached unit universe instead
    Clone(CloneArgs),
    /// Documentation-duplication judgment (M5-3g): exact Jaccard via
    /// the core's docdup/1 over the cached live segments
    Docdup(DocdupArgs),
    /// Detect T1/T2 clones via the winnowing fingerprint index (M2)
    Dedup(DedupArgs),
    /// Run the per-project daemon in the foreground (ADR-003);
    /// normally lazy-started by `ce ping` / hook probes
    Daemon {
        /// Project root to serve
        root: PathBuf,
    },
    /// Round-trip a ping through the project daemon (lazy-starts it)
    Ping {
        /// Project root (default: current directory)
        root: Option<PathBuf>,
    },
    /// PreToolUse cheap gate: read the hook envelope on stdin, probe
    /// the daemon, emit a permission decision per ce.toml [guard]
    Probe {
        /// Hook mode (the only mode in M3)
        #[arg(long)]
        hook: bool,
    },
    /// Stop audit v1: net LOC + duplicate blocks touching changed
    /// files (blocks the stop only in deny mode)
    Audit {
        /// Hook mode (the only mode in M3)
        #[arg(long)]
        hook: bool,
    },
    /// SessionStart health line + daemon warm-up
    Health {
        /// Hook mode (the only mode in M3)
        #[arg(long)]
        hook: bool,
    },
    /// pre-commit gate: staged net LOC + touched duplicates
    /// (exit 1 in deny mode when duplicates are touched)
    Precommit {
        /// Repository root (default: current directory)
        root: Option<PathBuf>,
    },
    /// Minimal MCP server over stdio: scan + check_duplication
    Mcp {
        /// Project root the tools operate on (default: current directory)
        root: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match analysis(Cli::parse().cmd) {
        Ok(code) => code,
        Err(cmd) => infra(cmd),
    }
}

/// Analysis-family dispatch (the metric/judgment subcommands); hands
/// back any command it does not own. The two-stage match keeps each
/// dispatcher under the repo's own complexity gate as families grow.
fn analysis(cmd: Cmd) -> Result<ExitCode, Cmd> {
    Ok(match cmd {
        Cmd::Scan { path, format } => cmds::scan_cmd(path, json(format)),
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
        } => cmds::deadcode_cmd(&cmds::or_cwd(root), db, &core, json(format)),
        Cmd::Clone(a) => main_judge::clone_cmd(a),
        Cmd::Docdup(a) => main_judge::docdup_cmd(a),
        Cmd::Dedup(a) => cmds::dedup_cmd(a),
        other => return Err(other),
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
        Cmd::Daemon { root } => cmds::serve_cmd("daemon", daemon::server::serve(&root)),
        Cmd::Ping { root } => cmds::ping_cmd(root),
        _ => unreachable!("analysis() owns every other command"),
    }
}
