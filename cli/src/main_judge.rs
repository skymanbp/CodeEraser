//! Judgment-family subcommand bodies (`ce clone`, `ce docdup`) —
//! split from main_cmds.rs when the docdup family pushed it over the
//! repo's own 300-line gate, with the shared flag set factored the
//! same day the ratchet caught DocdupArgs re-growing CloneArgs
//! field-for-field.

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
    /// Path to the ce-core executable
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
    /// Exit 1 when any duplication is reported (the CI dogfood gate —
    /// plan §7.5's docdup clause, honored in code since M5 close)
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
}

/// `ce structure` (M6 S2): the tree-scale entropy judgment —
/// aggregates to the core's structure/1, dense verdicts re-labelled
/// with local names. Report-only until a score floor lands (S3+).
pub fn structure_cmd(a: StructureArgs) -> ExitCode {
    let j = a.judge;
    let as_json = json(j.format);
    emit(
        "structure",
        || codeeraser::structure::judge::run(&or_cwd(j.root), j.db, &j.core, a.deep),
        |r| codeeraser::structure::judge::print(r, as_json),
    )
}

/// `ce join` (M5-3h): assemble the three signal legs — similarity,
/// graph position, per-unit churn — report-only until the verdict
/// lattice's wire hookup (3i).
pub fn join_cmd(a: JoinArgs) -> ExitCode {
    let j = a.judge;
    let as_json = json(j.format);
    emit(
        "join",
        || join::run(&or_cwd(j.root), j.db, &j.core, a.days),
        |r| join::print(r, as_json),
    )
}

/// Run one family's judgment and print its report — the ONE
/// run/print/fail shape every judgment command is.
fn emit<R>(
    name: &str,
    run: impl FnOnce() -> anyhow::Result<R>,
    print: impl FnOnce(&R),
) -> ExitCode {
    emit_checked(name, run, print, |_| None)
}

/// emit plus a veto: a --check style gate turns a printed report
/// into exit 1 with a named reason — one throat for every checkable
/// judgment family (the dedup --check shape).
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
                format!(
                    "{} reported duplication(s) — resolve or exempt them",
                    r.hits.len()
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
        return emit(
            "clone",
            || dedup::t3::run(&or_cwd(j.root), j.db, &j.core),
            |r| dedup::t3::print(r, as_json),
        );
    }
    emit(
        "clone",
        || clone_units(&or_cwd(j.root), j.db),
        |rows| print_units(rows, as_json),
    )
}

fn clone_units(root: &Path, db: Option<PathBuf>) -> anyhow::Result<Vec<dedup::unitcache::UnitRow>> {
    let (idx, _db_path) = dedup::refreshed_index(root, db)?;
    let orphans = dedup::unitcache::identity_orphans(&idx)?;
    anyhow::ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    dedup::unitcache::unit_rows(&idx)
}

fn print_units(rows: &[dedup::unitcache::UnitRow], json: bool) {
    if json {
        let doc = serde_json::json!({
            "schema": "ce.clone-units/0.1.0",
            "units": rows.iter().map(|u| serde_json::json!({
                "path": u.path, "key": u.key, "nth": u.nth, "nodes": u.nodes,
            })).collect::<Vec<_>>(),
        });
        println!("{doc}");
        return;
    }
    for u in rows {
        println!("{}  {}#{}  {} nodes", u.path, u.key, u.nth, u.nodes);
    }
    println!("clone units: {}", rows.len());
}
