//! `ce dedup` clone-detection hot path (plan ADR-005): normalized
//! token stream → winnowing/Rabin-Karp fingerprints (Schleimer et al.
//! SIGMOD'03) → SQLite inverted index → clone blocks. T1/T2 only
//! here; T3 is the M5 cold path.

mod budget;
pub mod candidates;
pub mod groups;
pub mod index;
pub mod minhash;
pub mod pairs;
pub mod probe;
mod report;
mod rescache;
pub(crate) mod schema;
pub(crate) mod sources;
pub mod struct_fp;
pub mod t3;
pub mod tokens;
pub mod unitcache;
mod walkidx;
pub mod winnow;

use crate::config::Config;
use crate::graph::{ladder, store, wire};
use crate::scan::Format;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// CLI options for [`run`] (bundled: six loose params would trip the
/// project's own params threshold).
pub struct RunOpts {
    pub format: Format,
    pub db: Option<PathBuf>,
    pub min_tokens: Option<usize>,
    pub min_distinct: Option<usize>,
    pub check: bool,
    /// ce-core path: the budget comparison under `check` (ADR-008 P2)
    /// and the report itself (plan v2.32 step 5) are the core's.
    pub core: String,
}

/// Batch entry point: refresh the index (incremental), reap deleted
/// files, verify anchors by token-stream extension, report blocks.
/// `min_tokens` lowers the report filter below the guarantee t for
/// calibration runs; detection below t is opportunistic (anchors are
/// only guaranteed at >= t). `check` turns the run into the R12
/// ratchet gate against ce.toml [dedup] budget.
pub fn run(root: &Path, opts: RunOpts) -> Result<ExitCode> {
    if opts.check {
        // the ratchet road accepts the operating point or tighter only
        budget::gate_filters(opts.min_tokens, opts.min_distinct)?;
    }
    let (found, summary) = analyze(root, opts.db, opts.min_tokens, opts.min_distinct)?;
    // the R12 gate lives in budget.rs (split at the 300-line dogfood
    // ceiling when the review-repair asserts landed); since 2.19.0 it
    // ships the pre-filter distinct counts and the effective floor, and
    // the CORE's derivation is the gated number (batch-7 slice 1). A
    // gate that could not judge still lets the report print first —
    // its refusal is the run's error after it, as before the switch.
    let gate = opts.check.then(|| {
        budget::check(
            root,
            found.blocks.len(),
            &found.distincts,
            opts.min_distinct,
            &opts.core,
        )
    });
    let judged = gate.as_ref().and_then(|g| g.as_ref().ok().copied());
    let answer = report::answer(&opts.core, &found, &summary, judged)?;
    report::emit(opts.format, &answer)?;
    if let Some(Err(err)) = gate {
        return Err(err);
    }
    Ok(ExitCode::from(u8::from(answer.fail)))
}

/// One command boundary's measurement: blocks + their open index + its path (P10).
pub type Snapshot = (pairs::Blocks, index::Index, PathBuf);

/// Measure ONCE and thread values — each multi-family leg re-ran the whole
/// walk before (structure --deep paid it three times, the erase gather four).
pub fn snapshot(root: &Path, db: Option<PathBuf>) -> Result<Snapshot> {
    let (found, _summary) = analyze(root, db.clone(), None, None)?;
    let db_path = index_db_path(root, db);
    let idx = index::Index::open(&db_path, Params::default())?;
    Ok((found, idx, db_path))
}

/// Refresh the index for `root` and open it — the one opening every
/// cached-graph consumer (deadcode, `ce clone --units`) walks; the
/// path rides back for error messages that name the database.
pub fn refreshed_index(root: &Path, db: Option<PathBuf>) -> Result<(index::Index, PathBuf)> {
    let (_found, idx, db_path) = snapshot(root, db)?;
    Ok((idx, db_path))
}

