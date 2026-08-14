//! Subcommand bodies for the `ce` binary (split from main.rs at the
//! 300-line dogfood gate — the RG13 plan written into the M5-2
//! design). main.rs owns the clap surface; this file owns the work.

use clap::ValueEnum;
use codeeraser::{churn, corelink, daemon, dedup, graph, scan};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Copy, ValueEnum)]
pub enum OutFormat {
    Console,
    Json,
}

pub fn json(format: OutFormat) -> bool {
    matches!(format, OutFormat::Json)
}

pub fn or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| PathBuf::from("."))
}

pub fn fmt(json: bool) -> scan::Format {
    if json {
        scan::Format::Json
    } else {
        scan::Format::Console
    }
}

pub fn scan_cmd(path: Option<PathBuf>, json: bool) -> ExitCode {
    fallible("scan", scan::run(&or_cwd(path), fmt(json)))
}

pub fn churn_cmd(root: &Path, days: u32, json: bool) -> ExitCode {
    match churn::run(root, days) {
        Ok(report) => {
            if json {
                println!("{}", churn::report_json(&report));
            } else {
                churn::print_console(&report, days);
            }
            ExitCode::SUCCESS
        }
        Err(err) => fail("churn", err),
    }
}

pub fn graph_cmd(root: &Path, sites: bool, json: bool) -> ExitCode {
    if !sites {
        eprintln!("ce graph: only --sites exists (deadcode is its own subcommand)");
        return ExitCode::from(2);
    }
    graph::run_sites(root, json)
}

pub fn deadcode_cmd(root: &Path, db: Option<PathBuf>, core: &str, json: bool) -> ExitCode {
    match graph::deadcode::run(root, db, core) {
        Ok(report) => {
            print_deadcode(&report, json);
            ExitCode::SUCCESS
        }
        Err(err) => fail("deadcode", err),
    }
}

/// Section/package aggregates are reported, never called dead
/// (decision 4); the unresolved-site count rides every summary — the
/// verdicts assume none of those sites lands in-corpus, and the
/// reader sees it.
fn print_deadcode(r: &graph::deadcode::Report, json: bool) {
    if json {
        let doc = serde_json::json!({
            "schema": "ce.deadcode-report/0.1.0",
            "dead": r.dead.iter().map(|(n, v, w)| {
                serde_json::json!({"name": n, "verdict": v, "why": w})
            }).collect::<Vec<_>>(),
            "reported": r.reported.iter().map(|(n, v)| {
                serde_json::json!({"name": n, "verdict": v})
            }).collect::<Vec<_>>(),
            "counts": {"nodes": r.nodes, "kept_edges": r.kept},
            "unresolved_sites": r.unresolved_sites,
            "degraded": r.degraded,
        });
        println!("{doc}");
        return;
    }
    for (name, verdict, why) in &r.dead {
        println!("dead: {name}  {verdict}  ({why})");
    }
    for (name, verdict) in &r.reported {
        println!("aggregate: {name}  {verdict}  (reported, never dead — decision 4)");
    }
    println!(
        "deadcode: {} nodes, {} kept edges, {} dead, {} aggregate reports, \
         {} unresolved sites (verdicts assume none lands in-corpus)",
        r.nodes,
        r.kept,
        r.dead.len(),
        r.reported.len(),
        r.unresolved_sites
    );
    if let Some(reason) = &r.degraded {
        println!("degraded: {reason} (nothing was analyzed)");
    }
}

/// The dedup flag set (clap surface + body in one place: six loose
/// params would trip the project's own params threshold; the flag
/// SURFACE is unchanged from its main.rs days).
#[derive(clap::Args)]
pub struct DedupArgs {
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

pub fn dedup_cmd(a: DedupArgs) -> ExitCode {
    let opts = dedup::RunOpts {
        format: fmt(json(a.format)),
        db: a.db,
        min_tokens: a.min_tokens,
        min_distinct: a.min_distinct,
        check: a.check,
    };
    match dedup::run(&or_cwd(a.path), opts) {
        Ok(code) => code,
        Err(err) => fail("dedup", err),
    }
}

/// The three hook entries share one contract: --hook or nothing.
pub fn hook_cmd(hook: bool, name: &str, run: fn() -> ExitCode) -> ExitCode {
    if hook {
        run()
    } else {
        eprintln!("ce {name}: only --hook mode exists in M3");
        ExitCode::from(2)
    }
}

pub fn serve_cmd(name: &str, result: anyhow::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(name, err),
    }
}

pub fn ping_cmd(root: Option<PathBuf>) -> ExitCode {
    let root = or_cwd(root);
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
        Err(err) => fail("ping", err),
    }
}

/// Environment + project health (plan §5.9-5): non-spawning project
/// status line, the A9f degraded-run counter from the observe feed,
/// then the ce-core handshake (which sets the exit code, as in M0).
pub fn doctor(core: &str, root: &Path) -> ExitCode {
    println!(
        "ce {} (proto {})",
        env!("CARGO_PKG_VERSION"),
        corelink::PROTO
    );
    println!("project: {}", codeeraser::health::doctor_line(root));
    let (degraded, total) = codeeraser::health::degraded_runs(root);
    println!("degraded runs (observe feed): {degraded} of {total} entries");
    match corelink::run(core) {
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

fn fallible(name: &str, result: anyhow::Result<ExitCode>) -> ExitCode {
    match result {
        Ok(code) => code,
        Err(err) => fail(name, err),
    }
}

pub fn fail(name: &str, err: anyhow::Error) -> ExitCode {
    eprintln!("ce {name}: {err:#}");
    ExitCode::from(2)
}
