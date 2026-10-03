//! Judgment-family subcommand bodies (`ce clone`, `ce docdup`,
//! `ce trend`, `ce structure`, `ce join`) — split from main_cmds.rs
//! when the docdup family pushed it over the repo's own 300-line
//! gate, with the shared flag set factored the same day the ratchet
//! caught DocdupArgs re-growing CloneArgs field-for-field; the other
//! three families moved in as they grew the same JudgeArgs shape.

use crate::main_cmds::{OutFormat, answered, fail, json, or_cwd};
use codeeraser::{dedup, docdup, join};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The flag set every core-judgment family shares: root, output
/// format, cache db (the core is the process's global `--core`).
/// Families flatten this and add their own switches (pub(crate): the
/// score family lives in its own module and reads the same set).
#[derive(clap::Args)]
pub struct JudgeArgs {
    /// Directory to analyze (default: current directory)
    pub(crate) root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutFormat::Console)]
    pub(crate) format: OutFormat,
    /// Index database path (default: <root>/.ce/index.db)
    #[arg(long)]
    pub(crate) db: Option<PathBuf>,
}

impl JudgeArgs {
    /// A family the core answers (plan v2.32 step 5): `ask` gets the
    /// root and the db, the answer is printed in the asked form and the
    /// exit reads the core's veto (main_cmds::answered) — the one body
    /// of `ce docdup`, `ce trend` and `ce similar`.
    pub(crate) fn answered(
        self,
        family: &str,
        ask: impl FnOnce(&Path, Option<PathBuf>) -> anyhow::Result<codeeraser::document::Answer>,
    ) -> ExitCode {
        let as_json = json(self.format);
        answered(family, family, as_json, ask(&or_cwd(self.root), self.db))
    }
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
/// JudgeArgs beside the process's core, run with (root, db, core), print with as_json, seat a
/// veto (`|_| None` for a family with no fail bit). structure/join
/// were shape twins and `ce trend` would have been the third — the
/// P4 ratchet caught the stanza; it exists once, and the no-veto
/// wrapper that once fronted it was itself a counted clone of this
/// signature (v2.18 subtraction batch).
fn family_checked<R>(
    (j, core): (JudgeArgs, &str),
    name: &str,
    run: impl FnOnce(&Path, Option<PathBuf>, &str) -> anyhow::Result<R>,
    print: impl FnOnce(&R, bool),
    veto: impl FnOnce(&R) -> Option<String>,
) -> ExitCode {
    let as_json = json(j.format);
    emit_checked(
        name,
        || run(&or_cwd(j.root), j.db, core),
        |r| print(r, as_json),
        veto,
    )
}

/// `ce trend` (M7-P4): the score trajectory over mainline history —
/// cached in the index db, rebuildable from git at will.
pub fn trend_cmd(a: TrendArgs, core: &str) -> ExitCode {
    a.judge.answered("trend", |root, db| {
        codeeraser::trend::judged(root, db, core, a.commits, a.batch)
    })
}

/// `ce structure` (M6 S2): the tree-scale entropy judgment —
/// aggregates to the core's structure/1, dense verdicts re-labelled
/// with local names. Report-only by ruling — no score floor (v2.22, O53).
pub fn structure_cmd(a: StructureArgs, core: &str) -> ExitCode {
    family_checked(
        (a.judge, core),
        "structure",
        move |r, db, c| {
            codeeraser::structure::judge::run(r, db, c, (a.deep, a.days, a.split_candidates))
        },
        codeeraser::report::print_bound,
        |_| None,
    )
}

/// `ce join` (M5-3h): assemble the three signal legs — similarity,
/// graph position, per-unit churn — report-only; the verdict lattice
/// judges them on the verdict/1 wire via `ce check` (M5-3i).
pub fn join_cmd(a: JoinArgs, core: &str) -> ExitCode {
    family_checked(
        (a.judge, core),
        "join",
        move |r, db, c| join::run(r, db, c, a.days),
        codeeraser::report::print_bound,
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
/// raw inter/union plus the core's verdict bits back (ADR-008 P1); the
/// report, its lines and the `--check` veto are the core's (plan v2.32
/// step 5).
pub fn docdup_cmd(a: DocdupArgs, core: &str) -> ExitCode {
    a.judge.answered("docdup", |root, db| {
        docdup::judge::answer(root, db, core, a.check)
    })
}

/// `ce clone` (M5-3e): the T3 TED judgment over the frozen candidate
/// pass — trees to the core in chunks, raw scores plus the core's
/// verdict bits back (ADR-008 P1). `--units` (M5-3b) instead lists
/// the cached unit universe after asserting the unitsig/symbols
/// identity agreement (zero orphans — the nth throat is one
/// function, checked, not assumed). Both documents and their lines
/// are the core's (plan v2.32 step 5).
pub fn clone_cmd(a: CloneArgs, core: &str) -> ExitCode {
    let j = a.judge;
    let root = or_cwd(j.root);
    let answer = if a.units {
        codeeraser::faces::listed_units(&root).and_then(|u| dedup::t3::units_answer(core, &u))
    } else {
        dedup::t3::answer(&root, j.db, core)
    };
    let family = if a.units { "clone-units" } else { "clone" };
    answered("clone", family, json(j.format), answer)
}