/// The ONE spelling of the default index location (a third caller —
/// trend — would have grown a third copy of the join).
///
/// The default hangs off the project ANCHOR above `root`, so a run
/// pointed at a subdirectory shares the project's one index instead of
/// minting a second `.ce/` there. Five such strays existed in this
/// very repository (core/, cli/, gui/src-tauri/, and two deeper),
/// invisible because the .gitignore rule was unanchored too. An
/// explicit `--db` still wins verbatim — that is the escape hatch for
/// a deliberately separate index.
pub(crate) fn index_db_path(root: &Path, db: Option<PathBuf>) -> PathBuf {
    db.unwrap_or_else(|| crate::root::project_root(root).join(".ce/index.db"))
}

/// The walkidx read + index.rs text conversion, verbatim — ONE decode
/// for every judgment-side re-read (t3 trees, docdup sequences): a
/// different decode here would judge text the cache never saw.
///
/// A file that VANISHES between the index refresh and this re-read
/// aborts the judgment BY NAME, deliberately not the walk's skip
/// rule: at index time a vanished file simply is not live, but here
/// its cached rows already seeded candidates, and silently dropping
/// them would report "no duplication" for a file this run never
/// examined — the silent-clean class this product forbids. The named
/// abort costs a re-run, which converges; the window is the
/// microseconds after the same command's own refresh.
pub fn walked_text(root: &Path, path: &str) -> Result<(String, crate::scan::lang::Lang)> {
    use anyhow::Context;
    let full = root.join(path);
    let bytes = crate::scan::walk::read_surviving(&full)?
        .with_context(|| format!("{path}: vanished mid-judgment — re-run to converge"))?;
    let lang = crate::scan::lang::Lang::from_path(Path::new(path))
        .with_context(|| format!("{path}: no lang"))?;
    Ok((String::from_utf8_lossy(&bytes).into_owned(), lang))
}

/// Library entry shared by the CLI and the daemon: index, verify,
/// and return the blocks + summary without printing anything.
pub fn analyze(
    root: &Path,
    db: Option<PathBuf>,
    min_tokens: Option<usize>,
    min_distinct: Option<usize>,
) -> Result<(pairs::Blocks, Summary)> {
    let config = Config::load(root).map_err(anyhow::Error::msg)?;
    let db_path = index_db_path(root, db);
    let p = Params::default();
    let mut idx = index::Index::open(&db_path, p)?;
    let seen = idx.indexed_paths()?;
    let walked = walkidx::index_all(root, &config, &mut idx)?;
    // the keep set is every indexed path — live files and the prose-
    // only arm (plan v2.30 step 5b-8) — not `live` alone
    let removed = idx.remove_missing(&walked.indexed(), &seen)?;
    let filter = pairs::Filter {
        min_tokens: min_tokens.unwrap_or(p.guarantee()),
        min_distinct: min_distinct.unwrap_or(pairs::DEFAULT_MIN_DISTINCT),
    };
    // Result cache (rescache.rs): with no indexed content moved, the
    // reload + matching phases re-derive byte-identical blocks at
    // ~56% of the warm cost (measured; PERF-BUDGET.md). On a hit only
    // the edge sweep still runs — a resolve_key can shift with no
    // content change — and the full-build mark is already set over
    // this very content by the run that stored the slot.
    let digest = rescache::digest(idx.raw())?;
    let found = match rescache::load(idx.raw(), digest, filter)? {
        Some(found) => {
            resolve_edges(&mut idx, root, &walked, &BTreeSet::new())?;
            found
        }
        None => recompute(&mut idx, root, &walked, p, filter)?,
    };
    let summary = summarize(&walked, removed, filter);
    Ok((found, summary))
}

