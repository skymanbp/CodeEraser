//! ce — CodeEraser CLI frontend.
//! `doctor` (ce-core handshake, M0) + `scan` (metrics, M1) +
//! `dedup` / `daemon` / `ping` (clone index + process model, M2).

use clap::{Parser, Subcommand, ValueEnum};
use codeeraser::{daemon, dedup, handshake, scan};
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
    /// Environment health: locate ce-core and verify the protocol handshake
    Doctor {
        /// Path to the ce-core executable
        #[arg(long, default_value = "ce-core")]
        core: String,
    },
    /// Measure size / complexity / readability metrics (M1 modules)
    Scan {
        /// Directory to scan (default: current directory)
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
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

#[derive(clap::Args)]
struct DedupArgs {
    /// Directory to index (default: current directory)
    path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutFormat::Console)]
    format: OutFormat,
    /// Index database path (default: <path>/.ce/index.db)
    #[arg(long)]
    db: Option<PathBuf>,
    /// Report threshold in normalized tokens (default: the
    /// winnowing guarantee threshold, 50)
    #[arg(long)]
    min_tokens: Option<usize>,
    /// Diversity floor: suppress blocks with fewer unique tokens
    /// (default 7, from the M2 calibration; 0 disables)
    #[arg(long)]
    min_distinct: Option<usize>,
    /// Only-shrink ratchet: exit 1 when clone blocks exceed the
    /// ce.toml [dedup] budget (M2 review R12)
    #[arg(long)]
    check: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutFormat {
    Console,
    Json,
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Doctor { core } => doctor(&core),
        Cmd::Scan { path, format } => scan_cmd(path, format),
        Cmd::Dedup(args) => dedup_cmd(args),
        Cmd::Probe { hook } => hook_cmd(hook, "probe", codeeraser::guard::run_hook),
        Cmd::Audit { hook } => hook_cmd(hook, "audit", codeeraser::audit::run_hook),
        Cmd::Health { hook } => hook_cmd(hook, "health", codeeraser::health::run_hook),
        Cmd::Precommit { root } => codeeraser::audit::run_precommit(&or_cwd(root)),
        Cmd::Mcp { root } => serve_cmd("mcp", codeeraser::mcp::serve(&or_cwd(root))),
        Cmd::Daemon { root } => serve_cmd("daemon", daemon::server::serve(&root)),
        Cmd::Ping { root } => ping_cmd(root),
    }
}

fn or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| PathBuf::from("."))
}

/// The three hook entries share one contract: --hook or nothing.
fn hook_cmd(hook: bool, name: &str, run: fn() -> ExitCode) -> ExitCode {
    if hook {
        run()
    } else {
        eprintln!("ce {name}: only --hook mode exists in M3");
        ExitCode::from(2)
    }
}

fn serve_cmd(name: &str, result: anyhow::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ce {name}: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn ping_cmd(root: Option<PathBuf>) -> ExitCode {
    let root = root.unwrap_or_else(|| PathBuf::from("."));
    let started = std::time::Instant::now();
    match daemon::client::request(&root, &daemon::proto::Request::Ping) {
        Ok(daemon::proto::Response::Pong { uptime_ms }) => {
            println!(
                "pong: daemon up {uptime_ms} ms, round-trip {} ms",
                started.elapsed().as_millis()
            );
            ExitCode::SUCCESS
        }
        Ok(other) => {
            eprintln!("ce ping: unexpected reply: {other:?}");
            ExitCode::from(2)
        }
        Err(err) => {
            eprintln!("ce ping: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn dedup_cmd(args: DedupArgs) -> ExitCode {
    let root = or_cwd(args.path);
    let opts = dedup::RunOpts {
        format: fmt(args.format),
        db: args.db,
        min_tokens: args.min_tokens,
        min_distinct: args.min_distinct,
        check: args.check,
    };
    match dedup::run(&root, opts) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("ce dedup: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn fmt(format: OutFormat) -> scan::Format {
    match format {
        OutFormat::Console => scan::Format::Console,
        OutFormat::Json => scan::Format::Json,
    }
}

fn scan_cmd(path: Option<PathBuf>, format: OutFormat) -> ExitCode {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    match scan::run(&root, fmt(format)) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("ce scan: {err:#}");
            ExitCode::from(2)
        }
    }
}

/// Environment + project health (plan §5.9-5): the same status line
/// the SessionStart hook emits, the A9f degraded-run counter from the
/// observe feed, then the ce-core handshake (which sets the exit
/// code, as in M0).
fn doctor(core: &str) -> ExitCode {
    println!(
        "ce {} (proto {})",
        env!("CARGO_PKG_VERSION"),
        handshake::PROTO
    );
    let root = PathBuf::from(".");
    println!("project: {}", codeeraser::health::status_line(&root));
    println!(
        "degraded runs (observe feed): {}",
        codeeraser::health::degraded_runs(&root)
    );
    match handshake::run(core) {
        Ok(reply) => {
            println!("ce-core {} (proto {})", reply.version, reply.proto);
            println!("handshake: OK");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("handshake: FAILED — {err}");
            ExitCode::from(2)
        }
    }
}
