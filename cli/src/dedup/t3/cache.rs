//! The T3 verdict cache (plan v2.30 step 5b-9): the core's clone
//! verdict for a pair of unit trees, remembered in the index under
//! the two trees' content keys, so the faces that read the T3 family
//! since 5b-9 — `ce check` and `ce join` beside `ce clone` — pay the
//! tree edit distance once per tree pair rather than once per run.
//! Every other phase of the judgment is sub-second on this repository
//! (release, warm: candidates 0.6 s, the S5 extension 0.08 s, trees
//! 0.35 s) while the core's judgment over 2,155 pairs is 3.1 s
//! (PERF-BUDGET, 5b-9) — the one cost that scales with the pair count.
//!
//! Replay, not policy (ADR-008): a row is the core's OWN bit for the
//! same two trees under the same proto and knobs, and `reported_clones`
//! reads every row — replayed or fresh — by that bit alone. The
//! key is the judge's whole input for one tree, its postorder kind
//! codes and lld column (tree.rs); two units with equal keys are the
//! same tree to the judge whatever file or language they came from.
//! The generation is (CACHE_REV, the core's proto, the three knobs
//! the reply echoes): a mismatch empties the table before any row is
//! read, so a core that judges differently is announced by its
//! version, never replayed. Rows whose trees left the tree are swept
//! after each judgment, so the table is bounded by the live tree
//! pairs; the parser-revision invalidation and the whole-database
//! wipe both drop it with the other derived tables (schema.rs,
//! schema/parser.rs).

use super::tree::UnitTree;
use super::{Outcome, Scored, ScoredTed};
use crate::dedup::candidates::{PairRow, limits};
use crate::dedup::tokens::fnv1a;
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Bump when a row's meaning changes (the key, the slot order, the
/// NULL reading): it sits in the generation, so old rows are emptied.
const CACHE_REV: i64 = 1;

/// CREATE-only DDL (the DROP half lives in dedup/schema.rs — one wipe
/// lifecycle). `ted` NULL = the core's prefilter proved the pair below
/// threshold: no score row and no bit, remembered as such.
pub const T3TED_SCHEMA: &str = "
CREATE TABLE t3ted (
  ka INTEGER NOT NULL, kb INTEGER NOT NULL,
  na INTEGER NOT NULL, nb INTEGER NOT NULL,
  ted INTEGER, clone INTEGER NOT NULL,
  PRIMARY KEY (ka, kb)) WITHOUT ROWID;
";

/// One remembered outcome in the slot's (ka, kb) order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    /// The core's prefilter proved the pair below threshold.
    Below { na: i64, nb: i64 },
    /// The core scored it: ted, the two sizes, its verdict bit.
    Scored {
        ted: i64,
        na: i64,
        nb: i64,
        clone: bool,
    },
}

pub type Table = HashMap<(i64, i64), Row>;

/// A tree's identity: fnv1a over its postorder kind codes and lld
/// column, the judge's whole input for it (wire.rs sends nothing else
/// per tree — the request-local dense relabelling is a bijection).
pub fn tree_key(t: &UnitTree) -> i64 {
    let mut buf = Vec::with_capacity(16 * t.lab.len() + 8);
    buf.extend_from_slice(b"t3tree/1");
    for (lab, lld) in t.lab.iter().zip(&t.lld) {
        buf.extend_from_slice(&lab.to_le_bytes());
        buf.extend_from_slice(&lld.to_le_bytes());
    }
    fnv1a(&buf) as i64
}

/// Per built unit: (tree key, node count) for a Tree outcome.
pub struct Keys(Vec<Option<(i64, i64)>>);

impl Keys {
    pub(super) fn of(built: &[Outcome]) -> Self {
        Keys(
            built
                .iter()
                .map(|b| match b {
                    Outcome::Tree(t) => Some((tree_key(t), t.lab.len() as i64)),
                    _ => None,
                })
                .collect(),
        )
    }

    /// The slot a pair of built units reads, whether the pair reads it
    /// swapped (ted is symmetric; the sizes are not), and the two
    /// sizes in slot order.
    fn slot(&self, a: usize, b: usize) -> ((i64, i64), bool, (i64, i64)) {
        let (Some((ka, na)), Some((kb, nb))) = (self.0[a], self.0[b]) else {
            unreachable!("sendable pairs reference built trees only")
        };
        if ka <= kb {
            ((ka, kb), false, (na, nb))
        } else {
            ((kb, ka), true, (nb, na))
        }
    }

    /// Every key a built tree carries this run — the sweep's keep set.
    pub fn live(&self) -> BTreeSet<i64> {
        self.0.iter().flatten().map(|&(k, _)| k).collect()
    }
}

