//! One answer per report family — the ONE road every face takes to a
//! report document (plan v2.33 step 10): the CLI prints it and exits on
//! its veto, the MCP catalog stringifies its document, the GUI backend
//! returns its document as-is. Lifted when batch 4 was about to copy
//! the MCP adapter bodies into the Tauri commands (the P4 ratchet
//! precedent: cross-surface shells chain into clone blocks); the CLI
//! joined at step 10, when an audit found each `main_*.rs` calling its
//! family's own `face::run` and options the CLI took that the machine
//! faces could not reach. Every option that changes the DOCUMENT is a
//! parameter here and reaches all three faces; what only the CLI sets —
//! the index path, the console's gate, its roast line, one merge group
//! — rides `Plumb`, and every such flag is a row of `exempt::EXEMPT`,
//! the one list the parity gate subtracts. Read-only by construction,
//! same as the MCP charter: no face writes a baseline or config, and
//! the erase face reaches the plan — applying is a closure the CLI
//! hands in, never a call made here. Callers resolve the root and the
//! core.

/// A face that hands its arguments on to one family call: the
/// signature (root, core, the document's options, `Plumb`) is written
/// once here, the plain! / face_cmd! precedent — the delegating faces
/// side by side were this repo's own clone verdict. `$p` comes first in
/// the invocation and last in the signature, as every face takes it.
/// The `checked:` arm mints the faces that take nothing but the gate
/// bit: the family's answer over (root, db, core, check).
macro_rules! road {
    (checked: $($(#[$m:meta])* $name:ident => $answer:path),+ $(,)?) => { $(
        $(#[$m])*
        pub fn $name(root: &Path, core: &str, p: Plumb) -> Result<Answer> {
            $answer(root, p.db, core, p.check)
        }
    )+ };
    ($(#[$m:meta])* $name:ident($root:ident, $core:ident, $p:ident $(, $a:ident: $t:ty)*) => $body:expr) => {
        $(#[$m])*
        pub fn $name($root: &Path, $core: &str, $($a: $t,)* $p: Plumb) -> Result<Answer> {
            $body
        }
    };
}

mod ask;
pub mod exempt;

pub use ask::{PRELUDE_SCHEMA, arch, flow, flow_kinds, merge, pack_text, query, rules, similar};

use crate::document::Answer;
use crate::erase::document::Diffs;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// What only the CLI sets (exempt.rs names each flag and why): the
/// index database (`--db`; machine faces use the project's default),
/// the console's gate (`--check`: a fact the lines and the veto read),
/// the roast line (`--roast`) and one merge group on the console
/// (`--group`). A machine face passes `Plumb::default()`.
#[derive(Default)]
pub struct Plumb {
    pub db: Option<PathBuf>,
    pub check: bool,
    pub roast: bool,
    pub group: Option<usize>,
}

impl Plumb {
    /// The index path alone (the CLI's `--db`, no gate armed).
    pub fn db(db: Option<PathBuf>) -> Self {
        Plumb {
            db,
            ..Plumb::default()
        }
    }

    /// The index path and the console's gate (`--db`, `--check`).
    pub fn gate(db: Option<PathBuf>, check: bool) -> Self {
        Plumb {
            db,
            check,
            ..Plumb::default()
        }
    }
}

/// An answer and the refusal a gate reached after it (dedup's
/// `--check`, erase's `--apply`): the CLI prints the report first and
/// the refusal after it, as it always did; a machine face never arms
/// either and reads the answer alone.
pub type Gated = (Answer, Option<anyhow::Error>);

/// The ONE core resolution of the machine faces (the MCP adapters, the
/// GUI's tasks): corelink's resolver chain — the process's `--core`,
/// CE_CORE_BIN, a `ce-core` sibling of this executable (the installed
/// layout), then PATH. The CLI resolves its own `--core` flag through
/// the same chain.
pub fn core() -> String {
    crate::corelink::resolve_core("ce-core")
}

/// The document of an answer: what every machine face hands out (the
/// console lines and the veto are the CLI's alone).
pub fn doc(answer: Result<Answer>) -> Result<Value> {
    Ok(answer?.document)
}

/// The document of a gated answer (a machine face arms no gate).
pub fn gated_doc(answer: Result<Gated>) -> Result<Value> {
    Ok(answer?.0.document)
}

/// Judged like its siblings (batch-7 slice 8): the scan face reads the
/// core's levels, and the core lays the document out (plan v2.32 step
/// 5); `--format sarif` is the CLI's projection of the same answer.
pub fn scan(root: &Path, core: &str) -> Result<Answer> {
    crate::scan::judged(root, core)
}

/// Both report thresholds ride, not one: `min_distinct` is the
/// DIVERSITY floor (`--min-distinct`), and a face that accepted only
/// `min_tokens` could not reproduce what the CLI prints. `--check` is
/// the CLI's ratchet road (dedup::gated).
pub fn dedup(
    root: &Path,
    core: &str,
    thresholds: (Option<usize>, Option<usize>),
    p: Plumb,
) -> Result<Gated> {
    crate::dedup::gated(root, (p.db, p.check), thresholds, core)
}

/// The churn window, laid out by the core (plan v2.32 step 5).
pub fn churn(root: &Path, core: &str, days: u32) -> Result<Answer> {
    crate::churn::answer(core, &crate::churn::run(root, days)?, days)
}

/// The reference sites, resolution-free (a walk: no index is read).
pub fn graph_sites(root: &Path, core: &str) -> Result<Answer> {
    crate::graph::sites_document(core, &crate::graph::analyze(root)?)
}

road!(checked:
    deadcode => crate::graph::deadcode::answer,
    /// The docdup report, laid out by the core (plan v2.32 step 5).
    docdup => crate::docdup::judge::answer,
);

road!(
    /// The T3 report, or under `units` the cached unit universe (`ce
    /// clone --units`). The identity assertion travels with the listing:
    /// a face that listed units without checking the unitsig/symbols
    /// agreement would hand out a universe nobody had checked. Both
    /// documents are the core's (plan v2.32 step 5).
    clone(root, core, p, units: bool) => match units {
        true => crate::dedup::t3::units_answer(core, &listed_units(root)?),
        false => crate::dedup::t3::answer(root, p.db, core),
    }
);

/// The cached unit universe, the identity agreement checked.
fn listed_units(root: &Path) -> Result<Vec<crate::dedup::unitcache::UnitRow>> {
    let (idx, _db) = crate::dedup::refreshed_index(root, None)?;
    let orphans = crate::dedup::unitcache::identity_orphans(&idx)?;
    anyhow::ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    crate::dedup::unitcache::unit_rows(&idx)
}

road!(join(root, core, p, days: u32) => crate::join::run(root, p.db, core, days));

road!(
    /// `knobs` = (deep, days, split): the S6 axis, the S5 window, the
    /// split-ROI advisory (`--split-candidates`).
    structure(root, core, p, knobs: (bool, Option<u32>, bool)) =>
        crate::structure::judge::run(root, p.db, core, knobs)
);

/// Report-only: this face never writes a baseline (MCP charter ③; `ce
/// baseline` is the CLI's own act). `days` is the churn window (absent
/// = the churn tables stay empty) and `floor` the `--fail-under` floor,
/// opt-in on every road — a face that could not arm it could not
/// reproduce the verdict CI prints, and the report echoes which it
/// judged under. The committed baseline is the face's one read (O31).
pub fn check(
    root: &Path,
    core: &str,
    (days, floor): (Option<u32>, Option<u32>),
    p: Plumb,
) -> Result<Answer> {
    let baseline = crate::score::baseline::read(root)?;
    let mut o = crate::score::run(
        root,
        crate::score::Opts {
            db: p.db,
            core: core.into(),
            days,
            floor,
            establish: false,
            pinned_soft: None,
            baseline,
        },
    )?;
    crate::score::document::document(core, &mut o, p.roast)
}

road!(
    /// `window` = (commits, batch): the mainline window and the most
    /// uncached commits one call measures.
    trend(root, core, p, window: (usize, Option<usize>)) =>
        crate::trend::judged(root, p.db, core, window.0, window.1)
);

/// The GUI's graph screen (plan v2.30 step 5b item 31): the canvas
/// document and the deadcode report off ONE deadcode-family judgment
/// — the assembly lives in graph::canvas. Read-only like every face.
pub fn graph_screen(root: &Path, core: &str) -> Result<Value> {
    crate::graph::canvas::screen(root, core)
}

/// The erase PLAN — dry-run by definition and by construction: this
/// face reaches `erase::planned`, which is read-only. `then` runs
/// between the plan (with its rendered diff) and its document: the
/// CLI's `--apply` and the GUI's apply command hand in the mutation and
/// the answer carries the count it returns; the GUI preview keeps the
/// diff; MCP hands in `|_, _| None`. This module applies nothing of its
/// own — a surface that could delete files on its own authority is the
/// one thing an eraser must never ship. Under `--check` an eraseable row is the
/// veto, and a vetoed plan is never handed on.
pub fn erase(
    root: &Path,
    core: &str,
    p: Plumb,
    then: impl FnOnce(&crate::erase::Plan, &Diffs) -> Option<Result<usize>>,
) -> Result<Gated> {
    use crate::erase::document::{self, Run};
    let (plan, held) = crate::erase::planned(root, p.db, core)?;
    // the diff is rendered before anything is applied: the hash check
    // names a file that moved since planning, and an applied file no
    // longer holds the bytes the plan showed
    let diffs = Diffs::of(root, &plan)?;
    let vetoed = p.check && plan.counts.eraseable > 0;
    let mut run = Run {
        check: p.check,
        applied: None,
    };
    let refused = match (!vetoed).then(|| then(&plan, &diffs)).flatten() {
        Some(Ok(n)) => {
            run.applied = Some(n);
            None
        }
        Some(Err(e)) => Some(e),
        None => None,
    };
    Ok((document::answer(core, held, &plan, &diffs, run)?, refused))
}

/// The GUI's erase preview: the `erase` face's plan document and,
/// under `diff`, the unified diff it rendered over the SAME plan — one
/// measurement, so the preview can never show bytes the plan did not
/// hash (erase.md).
pub fn erase_preview(root: &Path, core: &str) -> Result<Value> {
    let mut diff = String::new();
    let keep = |_: &crate::erase::Plan, d: &Diffs| {
        diff = d.unified();
        None
    };
    let mut doc = gated_doc(erase(root, core, Plumb::default(), keep))?;
    doc["diff"] = Value::String(diff);
    Ok(doc)
}

/// The applied-erase audit trail (plan v2.29 step 9, O50): apply.rs is
/// its one writer and this is its reader — every face renders the same
/// document, and none can append to it; a line the reader cannot parse
/// rides inside the document by line number.
pub fn erase_log(root: &Path, core: &str) -> Result<Answer> {
    let log = crate::erase::log::read(root)?;
    crate::erase::document::trail_answer(core, &log)
}

/// The machine's own state and the tier of every guard class the hooks
/// read (the write guard, flow, tombstone). Unlike every sibling it
/// cannot fail: a core that will not answer IS the finding, and it
/// rides inside the document (health::doctor).
pub fn doctor(root: &Path, core: &str) -> Value {
    crate::health::doctor::document(root, core)
}

/// Whether a newer release exists — the update DOCUMENT. Takes no root
/// and no core: the question is about THIS binary. Read-only like
/// every face; the apply leg (`update::apply::run`) is a human act at
/// the CLI or the GUI, the same line the erase plan draws.
pub fn update_check() -> Value {
    crate::update::document()
}
