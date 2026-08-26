//! Subcommand bodies for the `ce` binary (split from main.rs at the
//! 300-line dogfood gate — the RG13 plan written into the M5-2
//! design). main_cli.rs owns the clap surface (its own G3b split),
//! main.rs dispatches; this file owns the work.

use clap::ValueEnum;
use codeeraser::i18n::line;
use codeeraser::{churn, daemon, dedup, graph, scan};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Copy, ValueEnum)]
pub enum OutFormat {
    Console,
    Json,
}

/// The two finding-shaped commands (scan, dedup) alone take the wide
/// format — sarif in the help of a command whose report carries no
/// physical locations would be a promise nothing renders.
#[derive(Clone, Copy, ValueEnum)]
pub enum FindingsFormat {
    Console,
    Json,
    Sarif,
}

pub fn json(format: OutFormat) -> bool {
    matches!(format, OutFormat::Json)
}

pub fn or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| PathBuf::from("."))
}

pub fn findings_fmt(f: FindingsFormat) -> scan::Format {
    match f {
        FindingsFormat::Console => scan::Format::Console,
        FindingsFormat::Json => scan::Format::Json,
        FindingsFormat::Sarif => scan::Format::Sarif,
    }
}

pub fn scan_cmd(path: Option<PathBuf>, format: FindingsFormat, core: &str) -> ExitCode {
    fallible("scan", scan::run(&or_cwd(path), findings_fmt(format), core))
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
        // the two forms are the SAME ascii on both roads, so the
        // column padding survives translation untouched
        println!(
            "{}",
            line(
                "ce graph --sites [--format json]   list the reference sites",
                "ce graph --sites [--format json]   列出引用站点",
                &[],
            )
        );
        println!(
            "{}",
            line(
                "ce deadcode                        judge liveness over them",
                "ce deadcode                        在其上判决存活性",
                &[],
            )
        );
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
            graph::deadcode::print(&report, json);
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

/// The dedup flag set (clap surface + body in one place: six loose
/// params would trip the project's own params threshold; the flag
/// SURFACE is unchanged from its main.rs days).
#[derive(clap::Args)]
pub struct DedupArgs {
    /// Directory to index (default: current directory)
    path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FindingsFormat::Console)]
    format: FindingsFormat,
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
        format: findings_fmt(a.format),
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
        // exit 1 (not 0) is the point of the arm above: a harness that
        // forgot --hook must not read as ALLOW. So this stays a
        // failure and only the WORDS join the switch.
        eprintln!(
            "{}",
            line(
                "ce {}: pass --hook — this command reads a hook envelope on stdin",
                "ce {}：请传 --hook — 本命令自 stdin 读取钩子信封",
                &[&name],
            )
        );
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
/// The console face of the doctor DOCUMENT (K round step 6): the
/// facts are measured once, in health::doctor, and this renders
/// them. The GUI screen and the MCP surface render the same object,
/// so a diagnostic cannot say one thing here and another there.
/// Every fact hangs off the project ANCHOR above the given path
/// (ce.toml, baseline, .ce/index.db — root.rs), so `ce doctor cli`
/// reports the enclosing project and NAMES it: a silent re-root is
/// the half of that defect the operator pays for.
pub fn doctor(core: &str, root: &Path, as_json: bool) -> ExitCode {
    let d = codeeraser::health::doctor::document(root, core);
    if as_json {
        // the exit code still relays the handshake: a JSON face
        // that always exits 0 would make `ce doctor --format json`
        // the one diagnostic a script cannot gate on
        println!("{d}");
        return match d["core"]["handshake"] == serde_json::json!(true) {
            true => ExitCode::SUCCESS,
            false => ExitCode::from(2),
        };
    }
    // the renderer lives beside the measurement (health::doctor);
    // this owns only what a console face owns — which stream each
    // line takes, and the exit code
    let (lines, ok) = codeeraser::health::doctor::console(&d);
    let (last, head) = lines.split_last().expect("console yields the verdict");
    for l in head {
        println!("{l}");
    }
    if ok {
        println!("{last}");
        return ExitCode::SUCCESS;
    }
    eprintln!("{last}");
    ExitCode::from(2)
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