/// The generation the table must carry to be read; any other empties
/// it first. The proto is the core's own hello answer, so the bits a
/// different core would judge are never replayed as this one's.
pub fn open_generation(conn: &Connection, proto: &str) -> Result<()> {
    let l = limits();
    let want = fnv1a(
        format!(
            "t3cache/{CACHE_REV}|{proto}|{}/{}|{}",
            l.tsed_num, l.tsed_den, l.min_unit_nodes
        )
        .as_bytes(),
    ) as i64;
    let have: Option<i64> = conn
        .query_row("SELECT v FROM meta WHERE k = 't3_cache'", [], |r| r.get(0))
        .optional()?;
    if have != Some(want) {
        conn.execute_batch("DELETE FROM t3ted")?;
        conn.execute(
            "INSERT INTO meta (k, v) VALUES ('t3_cache', ?1)
             ON CONFLICT(k) DO UPDATE SET v = ?1",
            (want,),
        )?;
    }
    Ok(())
}

/// The whole table — bounded by the live tree pairs (see `remember`).
pub fn load(conn: &Connection) -> Result<Table> {
    let mut stmt = conn.prepare("SELECT ka, kb, na, nb, ted, clone FROM t3ted")?;
    let rows = stmt.query_map([], |r| {
        let (na, nb) = (r.get(2)?, r.get(3)?);
        let row = match r.get::<_, Option<i64>>(4)? {
            None => Row::Below { na, nb },
            Some(ted) => Row::Scored {
                ted,
                na,
                nb,
                clone: r.get::<_, i64>(5)? != 0,
            },
        };
        Ok(((r.get(0)?, r.get(1)?), row))
    })?;
    rows.collect::<rusqlite::Result<_>>().map_err(Into::into)
}

/// Partition the sendable pairs: the ones the table answers become
/// rows now (a Below answer is a pair with no row, as the core would
/// have answered it), the rest go to the wire. Returns (rows, the
/// pairs to send, the replayed count).
pub(super) fn replay<'p>(
    held: &Table,
    keys: &Keys,
    sendable: &[&'p PairRow],
) -> (Scored, Vec<&'p PairRow>, u64) {
    let (mut rows, mut send, mut replayed) = (Vec::new(), Vec::new(), 0);
    for &p in sendable {
        let (slot, swapped, _) = keys.slot(p.a, p.b);
        match held.get(&slot) {
            None => send.push(p),
            Some(Row::Below { .. }) => replayed += 1,
            Some(&Row::Scored { ted, na, nb, clone }) => {
                let (n1, n2) = if swapped { (nb, na) } else { (na, nb) };
                rows.push((p.a, p.b, (ted, n1, n2, clone)));
                replayed += 1;
            }
        }
    }
    (rows, send, replayed)
}

/// This run's fresh outcomes by slot: every scored row with the
/// core's bit, every sent pair the core answered no row for as Below.
/// Two sent pairs over the same two trees share one slot and one
/// outcome, so the map keeps the first.
pub(super) fn fresh(
    keys: &Keys,
    sent: &[&PairRow],
    scored: &[(usize, usize, ScoredTed)],
) -> Vec<((i64, i64), Row)> {
    let answered: HashSet<(usize, usize)> = scored.iter().map(|&(a, b, _)| (a, b)).collect();
    let mut out: Table = HashMap::new();
    for &(a, b, (ted, n1, n2, clone)) in scored {
        let (slot, swapped, _) = keys.slot(a, b);
        let (na, nb) = if swapped { (n2, n1) } else { (n1, n2) };
        out.entry(slot)
            .or_insert(Row::Scored { ted, na, nb, clone });
    }
    for p in sent.iter().filter(|p| !answered.contains(&(p.a, p.b))) {
        let (slot, _, (na, nb)) = keys.slot(p.a, p.b);
        out.entry(slot).or_insert(Row::Below { na, nb });
    }
    out.into_iter().collect()
}

/// Remember the fresh outcomes and sweep the held rows whose trees
/// left the tree — one transaction, so a reader never sees half a run.
pub fn remember(
    conn: &Connection,
    fresh: &[((i64, i64), Row)],
    live: &BTreeSet<i64>,
    held: &Table,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    {
        let mut put = tx.prepare(
            "INSERT OR REPLACE INTO t3ted (ka, kb, na, nb, ted, clone)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for &((ka, kb), row) in fresh {
            let (na, nb, ted, clone) = match row {
                Row::Below { na, nb } => (na, nb, None, false),
                Row::Scored { ted, na, nb, clone } => (na, nb, Some(ted), clone),
            };
            put.execute((ka, kb, na, nb, ted, clone as i64))?;
        }
        let mut del = tx.prepare("DELETE FROM t3ted WHERE ka = ?1 AND kb = ?2")?;
        for (ka, kb) in held.keys() {
            if !live.contains(ka) || !live.contains(kb) {
                del.execute((ka, kb))?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3/cache.rs"]
mod tests;
