//! The persisted bags (spec §三 倒排表, index schema v16): two tables
//! holding only fnv1a64 term hashes and counts — `bag`, one row per
//! unit term (the posting), keyed by the unit's own `unitsig` row so
//! the bag universe IS the unitsig universe by foreign key; and `df`,
//! one row per term with the units carrying it and, for a word, the
//! units counting it inside the PPMI cap (the pair count's marginal).
//! Pairs themselves are NOT stored: a pair table for this repository
//! held 688k rows, grew the index 5.4× and the cold index 7–10× (A/B
//! in the step-3 record), and the association view that reads pairs
//! is opt-in — so reader.rs derives a word's co-occurrence counts at
//! query time from the bag rows of the units that carry it, exactly
//! the counts the in-memory table keeps. Written inside refresh_file's
//! content-hash-gated transaction in two halves around the unitsig
//! refresh (whose row replacement cascades the old bag rows away):
//! `retire` tallies the file's old bags at −1 first, `refresh_bags`
//! tallies the new ones at +1 after, and only the non-zero NET deltas
//! reach SQL — an edit to one function costs that unit's terms, not
//! the file's and never the corpus's (a unit the edit did not touch
//! cancels itself out). A foreign file (owner = 1, measured by nobody)
//! writes no rows; flipping owner retires its rows through the same
//! gate, and a file leaving the tree retires them before its `files`
//! row cascades. The DROP half lives in dedup/schema.rs (one wipe
//! lifecycle); `SIMILAR_REV` sits in the cache key, so any change to
//! the term road wipes the tables with the rest. Privacy (plan
//! §5.9.2): no word text enters the database.

use super::bag::{UnitBag, file_bags};
use super::ppmi::capped_words;
use super::terms::Channel;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction};
use std::collections::{BTreeMap, HashMap};

/// CREATE-only DDL. The bag table IS its posting list: a WITHOUT
/// ROWID table keyed (term_hash, unit), so a term's units are one
/// b-tree range and the cold index writes two b-trees per row, not
/// three (measured on the self tree: 8.2 s → see the step-3 record).
/// `idx_bag_unit` is the cascade child's FK index (the schema-v6
/// lesson) and the unit lookup the reader's shape / bag / pair reads
/// take. The counts
/// refuse to go negative by constraint — a drift between the rows and
/// their aggregate fails by name instead of ranking on a wrong idf —
/// and a marginal never exceeds its df.
pub const SIMILAR_SCHEMA: &str = "
CREATE TABLE bag (
  term_hash INTEGER NOT NULL,
  unit INTEGER NOT NULL REFERENCES unitsig(id) ON DELETE CASCADE,
  tf INTEGER NOT NULL, channel INTEGER NOT NULL,
  PRIMARY KEY (term_hash, unit)) WITHOUT ROWID;
CREATE INDEX idx_bag_unit ON bag(unit);
CREATE TABLE df (term_hash INTEGER PRIMARY KEY,
  df INTEGER NOT NULL CHECK (df >= 0),
  marg INTEGER NOT NULL CHECK (marg >= 0 AND marg <= df)) WITHOUT ROWID;
";

/// Net movement of the aggregate for one file refresh: per term, the
/// change in units carrying it and in units counting it inside the
/// PPMI cap. Opened by `retire` before the unitsig rows are replaced,
/// closed by `refresh_bags` after.
#[derive(Default)]
pub struct Delta {
    df: BTreeMap<u64, (i64, i64)>,
}

/// The first half: the file's stored bags come out of the aggregate
/// (their rows leave with the unitsig cascade a moment later).
pub fn retire(tx: &Transaction<'_>, file_id: i64) -> Result<Delta> {
    let mut delta = Delta::default();
    for (_, old) in bags_where(tx, "u.file_id = ?1", (file_id,))? {
        delta.tally(&old, -1);
    }
    Ok(delta)
}

