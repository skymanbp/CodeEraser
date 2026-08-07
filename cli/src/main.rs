//! ce — CodeEraser CLI frontend.
//! `doctor` (ce-core handshake, M0) + `scan` (metrics, M1) +
//! `dedup` (clone index, M2).

use clap::{Parser, Subcommand, ValueEnum};
use codeeraser::{dedup, handshake, scan};
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
        Cmd::Dedup { path, format, db } => dedup_cmd(path, format, db),
    }
}

fn dedup_cmd(path: Option<PathBuf>, format: OutFormat, db: Option<PathBuf>) -> ExitCode {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    match dedup::run(&root, fmt(format), db) {
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
