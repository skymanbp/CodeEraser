//! Judgment-family subcommand bodies (`ce clone`, `ce docdup`,
//! `ce trend`, `ce structure`, `ce join`, `ce flow`, `ce arch`) — split from main_cmds.rs
//! when the docdup family pushed it over the repo's own 300-line
//! gate, with the shared flag set factored the same day the ratchet
//! caught DocdupArgs re-growing CloneArgs field-for-field; the other
//! families moved in as they grew the same JudgeArgs shape.

use crate::main_cmds::{OutFormat, json, or_cwd};
use crate::main_prelude::{answered, degraded, whole};
use codeeraser::churn;
use codeeraser::faces::{self, Plumb};
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
    /// exit reads the core's veto (main_prelude::answered) — the one body
    /// of `ce docdup`, `ce trend`, `ce structure`, `ce join` and `ce
    /// similar`.
    pub(crate) fn answered(
        self,
        family: &str,
        ask: impl FnOnce(&Path, Option<PathBuf>) -> anyhow::Result<codeeraser::document::Answer>,
    ) -> ExitCode {
        let as_json = json(self.format);
        answered(
            family,
            family,
            as_json,
            ask(&or_cwd(self.root), self.db),
            whole,
        )
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
    #[arg(long, default_value_t = churn::DEFAULT_DAYS)]
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

/// The commands whose whole body is one faces call through
/// `JudgeArgs::answered`: the fn scaffold is written once (the
/// faces::road! / face_cmd! precedent — four hand-written twins side by
/// side were this repo's own clone verdict). Each row names the args
/// binding, the core and the closure's (root, db) the face is asked by.
macro_rules! answered_cmd {
    ($($(#[$m:meta])* $name:ident($a:ident: $args:ty, $core:ident)
        => $family:literal, |$root:ident, $db:ident| $face:expr;)+) => { $(
        $(#[$m])*
        pub fn $name($a: $args, $core: &str) -> ExitCode {
            $a.judge.answered($family, |$root, $db| $face)
        }
    )+ };
}

answered_cmd! {
    /// `ce trend` (M7-P4): the score trajectory over mainline history —
    /// cached in the index db, rebuildable from git at will.
    trend_cmd(a: TrendArgs, core) => "trend",
        |root, db| faces::trend(root, core, (a.commits, a.batch), Plumb::db(db));

    /// `ce structure` (M6 S2): the tree-scale entropy judgment —
    /// aggregates to the core's structure/1, dense verdicts re-labelled
    /// with local names. Report-only by ruling — no score floor (v2.22,
    /// O53).
    structure_cmd(a: StructureArgs, core) => "structure",
        |root, db| faces::structure(root, core, (a.deep, a.days, a.split_candidates), Plumb::db(db));

    /// `ce join` (M5-3h): assemble the three signal legs — similarity,
    /// graph position, per-unit churn — report-only; the verdict lattice
    /// judges them on the verdict/1 wire via `ce check` (M5-3i).
    join_cmd(a: JoinArgs, core) => "join",
        |root, db| faces::join(root, core, a.days, Plumb::db(db));

    /// `ce docdup` (M5-3g): the documentation-duplication judgment over
    /// the live cached segments — shingle sets to the core in chunks,
    /// raw inter/union plus the core's verdict bits back (ADR-008 P1);
    /// the report, its lines and the `--check` veto are the core's (plan
    /// v2.32 step 5).
    docdup_cmd(a: DocdupArgs, core) => "docdup",
        |root, db| faces::docdup(root, core, Plumb::gate(db, a.check));
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
    let answer = faces::clone(&or_cwd(j.root), core, a.units, Plumb::db(j.db));
    let family = if a.units { "clone-units" } else { "clone" };
    answered("clone", family, json(j.format), answer, whole)
}

#[derive(clap::Args)]
pub struct FlowArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Exit 1 when `[flow] tier` is deny and a judged finding stands
    /// (unused parameters are advisory and never count)
    #[arg(long)]
    check: bool,
    /// Show only these kinds, comma-separated: a kind the core's flow
    /// catalogue lists; an unknown name is refused with the list. The
    /// counts stay whole
    #[arg(long, value_delimiter = ',')]
    kind: Vec<String>,
}

/// `ce flow` — the flow family's face (plan v2.31 step 5, design
/// booklet §5.4): the dead code inside every function of the tree,
/// judged by the core's flow/1. Report-only unless `--check`: then it
/// is its own gate, never `ce check`'s — exit 1 when `[flow] tier` is
/// deny and a judged finding stands (an unused parameter is advisory
/// everywhere and never counts). Exit 2 = a core that could not judge,
/// a malformed reply, a document the core did not lay out, an unknown
/// `--kind`, an unreadable ce.toml.
pub fn flow_cmd(a: FlowArgs, core: &str) -> ExitCode {
    let plumb = Plumb::gate(None, a.check);
    let answer = faces::flow(&or_cwd(a.judge.root), core, &a.kind, plumb);
    answered("flow", "flow", json(a.judge.format), answer, degraded)
}

#[derive(clap::Args)]
pub struct ArchArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Trace the files that depend on this one (root-relative;
    /// repeatable)
    #[arg(long = "impact")]
    impact: Vec<PathBuf>,
}

/// `ce arch` — the architecture family's face (plan v2.31 step 9,
/// design booklet §7.3): the layers, the cuts, the clusters, the
/// misplaced files and the directory metrics of the tree, and under
/// `--impact` the files a change to the named ones reaches. Advisory:
/// exit 0 with a document, 2 when the core could not judge (the
/// document names why), could not lay the document out, or a named
/// path is not a measured file. The `--impact` paths go as typed:
/// arch::face::run reads them as the index spells them, on every face
/// (plan v2.33 step 10).
pub fn arch_cmd(a: ArchArgs, core: &str) -> ExitCode {
    let j = a.judge;
    let focus: Vec<String> = a.impact.iter().map(|p| p.display().to_string()).collect();
    let answer = faces::arch(&or_cwd(j.root), core, &focus, Plumb::db(j.db));
    answered("arch", "arch", json(j.format), answer, degraded)
}
