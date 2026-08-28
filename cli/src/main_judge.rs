//! Judgment-family subcommand bodies (`ce clone`, `ce docdup`,
//! `ce trend`, `ce structure`, `ce join`) — split from main_cmds.rs
//! when the docdup family pushed it over the repo's own 300-line
//! gate, with the shared flag set factored the same day the ratchet
//! caught DocdupArgs re-growing CloneArgs field-for-field; the other
//! three families moved in as they grew the same JudgeArgs shape.

use crate::main_cmds::{OutFormat, fail, json, or_cwd};
use codeeraser::{dedup, docdup, join};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The flag set every core-judgment family shares: root, output
/// format, core path, cache db. Families flatten this and add their
/// own switches (pub(crate): the score family lives in its own
/// module and reads the same set).
#[derive(clap::Args)]
pub struct JudgeArgs {
    /// Directory to analyze (default: current directory)
    pub(crate) root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutFormat::Console)]
    pub(crate) format: OutFormat,
    /// Path to the ce-core executable (default: CE_CORE_BIN, a
    /// ce-core beside this binary, then PATH)
    #[arg(long, default_value = "ce-core")]
    pub(crate) core: String,
    /// Index database path (default: <root>/.ce/index.db)
    #[arg(long)]
    pub(crate) db: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct CloneArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// List the unit universe instead of judging
    #[arg(long)]
    units: bool,
}

#[derive(clap::Args)]
pub struct DocdupArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Exit 1 when any duplication is reported (the CI dogfood gate)
    #[arg(long)]
    check: bool,
}

#[derive(clap::Args)]
pub struct JoinArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Churn window in days
    #[arg(long, default_value_t = 14)]
    days: u32,
}

#[derive(clap::Args)]
pub struct StructureArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Also roll clone blocks and dead units up per directory and
    /// judge the S6 redundancy axis (runs the dedup census and the
    /// liveness judgment; absent = the axis is honestly unjudged)
    #[arg(long)]
    deep: bool,
    /// Judge the S5 doc-staleness axis over this git window in days
    /// (docs whose referenced code changed after their last edit;
    /// absent = the axis is honestly unjudged)
    #[arg(long)]
    days: Option<u32>,
    /// Price a split for every judged file past the committed soft
    /// line: the best seam with its ROI, or an exemption whose
    /// numbers say why the file stays whole
    #[arg(long)]
    split_candidates: bool,
}

#[derive(clap::Args)]
pub struct TrendArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Mainline window: newest N first-parent commits
    #[arg(long, default_value_t = codeeraser::trend::DEFAULT_COMMITS)]
    commits: usize,
    /// Measure at most this many uncached commits per run (absent =
    /// all of them; the GUI passes small batches for progress)
    #[arg(long)]
    batch: Option<usize>,
}

/// The one flag-unpack + emit stanza for report families: unpack
/// JudgeArgs, run with (root, db, core), print with as_json, seat a
/// veto (`|_| None` for a family with no fail bit). structure/join
/// were shape twins and `ce trend` would have been the third — the
/// P4 ratchet caught the stanza; it exists once, and the no-veto
/// wrapper that once fronted it was itself a counted clone of this
/// signature (v2.18 subtraction batch).
fn family_checked<R>(
    j: JudgeArgs,
    name: &str,
    run: impl FnOnce(&Path, Option<PathBuf>, &str) -> anyhow::Result<R>,
    print: impl FnOnce(&R, bool),
    veto: impl FnOnce(&R) -> Option<String>,
) -> ExitCode {
    let as_json = json(j.format);
    emit_checked(
        name,
        || run(&or_cwd(j.root), j.db, &j.core),
        |r| print(r, as_json),
        veto,
    )
}

/// `ce trend` (M7-P4): the score trajectory over mainline history —
/// cached in the index db, rebuildable from git at will.
pub fn trend_cmd(a: TrendArgs) -> ExitCode {
    family_checked(
        a.judge,
        "trend",
        move |r, db, c| codeeraser::trend::run(r, db, c, a.commits, a.batch),
        codeeraser::trend::print,
        // declaring [trend] decline_floor_micro armed a fail bit that
        // NO exit code read: the console printed `-> FAIL` and exited
        // 0, so a CI leg watching the trajectory passed a declining
        // one. The core owns the verdict; this is its exit-code
        // throat (the dedup --check shape). "?" over a defaulted 0: a
        // fabricated number would be the report lying about why it
        // failed — and the floor prints as its VALUE in ‰/day of the
        // score scale, not as a ce.toml key name (batch 9 P15).
        |r| {
            let declined = r.judgment.fail.then(|| {
                let pm = |v: i64| format!("{:.1}", v as f64 / 1000.0);
                let floor = r
                    .judgment
                    .knobs
                    .iter()
                    .find(|[c, _]| *c == 1)
                    .map(|[_, v]| *v);
                codeeraser::i18n::line(
                    "score falls {}‰/day, past the declared {}‰/day decline floor",
                    "分数每日下跌 {}‰，越过声明的每日 {}‰ 下行地板",
                    &[
                        &r.judgment
                            .slope_micro_per_day
                            .map_or("?".to_string(), |s| pm(-s)),
                        &floor.map_or("?".to_string(), pm),
                    ],
                )
            });
            // a point that REFUSED to measure (an unseated submodule,
            // a worktree that would not add) reaches the console, the
            // JSON and the GUI, and reached no exit code — the P15
            // defect one seat over. Keyed on `failed`, never on
            // `pending`: --batch and the GUI's slice walk leave
            // pending > 0 by design, and a threshold would be policy.
            declined.or_else(|| {
                let (sha, why) = r.failed.first()?;
                Some(codeeraser::i18n::line(
                    "{} of {} window commits refused to measure: {} — {}",
                    "窗口内 {} / {} 个提交拒绝测量：{} —— {}",
                    &[&r.failed.len(), &r.window, sha, why],
                ))
            })
        },
    )
}

