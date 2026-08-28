//! The `mentioned` veto's corpus pass (plan v2.17 L round piece (3);
//! sealed criterion §1 universe, §2 tokens, §5.1 storage). The veto
//! itself is a NEGATIVE instrument: only when a declaration's name
//! occurs in no other file of the corpus may anything say "no static
//! reference" — so every rule here leans toward counting a mention,
//! and the danger runs one way only (a referenced name read as
//! unmentioned). This module builds the corpus side: which files are
//! looked at (walk.rs), what counts as a token (token.rs), and the two
//! tables that hold the hashes (store.rs) — and the declaration side's
//! two readings (piece (4)): the name the veto judges (name.rs, §3.1)
//! and the AST half of the category word (conv/, §3.2), the latter
//! measured in `fourclass::units` and so inside `dedup::analyze`'s
//! per-file refresh, where the declaration node is already in hand.
//!
//! The corpus pass has its OWN entry — `refresh` below — and is not
//! part of `dedup::analyze`: that function is the guard/audit hot path
//! with a 1.5 s budget, and an unconditional read-and-tokenize of the
//! whole tree would breach it. Only the consumers that carry the
//! advisory (deadcode and its GUI/MCP faces) call it; the judged files
//! are therefore read twice per such run, a measured and accepted cost.
//!
//! Consistency is per file, never per run: file M read at t₀ without
//! `foo`, `foo` added at t₁, declaring file D read at t₂ ⇒ this run
//! reports a false unmentioned and the next converges. Stated, not
//! painted over.

pub mod candidates;
mod census;
pub mod conv;
pub mod face;
pub mod name;
pub mod selfref;
pub mod store;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_caps;
#[cfg(test)]
mod tests_index;
mod token;
mod walk;

pub use candidates::{AdvisoryName, Names, UNMENTIONED_SOFT_CAP};
pub use census::Outside;

use crate::dedup::index::Index;
use crate::dedup::tokens::fnv1a;
use anyhow::Result;
use rusqlite::{Transaction, TransactionBehavior};
use std::collections::BTreeSet;
use std::path::Path;

/// Bump when ANY frozen input of the pass changes — the stored rows
/// are then re-derived (a mismatch empties the two mention tables and
/// nothing else). The inputs, each a sentence in its file:
///   - token.rs: the run alphabet (opens / continues), the script
///     split's identifier side, the `$` arm and its extension table
///     (`MENTION_WHOLE_RUN_EXTS`, lower-cased lookup, no extension =
///     union), the digit-led drop, the fold filter set and the
///     7-literal-character fold threshold, and the fold gate's
///     segmenter (`_`/camel boundaries, an all-caps run one segment)
///     with its 2-segment declaration-side threshold — frozen with
///     the rev (spec §5.1) although it gates the declaration side
///     and fills no stored row;
///   - walk.rs: every walker parameter, the nested-repository cut,
///     the file-symlink rule, the 4 MiB cap, the exclusion table
///     (shared secret globs + omni-mentioners), the binary rule;
///   - this file: the per-file distinct-token cap and the table cap.
///
/// 1 = the pass as sealed (spec v9).
pub const MENTION_REV: i64 = 1;

/// Distinct tokens one file may store; the rest are clipped and
/// counted. The table cap bounds the whole database (a 500 MB tree
/// would otherwise mean ~13 M rows and minutes of inserts): this
/// instrument's declared corpus scale is ≤ 50 MB of text.
pub const FILE_TOKEN_CAP: usize = 65_536;
pub const TABLE_ROW_CAP: usize = 4_194_304;

/// Files per commit. Atomicity stays per file (a batch commits whole,
/// so no file is ever half-written), the write lock is held for at
/// most this many files, and the per-commit cost — the whole price of
/// the first cut, 17.8 s cold on the self corpus with one fsync'd
/// commit per file — is paid once per batch (the measured numbers
/// live in docs/PERF-BUDGET.md).
const BATCH_FILES: usize = 32;

/// The two caps as a value, so a leg can shrink them to a handful.
#[derive(Clone, Copy)]
struct Caps {
    file: usize,
    table: usize,
}

impl Default for Caps {
    fn default() -> Self {
        Caps {
            file: FILE_TOKEN_CAP,
            table: TABLE_ROW_CAP,
        }
    }
}

/// The pass header (K41): what the universe was and what was left out
/// of it, so a generated mirror or a skipped binary is visible rather
/// than silently absent. Every field names its scope: the tree, the
/// store, or this run.
#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
    pub universe: usize,
    pub sources: usize,
    pub rows: usize,
    /// Files holding the per-file cap of rows — the store's standing
    /// clip, reported every run (a file with exactly that many distinct
    /// tokens counts too: the store cannot tell the two apart).
    pub capped: usize,
    /// `name$N` runs in the `dist/*.js` members of U — a tree fact,
    /// recounted every run whether or not the file changed (K41).
    pub dist_js_dedup_runs: usize,
    pub skipped: Skipped,
    pub run: Run,
    pub outside: Outside,
}

/// What the walk left out, each cause counted where it is decided.
#[derive(Debug, Default, serde::Serialize)]
pub struct Skipped {
    pub oversize: usize,
    pub binary: usize,
    pub walk_errors: usize,
}

/// This run's own movement: the convergence facts (K40) and the two
/// cap counters, which are deltas of this run's writes — a starved
/// file is retried every run, so starvation stays visible until it
/// clears.
#[derive(Debug, Default, serde::Serialize)]
pub struct Run {
    pub refreshed: usize,
    pub removed: usize,
    pub rescanned: bool,
    pub clipped: usize,
    pub starved: usize,
}

