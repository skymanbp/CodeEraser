//! ce — CodeEraser CLI frontend.
//! M0 scope: `--version` and `doctor` (spawns ce-core, verifies the
//! NDJSON handshake per contracts/VERSIONING.md).

mod handshake;

use clap::{Parser, Subcommand};
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
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Doctor { core } => doctor(&core),
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
