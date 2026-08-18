//! The clap surface (split from main.rs at the 300-line dogfood gate
//! when the bilingual help landed, M8-G3b). English help lives in
//! the doc comments exactly as it always did — zero tokens, so the
//! dedup ratchet sees no per-item attribute scaffolding (the
//! attribute-per-variant first draft minted 24 twin blocks and the
//! repo's own gate refused it). Chinese arrives at runtime through
//! main_lang::localize over the built Command.

use crate::main_cmds::{DedupArgs, OutFormat};
use crate::main_judge::{CloneArgs, DocdupArgs, JoinArgs, StructureArgs, TrendArgs};
use crate::main_score::{BaselineArgs, CheckArgs};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ce",
    // usage lines otherwise render argv[0] — "ce.exe" on Windows,
    // "ce" elsewhere; the docs gate regenerates the same page on
    // every platform only when the canonical name is pinned
    bin_name = "ce",
    version,
    about = "CodeEraser — erase LLM-induced code & document entropy",
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// Console language (wins over CE_LANG)
    #[arg(long, global = true, value_parser = ["en", "zh"])]
    pub(crate) lang: Option<String>,
    #[command(subcommand)]
    pub(crate) cmd: Cmd,
}

/// Read `--lang` straight off argv (both `--lang zh` and `--lang=zh`)
/// — the pin must land before clap builds the Command, or the help
/// text could not switch on the same invocation. The clap arg above
/// still declares, documents and validates the flag.
pub(crate) fn lang_from_argv() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--lang")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| {
            args.iter()
                .find_map(|a| a.strip_prefix("--lang=").map(str::to_string))
        })
}

#[derive(Subcommand)]
pub(crate) enum Cmd {
    /// Environment + project health: ce-core handshake, project
    /// status line, degradation counter (never starts the daemon)
    Doctor {
        /// Path to the ce-core executable
        #[arg(long, default_value = "ce-core")]
        core: String,
        /// Project root to report on (default: current directory)
        root: Option<PathBuf>,
    },
    /// Measure size / complexity / readability metrics (M1 modules;
    /// levels graded by the core since ADR-008 P3)
    Scan {
        /// Directory to scan (default: current directory)
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
        /// Path to the ce-core executable
        #[arg(long, default_value = "ce-core")]
        core: String,
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
        /// Exit 1 when any file-tier dead verdict lands (the CI
        /// dogfood gate for the M5-2 full-disposition acceptance row)
        #[arg(long)]
        check: bool,
    },
    /// T3 near-miss clone judgment (M5-3): TED via the core's
    /// clone/1; --units lists the cached unit universe instead
    Clone(CloneArgs),
    /// Documentation-duplication judgment (M5-3g): exact Jaccard via
    /// the core's docdup/1 over the cached live segments
    Docdup(DocdupArgs),
    /// Three-signal join (M5-3h): similarity + graph position +
    /// per-unit churn, file and unit tiers (report-only until 3i)
    Join(JoinArgs),
    /// Tree-scale structure judgment (M6): entropy, axes and
    /// findings via the core's structure/1 (report-only in S2)
    Structure(StructureArgs),
    /// Score trajectory over mainline history (M7-P4): per-commit
    /// absolute check score, cached in the index, rebuildable
    Trend(TrendArgs),
    /// ADR-006 gate (M5-3i): judge the repo against ce-baseline.json
    /// — ratchet OR --fail-under floor, either alone fails
    Check(CheckArgs),
    /// Persist the core's newBaseline as ce-baseline.json (the
    /// violation set only shrinks without CE_ACCEPT_BASELINE=1)
    Baseline(BaselineArgs),
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
    /// MCP server over stdio: the read-only report face of every
    /// judgment family (M7-P2)
    Mcp {
        /// Project root the tools operate on (default: current directory)
        root: Option<PathBuf>,
    },
    /// Uninstall project state: .ce/, baseline, pins (dry-run default)
    Eject {
        /// Project root to eject (default: current directory)
        root: Option<PathBuf>,
        /// Actually remove (default: dry run naming every target)
        #[arg(long)]
        yes: bool,
    },
}