/// Bring the mention tables of `idx` up to date with the tree under
/// `root`: walk U, refresh every changed file under its own hash gate,
/// reap what vanished, and report the header.
pub fn refresh(root: &Path, idx: &Index) -> Result<Stats> {
    refresh_under(root, idx, Caps::default())
}

fn refresh_under(root: &Path, idx: &Index, caps: Caps) -> Result<Stats> {
    let conn = idx.raw();
    let (tuned, rescanned) = store::prepare(conn, MENTION_REV)?;
    let mut pass = Pass {
        caps,
        ..Pass::default()
    };
    pass.stats.run.rescanned = rescanned;
    let seen = store::indexed_paths(conn)?;
    let universe = walk::universe(root)?;
    pass.stats.skipped.oversize = universe.oversize;
    pass.stats.skipped.walk_errors = universe.errors;
    // what the walk no longer lists is reaped FIRST, so a file the
    // table cap starved last run finds the freed room this run
    let listed: BTreeSet<String> = universe.files.iter().map(|(_, rel)| rel.clone()).collect();
    pass.stats.run.removed = store::prune(conn, &listed, &seen)?;
    (pass.rows, _) = store::totals(conn)?;
    let mut live = BTreeSet::new();
    for batch in universe.files.chunks(BATCH_FILES) {
        let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
        for (path, rel) in batch {
            if pass.file(&tx, path, rel)? {
                live.insert(rel.clone());
            }
        }
        tx.commit()?;
    }
    pass.stats.universe = live.len();
    // listed but unreadable or binary this run: its rows go too
    pass.stats.run.removed += store::prune(conn, &live, &seen)?;
    store::finish(&tuned)?;
    (pass.stats.rows, pass.stats.sources) = store::totals(conn)?;
    pass.stats.capped = store::capped(conn, caps.file)?;
    pass.stats.outside = census::outside(root, conn, &live)?;
    Ok(pass.stats)
}

/// One pass in flight: the caps, the header, and the table's row
/// total — one COUNT per run (it scans the whole index, so once per
/// batch would not scale), carried forward by this pass's own writes.
/// A foreign writer can only make the snapshot LOW, which the table
/// cap then overshoots by that writer's rows (benign); the one
/// dangerous shape — a file's rows exceeding the snapshot — is
/// re-read under the batch's lock in `write`, so the arithmetic never
/// underflows.
#[derive(Default)]
struct Pass {
    caps: Caps,
    rows: usize,
    stats: Stats,
}

impl Pass {
    /// One file of U — unreadable (counted), unchanged under its own
    /// hash gate, binary (counted), or refreshed. True when the file
    /// is live text this run. A `dist/*.js` member is decoded even
    /// when unchanged: its bundler-suffix witness is a tree fact.
    fn file(&mut self, tx: &Transaction<'_>, path: &Path, rel: &str) -> Result<bool> {
        let Ok(bytes) = std::fs::read(path) else {
            self.stats.skipped.walk_errors += 1;
            return Ok(false);
        };
        let hash = fnv1a(&bytes) as i64;
        let unchanged = store::stored_hash(tx, rel)? == Some(hash);
        let witness = token::whole_run_only(rel) && rel.split('/').any(|seg| seg == "dist");
        if unchanged && !witness {
            return Ok(true);
        }
        let Some(text) = walk::decode(&bytes) else {
            self.stats.skipped.binary += 1;
            return Ok(false);
        };
        if witness {
            self.stats.dist_js_dedup_runs += token::runs(&text)
                .filter(|r| token::dedup_suffixed(r))
                .count();
        }
        if unchanged {
            return Ok(true);
        }
        self.write(tx, rel, hash, &text)
    }

    /// Tokens de-duplicated and written under the file's own hash. The
    /// per-file cap is a function of the bytes, so its clip is final and
    /// the hash is stored; the table cap is a function of the whole
    /// store, so a starved file gets neither rows nor hash and the next
    /// run retries it once room exists.
    fn write(&mut self, tx: &Transaction<'_>, rel: &str, hash: i64, text: &str) -> Result<bool> {
        let mut distinct: BTreeSet<&str> = BTreeSet::new();
        token::emit(text, token::whole_run_only(rel), &mut |t| {
            distinct.insert(t);
        });
        let before = store::rows_of(tx, rel)?;
        if before > self.rows {
            // a foreign writer grew this file past the snapshot: the
            // truth is re-read under this batch's own lock
            (self.rows, _) = store::totals(tx)?;
        }
        let want = distinct.len().min(self.caps.file);
        if self.rows - before + want > self.caps.table {
            self.stats.run.clipped += distinct.len();
            self.stats.run.starved += 1;
            return Ok(true);
        }
        let kept: Vec<store::Row> = distinct.iter().take(want).map(|t| row_of(t)).collect();
        self.stats.run.clipped += distinct.len() - want;
        store::replace_file(tx, rel, hash, &kept)?;
        self.rows = self.rows - before + want;
        self.stats.run.refreshed += 1;
        Ok(true)
    }
}

fn row_of(token: &str) -> store::Row {
    let folded = (token.chars().count() >= token::FOLD_MIN_CHARS)
        .then(|| fnv1a(token::fold(token).as_bytes()) as i64);
    store::Row {
        ident: fnv1a(token.as_bytes()) as i64,
        folded,
    }
}