/// `ce structure` (M6 S2): the tree-scale entropy judgment —
/// aggregates to the core's structure/1, dense verdicts re-labelled
/// with local names. Report-only until a score floor lands (S3+).
pub fn structure_cmd(a: StructureArgs) -> ExitCode {
    family_checked(
        a.judge,
        "structure",
        move |r, db, c| {
            codeeraser::structure::judge::run(r, db, c, (a.deep, a.days, a.split_candidates))
        },
        codeeraser::structure::report::print,
        |_| None,
    )
}

/// `ce join` (M5-3h): assemble the three signal legs — similarity,
/// graph position, per-unit churn — report-only; the verdict lattice
/// judges them on the verdict/1 wire via `ce check` (M5-3i).
pub fn join_cmd(a: JoinArgs) -> ExitCode {
    family_checked(
        a.judge,
        "join",
        move |r, db, c| join::run(r, db, c, a.days),
        join::print,
        |_| None,
    )
}

/// Run one family's judgment and print its report — the ONE
/// run/print/fail shape every judgment command is; the veto seat is
/// how a --check style gate turns a printed report into exit 1 with
/// a named reason (the dedup --check shape), `|_| None` for a family
/// without one.
fn emit_checked<R>(
    name: &str,
    run: impl FnOnce() -> anyhow::Result<R>,
    print: impl FnOnce(&R),
    veto: impl FnOnce(&R) -> Option<String>,
) -> ExitCode {
    match run() {
        Ok(report) => {
            print(&report);
            if let Some(why) = veto(&report) {
                eprintln!("{name} check: {why}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(err) => fail(name, err),
    }
}

/// `ce docdup` (M5-3g): the documentation-duplication judgment over
/// the live cached segments — shingle sets to the core in chunks,
/// raw inter/union plus the core's verdict bits back (ADR-008 P1).
pub fn docdup_cmd(a: DocdupArgs) -> ExitCode {
    let j = a.judge;
    let as_json = json(j.format);
    emit_checked(
        "docdup",
        || docdup::judge::run(&or_cwd(j.root), j.db, &j.core),
        |r| docdup::judge::print(r, as_json),
        |r| {
            (a.check && !r.hits.is_empty()).then(|| {
                codeeraser::i18n::line(
                    "{} reported duplication(s) — resolve or exempt them",
                    "{} 处重复被报告 — 请解决或豁免",
                    &[&r.hits.len()],
                )
            })
        },
    )
}

/// `ce clone` (M5-3e): the T3 TED judgment over the frozen candidate
/// pass — trees to the core in chunks, raw scores plus the core's
/// verdict bits back (ADR-008 P1). `--units` (M5-3b) instead lists
/// the cached unit universe after asserting the unitsig/symbols
/// identity agreement (zero orphans — the nth throat is one
/// function, checked, not assumed).
pub fn clone_cmd(a: CloneArgs) -> ExitCode {
    let j = a.judge;
    let as_json = json(j.format);
    if !a.units {
        return emit_checked(
            "clone",
            || dedup::t3::run(&or_cwd(j.root), j.db, &j.core),
            |r| dedup::t3::print(r, as_json),
            |_| None,
        );
    }
    emit_checked(
        "clone",
        || codeeraser::faces::clone_units(&or_cwd(j.root)),
        |doc| print_units(doc, as_json),
        |_| None,
    )
}

/// The console face of the unit universe. The DOCUMENT is built in
/// faces::clone_units — it used to be assembled here, which made it
/// the one family document no machine surface could reach, and the
/// `--units` MCP gap was that fact wearing a different hat.
fn print_units(doc: &serde_json::Value, json: bool) {
    if json {
        println!("{doc}");
        return;
    }
    let units = doc["units"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    for u in units {
        println!(
            "{}  {}#{}  {} nodes",
            u["path"].as_str().unwrap_or("?"),
            u["key"].as_str().unwrap_or("?"),
            u["nth"],
            u["nodes"]
        );
    }
    println!(
        "{}",
        codeeraser::i18n::line("clone units: {}", "克隆单元：{}", &[&units.len()])
    );
}