/// The miss path: instances + stream reload, edge sweep, full-build
/// mark and block verification — then the slot is left describing
/// THIS run's result, the invariant the hit path's freshness proof
/// (digest match => same indexed content) rests on.
fn recompute(
    idx: &mut index::Index,
    root: &Path,
    walked: &walkidx::WalkIndex,
    p: Params,
    filter: pairs::Filter,
) -> Result<pairs::Blocks> {
    let mut instances = idx.all_instances()?;
    let streams = walkidx::load_streams(root, &pairs::candidate_files(&instances), idx, p)?;
    if !streams.1.is_empty() {
        // a file changed between refresh and stream load; the streams
        // were re-fed into the index, so re-fetch the instances to
        // keep offsets and streams consistent (attack-review D1)
        instances = idx.all_instances()?;
    }
    resolve_edges(idx, root, walked, &streams.1)?;
    // completeness as a POSITIVE cross-process fact for coldstart —
    // this line only runs after every table of a FULL pass committed
    // (clearance review: a row count read a concurrent writer's
    // partial index as complete)
    idx.mark_full_build()?;
    let found = pairs::clone_blocks(&instances, &streams.0, filter);
    // the digest is re-read: the D1 re-feed above can move file rows
    // mid-run, and the slot must describe the state the blocks came from
    rescache::store(idx.raw(), rescache::digest(idx.raw())?, filter, &found)?;
    Ok(found)
}

/// Per-run summary: block facts ride `found`, refresh facts ride
/// THIS run's walk — a served cache result must not replay the
/// storing run's refreshed/removed counters.
fn summarize(walked: &walkidx::WalkIndex, removed: usize, filter: pairs::Filter) -> Summary {
    Summary {
        // has_tokens files only: Markdown rows are graph cache, and
        // counting them would silently change dedup-report/0.5.0
        files: walked.tokenized,
        refreshed: walked.dirty.len(),
        removed,
        min_tokens: filter.min_tokens,
        min_distinct: filter.min_distinct,
    }
}

/// Phases 2 and 1.5 (design §3), AFTER every refresh of the run so
/// the sweep covers all current sites: the ladder resolver rides the
/// key gate — a stable key touches nothing, a shifted key sweeps
/// every edge. Without a sweep, content refreshes (walk or stream
/// reload) kept the key but cascade-dropped their files' edges; the
/// per-file pass restores exactly those.
fn resolve_edges(
    idx: &mut index::Index,
    root: &Path,
    walked: &walkidx::WalkIndex,
    stream_dirty: &BTreeSet<String>,
) -> Result<()> {
    // one memo per sweep call: the quiescent-tree window the caches
    // are sound for (ladder::Memo)
    let memo = ladder::Memo::default();
    let scope = ladder::Scope {
        files: &walked.live,
        assets: &walked.assets,
        configs: &walked.configs,
        root,
        memo: &memo,
        crate_roots: &walked.crate_roots,
        search_roots: &walked.search_roots,
        java: &walked.java,
        lua: &walked.lua,
        includes: &walked.includes,
    };
    let mut resolver = |s: &store::CachedSite| wire::edges(s, &scope);
    if idx.ensure_edges_resolved(walked.resolve_key, &mut resolver)? {
        return Ok(());
    }
    let mut dirty = walked.dirty.clone();
    dirty.extend(stream_dirty.iter().cloned());
    idx.resolve_refreshed(&dirty, &mut resolver)
}

pub use report::{answer, report_json};

/// One run's walk counters and the two report filters it used — the
/// facts the dedup document reads beside the blocks (the block, group
/// and stale counts ride `found`; the winnowing point is the core's).
pub struct Summary {
    pub files: usize,
    pub refreshed: usize,
    pub removed: usize,
    pub min_tokens: usize,
    pub min_distinct: usize,
}

/// Winnowing parameters. Guarantee threshold t = matches of at least
/// `t` normalized tokens are always detected (SIGMOD'03 correctness
/// bound); noise threshold k = matches shorter than `kgram` tokens are
/// never reported. window = t - k + 1.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub kgram: usize,
    pub window: usize,
}

impl Params {
    /// Winnowing guarantee threshold t: every match of at least this
    /// many tokens shares a fingerprint — also the report threshold.
    pub fn guarantee(&self) -> usize {
        self.window + self.kgram - 1
    }
}

impl Default for Params {
    /// t = 50 tokens aligns with the jscpd min-tokens default
    /// (plan §4.1 clone row); k = 25 → window 26.
    fn default() -> Self {
        Self {
            kgram: 25,
            window: 26,
        }
    }
}
