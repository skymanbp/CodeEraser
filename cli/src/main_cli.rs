//! The clap surface (split from main.rs at the 300-line dogfood gate
//! when the bilingual help landed, M8-G3b). English help lives in
//! the doc comments exactly as it always did — zero tokens, so the
//! dedup ratchet sees no per-item attribute scaffolding (the
//! attribute-per-variant first draft minted 24 twin blocks and the
//! repo's own gate refused it). Help text answers only what a reader
//! can act on (batch 9 P5): what a command measures or judges,
//! whether it can fail the build, and what it costs — never a plan
//! coordinate the user cannot resolve.

use crate::main_cmds::{DedupArgs, FindingsFormat, OutFormat};
use crate::main_erase::EraseArgs;
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
        /// Path to the ce-core executable (default: CE_CORE_BIN, a
        /// ce-core beside this binary, then PATH)
        #[arg(long, default_value = "ce-core")]
        core: String,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
        /// Project root to report on (default: current directory)
        root: Option<PathBuf>,
    },
    /// Measure size / complexity / readability metrics; levels
    /// graded by the core
    Scan {
        /// Directory to scan (default: current directory)
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FindingsFormat::Console)]
        format: FindingsFormat,
        /// Path to the ce-core executable (default: CE_CORE_BIN, a
        /// ce-core beside this binary, then PATH)
        #[arg(long, default_value = "ce-core")]
        core: String,
    },
    /// Time-dimension metrics: append vs rewrite, windowed churn,
    /// co-change pairs (report-only; the join consumes them). Costs
    /// minutes on the default window — a git subprocess per commit
    /// and a blame per touched file; progress rides stderr
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
    /// (resolution-free); --mentions refreshes the mention universe
    /// and reports its header; liveness lives under `ce deadcode`
    Graph {
        /// Directory to analyze (default: current directory)
        root: Option<PathBuf>,
        /// List reference sites
        #[arg(long)]
        sites: bool,
        /// Refresh the mention universe (every text file the tree
        /// could reference a name from) and report what it holds
        #[arg(long, conflicts_with = "sites")]
        mentions: bool,
        /// Index database path (default: <root>/.ce/index.db)
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
    },
    /// Judge liveness over the cached reference graph: the ladder's
    /// edges, the core's four-way verdicts, and the symbol-level
    /// advisory (declarations no other file spells — never a verdict)
    Deadcode {
        /// Directory to judge (default: current directory)
        root: Option<PathBuf>,
        /// Index database path (default: <root>/.ce/index.db)
        #[arg(long)]
        db: Option<PathBuf>,
        /// Path to the ce-core executable (default: CE_CORE_BIN, a
        /// ce-core beside this binary, then PATH)
        #[arg(long, default_value = "ce-core")]
        core: String,
        #[arg(long, value_enum, default_value_t = OutFormat::Console)]
        format: OutFormat,
        /// Exit 1 when any file-tier dead verdict lands, or when the
        /// judgment itself degraded (a gate that could not judge
        /// never passes)
        #[arg(long)]
        check: bool,
    },
    /// T3 near-miss clone judgment: tree edit distance via the
    /// core's clone/1; --units lists the cached unit universe instead
    Clone(CloneArgs),
    /// Documentation-duplication judgment: exact Jaccard via the
    /// core's docdup/1 over the cached live segments
    Docdup(DocdupArgs),
    /// Three-signal join: similarity + graph position + per-unit
    /// churn, file and unit tiers (report-only). Costs a churn window
    /// plus a full index — minutes; progress rides stderr
    Join(JoinArgs),
    /// Tree-scale structure judgment: entropy, axes and findings via
    /// the core's structure/1 (report-only)
    Structure(StructureArgs),
    /// Score trajectory over mainline history: per-commit absolute
    /// check score, cached in the index, rebuildable. Each uncached
    /// commit is a full check in a temp worktree — bound a cold run
    /// with --batch; progress rides stderr
    Trend(TrendArgs),
    /// Deterministic two-phase eraser: plan what is provably safe to
    /// erase via the core's erase/1; dry-run by default
    Erase(EraseArgs),
    /// The ratchet gate: judge the repo against ce-baseline.json —
    /// ratchet OR --fail-under floor, either alone fails
    Check(CheckArgs),
    /// Persist the core's newBaseline as ce-baseline.json (the
    /// violation set only shrinks without CE_ACCEPT_BASELINE=1; a
    /// degraded judgment is never persisted)
    Baseline(BaselineArgs),
    /// Detect T1/T2 clones via the winnowing fingerprint index
    Dedup(DedupArgs),
    /// Run the per-project daemon in the foreground; normally
    /// lazy-started by `ce ping` / hook probes
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
        /// Hook mode: read the JSON envelope on stdin (required)
        #[arg(long)]
        hook: bool,
    },
    /// Stop audit v1: net LOC + duplicate blocks touching changed
    /// files (blocks the stop only in deny mode)
    Audit {
        /// Hook mode: read the JSON envelope on stdin (required)
        #[arg(long)]
        hook: bool,
    },
    /// SessionStart health line + daemon warm-up
    Health {
        /// Hook mode: read the JSON envelope on stdin (required)
        #[arg(long)]
        hook: bool,
    },
    /// pre-commit gate: staged net LOC + touched duplicates (exit 1
    /// in deny mode when duplicates are touched). FAIL-OPEN: with no
    /// reachable ce-core it reports the skip and exits 0 — the one
    /// CI-facing gate that passes on a missing core
    Precommit {
        /// Repository root (default: current directory)
        root: Option<PathBuf>,
    },
    /// MCP server over stdio: the read-only report face of every
    /// judgment family
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
