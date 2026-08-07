//! ce — CodeEraser CLI frontend.
//! `doctor` (ce-core handshake, M0) + `scan` (metrics, M1).

use clap::{Parser, Subcommand, ValueEnum};
use codeeraser::{handshake, scan};
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
    }
}

fn scan_cmd(path: Option<PathBuf>, format: OutFormat) -> ExitCode {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let fmt = match format {
        OutFormat::Console => scan::Format::Console,
        OutFormat::Json => scan::Format::Json,
    };
    match scan::run(&root, fmt) {
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
