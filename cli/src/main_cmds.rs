//! Subcommand bodies for the `ce` binary (split from main.rs at the
//! 300-line dogfood gate — the RG13 plan written into the M5-2
//! design). main_cli.rs owns the clap surface (its own G3b split),
//! main.rs dispatches; this file owns the work.

use clap::ValueEnum;
use codeeraser::i18n::line;
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

pub fn scan_cmd(path: Option<PathBuf>, json: bool, core: &str) -> ExitCode {
    fallible("scan", scan::run(&or_cwd(path), fmt(json), core))
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
        // A bare `ce graph` is a question, not a mistake — the same
        // reading v0.7.3 gave a bare `ce`. It used to answer on
        // stderr with exit 2, which PowerShell repaints as a red
        // NativeCommandError wall and every wrapper reads as a
        // failure. The usable form goes to stdout and the exit is
        // clean; --format is named here because it is inert without
        // --sites, which was the second half of the same puzzle.
        println!("ce graph --sites [--format json]   list the reference sites");
        println!("ce deadcode                        judge liveness over them");
        return ExitCode::SUCCESS;
    }
    graph::run_sites(root, json)
}

pub fn deadcode_cmd(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    json: bool,
    check: bool,
) -> ExitCode {
    match graph::deadcode::run(root, db, core) {
        Ok(report) => {
            print_deadcode(&report, json);
            // --check (M5-close CI gate): the DECISION is the core's
            // fail bit since 2.18.0 (batch-7 slice 4) — any file-tier
            // dead verdict, or a degraded run that judged nothing,
            // fails; this arm only picks which message renders. The
            // M5-2 acceptance row "本仓库 deadcode 发现全处置" was
            // honored by discipline only before the gate existed.
            if check && report.fail {
                if report.degraded.is_some() {
                    eprintln!(
                        "{}",
                        line(
                            "deadcode check: degraded ({}) — nothing was judged, refusing to pass",
                            "deadcode check：已降级（{}）— 未判决任何内容，拒绝通过",
                            &[&report.degraded.as_deref().unwrap_or("?")],
                        )
                    );
                } else {
                    eprintln!(
                        "{}",
                        line(
                            "deadcode check: {} dead file(s) — disposition or entry_globs them",
                            "deadcode check：{} 个死文件 — 请处置或加入 entry_globs",
                            &[&report.dead.len()],
                        )
                    );
                }
                return ExitCode::FAILURE;
            }
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
        println!("{}", codeeraser::report::deadcode_json(r));
        return;
    }
    for d in &r.dead {
        println!(
            "{}",
            line(
                "dead: {}  {}  ({}){}",
                "死件：{}  {}（{}）{}",
                &[
                    &d.path,
                    &d.verdict,
                    &d.why,
                    &graph::deadcode::conf_word(d.conf)
                ],
            )
        );
    }
    for (name, verdict) in &r.reported {
        println!(
            "{}",
            line(
                "aggregate: {}  {}  (reported, never dead — decision 4)",
                "聚合件：{}  {}（仅报告，永不判死 — 决议 4）",
                &[name, verdict],
            )
        );
    }
    print_deadcode_tail(r);
}

/// The summary + degraded lines (split at the 50-line fn gate).
fn print_deadcode_tail(r: &graph::deadcode::Report) {
    println!(
        "{}",
        line(
            "deadcode: {} nodes, {} kept edges, {} dead, {} aggregate reports, \
             {} unresolved sites (verdicts assume none lands in-corpus)",
            "死码：{} 节点，{} 保留边，{} 死件，{} 聚合报告，{} 未解析调用点（判决假设它们皆不落语料内）",
            &[
                &r.nodes,
                &r.kept,
                &r.dead.len(),
                &r.reported.len(),
                &r.unresolved_sites,
            ],
        )
    );
    if let Some(reason) = &r.degraded {
        println!(
            "{}",
            line(
                "degraded: {} (nothing was analyzed)",
                "降级：{}（未分析任何内容）",
                &[reason],
            )
        );
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
    /// (default 7, from measured calibration; 0 disables)
    #[arg(long)]
    min_distinct: Option<usize>,
    /// Only-shrink ratchet: exit 1 when clone blocks exceed the
    /// ce.toml [dedup] budget (the comparison is the core's verdict;
    /// a degraded judgment refuses to gate at all and exits 2)
    #[arg(long)]
    check: bool,
    /// Path to the ce-core executable, consulted by --check alone
    /// (default: CE_CORE_BIN, a ce-core beside this binary, then PATH)
    #[arg(long, default_value = "ce-core")]
    core: String,
}

pub fn dedup_cmd(a: DedupArgs) -> ExitCode {
    let opts = dedup::RunOpts {
        format: fmt(json(a.format)),
        db: a.db,
        min_tokens: a.min_tokens,
        min_distinct: a.min_distinct,
        check: a.check,
        core: a.core,
    };
    match dedup::run(&or_cwd(a.path), opts) {
        Ok(code) => code,
        Err(err) => fail("dedup", err),
    }
}

/// The three hook entries share one contract: --hook or nothing.
///
/// The usage error exits 1, NOT 2: exit 2 IS the hook protocol's DENY
/// (PreToolUse blocks the call and feeds stderr to the model), so a
/// `ce` too old to know `--hook` — the version skew a machine-PATH
/// install makes routine — answered every probe with a hard deny
/// whose reason was a usage string. 2 stays reserved for a verdict.
pub fn hook_cmd(hook: bool, name: &str, run: fn() -> ExitCode) -> ExitCode {
    if hook {
        run()
    } else {
        eprintln!("ce {name}: pass --hook — this command reads a hook envelope on stdin");
        ExitCode::FAILURE
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
    // Every fact below hangs off the project ANCHOR above the given
    // path (ce.toml, baseline, .ce/index.db — root.rs), so `ce doctor
    // cli` reported the enclosing project while printing nothing that
    // said so. Resolve once and NAME it: a silent re-root is the half
    // of that defect the operator pays for.
    let root = &codeeraser::root::project_root(root);
    println!(
        "ce {} (proto {})",
        env!("CARGO_PKG_VERSION"),
        corelink::PROTO
    );
    println!(
        "{}",
        line(
            "project: {} {}",
            "项目：{} {}",
            &[&root.display(), &codeeraser::health::doctor_line(root)],
        )
    );
    let (degraded, total) = codeeraser::health::degraded_runs(root);
    println!(
        "{}",
        line(
            "degraded runs (observe feed): {} of {} entries",
            "降级运行（observe 流水）：{} / {} 条",
            &[&degraded, &total],
        )
    );
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
