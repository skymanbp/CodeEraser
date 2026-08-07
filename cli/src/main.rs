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
    Dedup {
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
    },
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
        Cmd::Dedup {
            path,
            format,
            db,
            min_tokens,
        } => dedup_cmd(path, format, db, min_tokens),
        Cmd::Daemon { root } => match daemon::server::serve(&root) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("ce daemon: {err:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Ping { root } => ping_cmd(root),
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

fn dedup_cmd(
    path: Option<PathBuf>,
    format: OutFormat,
    db: Option<PathBuf>,
    min_tokens: Option<usize>,
) -> ExitCode {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    match dedup::run(&root, fmt(format), db, min_tokens) {
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

fn doctor(core: &str) -> ExitCode {
    println!(
        "ce {} (proto {})",
        env!("CARGO_PKG_VERSION"),
        handshake::PROTO
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
