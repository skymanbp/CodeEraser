//! Reading the persisted bags back (spec §三): the seats of the own
//! unit universe — every unitsig row of an own file in (path, key,
//! nth) order, its span from `symbols`, its length summed off `bag` —
//! then postings from `bag`, df and the pair marginal from `df`, and
//! a word's co-occurrence counts derived from the bag rows of the
//! units that carry it (store.rs: no pair table), through the
//! `Postings` / `Cooc` traits bm25.rs and ppmi.rs rank against. The
//! in-memory `Corpus` the instruments build and this reader therefore
//! run ONE ranking road, and the replay asserts the two agree on every
//! unit of five corpora. Hits name seats; a seat names a unit and its
//! lines, never a term.

use super::bag::UnitBag;
use super::bm25::{Doc, Postings};
use super::ppmi::{Cooc, TERM_CAP};
use super::store::bags_where;
use super::terms::Channel;
use crate::dedup::index::Index;
use crate::graph::load;
use anyhow::{Context, Result, ensure};
use rusqlite::Connection;
use std::collections::HashMap;

/// One own unit as the index knows it.
pub struct Seat {
    pub path: String,
    /// Its `unitsig` row — the key every bag row carries.
    pub unit: i64,
    pub key: String,
    pub nth: i64,
    pub start_line: i64,
    pub end_line: i64,
    /// Σ tf of its bag — the BM25 document length; 0 for a unit with
    /// no terms at all.
    pub len: u32,
}

pub struct Reader<'c> {
    conn: &'c Connection,
    seats: Vec<Seat>,
    by_unit: HashMap<i64, usize>,
    /// Mean bag length, floored, never below one — fixed at open.
    avg_len: i128,
}

const SEATS: &str = "SELECT f.path, u.id, u.key, u.nth, s.start_line, s.end_line
    FROM unitsig u JOIN files f ON f.id = u.file_id
    LEFT JOIN symbols s ON s.file_id = u.file_id AND s.key = u.key AND s.nth = u.nth
    WHERE f.owner = 0 ORDER BY f.path, u.key, u.nth";

const LENS: &str = "SELECT unit, SUM(tf) FROM bag GROUP BY unit";

const WORD_CHANNELS: [Channel; 3] = [Channel::Name, Channel::Callee, Channel::Doc];

impl<'c> Reader<'c> {
    /// Open over an index: the seats and their lengths are read once;
    /// a unitsig row without its symbols twin, or a bag unit outside
    /// the own universe, is a corrupt cache named here rather than a
    /// candidate with no lines.
    pub fn open(idx: &'c Index) -> Result<Reader<'c>> {
        let conn = idx.raw();
        let mut seats = load::rows(conn, SEATS, |r| {
            let span: (Option<i64>, Option<i64>) = (r.get(4)?, r.get(5)?);
            Ok((
                Seat {
                    path: r.get(0)?,
                    unit: r.get(1)?,
                    key: r.get(2)?,
                    nth: r.get(3)?,
                    start_line: span.0.unwrap_or(0),
                    end_line: span.1.unwrap_or(0),
                    len: 0,
                },
                span.0.is_some(),
            ))
        })?
        .into_iter()
        .map(|(s, spanned)| {
            ensure!(
                spanned,
                "{}:{} #{}: unitsig row with no symbols row",
                s.path,
                s.key,
                s.nth
            );
            Ok(s)
        })
        .collect::<Result<Vec<Seat>>>()?;
        let by_unit: HashMap<i64, usize> =
            seats.iter().enumerate().map(|(i, s)| (s.unit, i)).collect();
        let mut total_len = 0u64;
        for (unit, len) in load::rows(conn, LENS, |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, u32>(1)?))
        })? {
            let seat = by_unit.get(&unit).with_context(|| {
                format!("bag rows for unitsig row {unit} outside the own universe")
            })?;
            seats[*seat].len = len;
            total_len += u64::from(len);
        }
        let avg_len = (total_len / seats.len().max(1) as u64).max(1) as i128;
        Ok(Reader {
            conn,
            seats,
            by_unit,
            avg_len,
        })
    }

    /// Two INTEGER columns of one parameterised query.
    fn two_columns(&self, sql: &str, params: impl rusqlite::Params) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map(params, |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    /// The seat holding `(path, key, nth)`, if the unit is indexed.
    pub fn seat_of(&self, path: &str, key: &str, nth: i64) -> Option<usize> {
        self.seats
            .iter()
            .position(|s| s.path == path && s.key == key && s.nth == nth)
    }

    fn seat(&self, unit: i64) -> Result<usize> {
        self.by_unit
            .get(&unit)
            .copied()
            .with_context(|| format!("bag rows for unitsig row {unit} outside the own universe"))
    }

    /// One column of a term's `df` row, 0 for a term the corpus lacks.
    fn count(&self, term: u64, column: &str) -> Result<i64> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!("SELECT {column} FROM df WHERE term_hash = ?1"))?;
        let n: Option<i64> = stmt
            .query_row((term as i64,), |r| r.get(0))
            .map(Some)
            .or_else(crate::dedup::schema::ignore_no_rows)?;
        Ok(n.unwrap_or(0))
    }

    /// One seat's stored bag, spanned from the seat.
    pub fn bag(&self, seat: usize) -> Result<UnitBag> {
        let s = &self.seats[seat];
        let mut bags = bags_where(self.conn, "b.unit = ?1", (s.unit,))?;
        let mut bag = bags
            .pop()
            .map_or_else(|| UnitBag::empty(s.key.clone(), s.nth), |(_, b)| b);
        (bag.start_line, bag.end_line) = (s.start_line as usize, s.end_line as usize);
        Ok(bag)
    }

    /// Every seat's stored bag in seat order — the corpus as the
    /// instruments build it in memory, read off the tables instead.
    pub fn docs(&self) -> Result<Vec<Doc>> {
        let mut docs: Vec<Doc> = self
            .seats
            .iter()
            .map(|s| {
                let mut bag = UnitBag::empty(s.key.clone(), s.nth);
                (bag.start_line, bag.end_line) = (s.start_line as usize, s.end_line as usize);
                Doc {
                    path: s.path.clone(),
                    bag,
                }
            })
            .collect();
        for (unit, bag) in bags_where(self.conn, "1", [])? {
            docs[self.seat(unit)?].bag.terms = bag.terms;
        }
        Ok(docs)
    }
}