/// The second half, once the file's unitsig rows are current: bag
/// every unit, seat each bag on its unitsig row, and move the
/// aggregate by the net difference.
pub fn refresh_bags(
    tx: &Transaction<'_>,
    file_id: i64,
    text: &str,
    lang: Lang,
    foreign: bool,
    mut delta: Delta,
) -> Result<()> {
    let bags = if foreign {
        Vec::new()
    } else {
        file_bags(text, lang)
    };
    let seats: HashMap<(String, i64), i64> = tx
        .prepare_cached("SELECT key, nth, id FROM unitsig WHERE file_id = ?1")?
        .query_map((file_id,), |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut ins = tx
        .prepare_cached("INSERT INTO bag (unit, term_hash, tf, channel) VALUES (?1, ?2, ?3, ?4)")?;
    for bag in &bags {
        let unit = seats
            .get(&(bag.key.clone(), bag.nth))
            .with_context(|| format!("{}#{}: bagged but not in unitsig", bag.key, bag.nth))?;
        for (term, (ch, tf)) in &bag.terms {
            ins.execute((unit, *term as i64, tf, ch.index() as i64))?;
        }
        delta.tally(bag, 1);
    }
    delta.apply(tx)
}

/// A file leaving the index: its rows come out of the aggregate
/// before the `files` cascade drops them (the cascade alone would
/// leave df counting a ghost).
pub fn retire_file(tx: &Transaction<'_>, file_id: i64) -> Result<()> {
    retire(tx, file_id)?.apply(tx)
}

/// The stored bags matching `cond` (a WHERE clause over `b` = bag and
/// `u` = its unitsig row), grouped back into units with their unitsig
/// id, in unit order. Spans are not stored here (the reader takes
/// them from `symbols`), so they read 0.
pub(super) fn bags_where(
    conn: &Connection,
    cond: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<(i64, UnitBag)>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT b.unit, u.key, u.nth, b.term_hash, b.tf, b.channel
         FROM bag b JOIN unitsig u ON u.id = b.unit
         WHERE {cond} ORDER BY b.unit, b.term_hash"
    ))?;
    let rows = stmt.query_map(params, |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)? as u64,
            r.get::<_, u32>(4)?,
            r.get::<_, i64>(5)?,
        ))
    })?;
    let mut out: Vec<(i64, UnitBag)> = Vec::new();
    for row in rows {
        let (unit, key, nth, term, tf, channel) = row?;
        let channel = *usize::try_from(channel)
            .ok()
            .and_then(|i| Channel::ALL.get(i))
            .ok_or_else(|| anyhow::anyhow!("bag row with channel {channel}: not a channel"))?;
        if out.last().is_none_or(|(u, _)| *u != unit) {
            out.push((unit, UnitBag::empty(key, nth)));
        }
        let (_, bag) = out.last_mut().expect("just pushed");
        bag.terms.insert(term, (channel, tf));
    }
    Ok(out)
}

impl Delta {
    /// One unit's contribution at `sign`: every distinct term to df;
    /// every capped word to the marginal as well.
    fn tally(&mut self, bag: &UnitBag, sign: i64) {
        for term in bag.terms.keys() {
            self.df.entry(*term).or_insert((0, 0)).0 += sign;
        }
        let (words, _) = capped_words(bag);
        for word in words {
            self.df.entry(word).or_insert((0, 0)).1 += sign;
        }
    }

    /// One move per changed term: an update where the row exists, an
    /// insert where it does not (SQLite checks a CHECK on the inserted
    /// values BEFORE resolving an upsert's conflict, so a falling move
    /// cannot ride an upsert). A move that would take a row negative,
    /// or retire a row that is not there, fails the CHECK by name — the
    /// cache drifted — and a row that fell to zero is swept.
    fn apply(&self, tx: &Transaction<'_>) -> Result<()> {
        let mut update =
            tx.prepare_cached("UPDATE df SET df = df + ?2, marg = marg + ?3 WHERE term_hash = ?1")?;
        let mut insert =
            tx.prepare_cached("INSERT INTO df (term_hash, df, marg) VALUES (?1, ?2, ?3)")?;
        let mut sweep = tx.prepare_cached("DELETE FROM df WHERE term_hash = ?1 AND df = 0")?;
        for (term, (d_df, d_marg)) in &self.df {
            if (*d_df, *d_marg) == (0, 0) {
                continue;
            }
            let moved = (*term as i64, d_df, d_marg);
            if update.execute(moved)? == 0 {
                insert.execute(moved)?;
            }
            if *d_df < 0 {
                sweep.execute((*term as i64,))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/similar/store.rs"]
mod tests;