impl Postings for Reader<'_> {
    fn n_docs(&self) -> usize {
        self.seats.len()
    }

    fn avg_len(&self) -> i128 {
        self.avg_len
    }

    fn df(&self, term: u64) -> Result<usize> {
        Ok(self.count(term, "df")? as usize)
    }

    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>> {
        self.two_columns(
            "SELECT unit, tf FROM bag WHERE term_hash = ?1",
            (term as i64,),
        )?
        .into_iter()
        .map(|(unit, tf)| Ok((self.seat(unit)?, u32::try_from(tf)?)))
        .collect()
    }

    fn len(&self, seat: usize) -> u32 {
        self.seats[seat].len
    }

    fn shape(&self, seat: usize) -> Result<Vec<u64>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT term_hash FROM bag WHERE unit = ?1 AND channel = ?2 ORDER BY term_hash",
        )?;
        let rows = stmt.query_map(
            (self.seats[seat].unit, Channel::Shape.index() as i64),
            |r| r.get::<_, i64>(0).map(|t| t as u64),
        )?;
        let mut shape: Vec<u64> = rows.collect::<rusqlite::Result<_>>()?;
        shape.sort_unstable();
        Ok(shape)
    }

    fn identity(&self, seat: usize) -> (&str, &str, i64) {
        let s = &self.seats[seat];
        (&s.path, &s.key, s.nth)
    }
}

impl Cooc for Reader<'_> {
    /// The word terms of every unit carrying `a`, grouped per unit,
    /// capped the way the pair count caps them (in u64 term order —
    /// SQL's INTEGER order is i64's, so the cut is taken here), and
    /// counted: a unit counts a pair only when both words sit inside
    /// its cap, so `a` beyond the cap counts nothing, as in the
    /// in-memory table. Cost is the bag rows of the units carrying
    /// `a`, and ppmi::neighbours never asks for a word in more than a
    /// quarter of the units.
    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>> {
        let ch = WORD_CHANNELS.map(|c| c.index() as i64);
        let rows = self.two_columns(
            "SELECT b.unit, b.term_hash FROM bag a JOIN bag b ON b.unit = a.unit
             WHERE a.term_hash = ?1 AND b.channel IN (?2, ?3, ?4) ORDER BY b.unit",
            (a as i64, ch[0], ch[1], ch[2]),
        )?;
        let mut units: Vec<(i64, Vec<u64>)> = Vec::new();
        for (unit, term) in rows {
            match units.last_mut() {
                Some((u, words)) if *u == unit => words.push(term as u64),
                _ => units.push((unit, vec![term as u64])),
            }
        }
        let mut n: HashMap<u64, u32> = HashMap::new();
        for (_, mut words) in units {
            words.sort_unstable();
            words.truncate(TERM_CAP);
            if !words.contains(&a) {
                continue;
            }
            for b in words.into_iter().filter(|b| *b != a) {
                *n.entry(b).or_insert(0) += 1;
            }
        }
        let mut out: Vec<(u64, u32)> = n.into_iter().collect();
        out.sort_unstable();
        Ok(out)
    }

    fn n_units(&self) -> u32 {
        self.seats.len() as u32
    }

    fn n_term(&self, a: u64) -> Result<u32> {
        Ok(self.count(a, "marg")? as u32)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/similar/reader.rs"]
mod tests;
