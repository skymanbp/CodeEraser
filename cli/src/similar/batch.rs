//! One refresh's bags in one ask (plan v2.33 W2-text Z4): the walk hands
//! each stale file here; a file with units waits, and the waiting files'
//! unit rows go to the core together — one bags/1 ask per batch, where
//! each file inside its own transaction asked before (750 asks on a cold
//! index of this repository). A batch is asked when the next file would
//! take it past either ceiling the core states (`limits.caps`: the
//! family's item ceiling, counted as its contract counts, and the
//! protocol's line in bytes), and when the walk ends; a file larger than
//! a ceiling goes alone and meets the core's named refusal. The ask
//! happens before any of the batch's transactions begins, and each file
//! is then written whole in walk order (Index::write_file: the old bags
//! retired before the unitsig rows are replaced, the new ones seated
//! after, only net-nonzero terms moving `df`, store.rs) — so a failed ask
//! writes none of its batch, and no file is committed without its bags.
//! A file with no units (a grammarless language, a foreign file) asks
//! nothing but still waits its turn: the files are written in walk order
//! exactly as before, so every row id comes out as it did. A batch also
//! goes when the source it holds would pass the line ceiling (a memory
//! bound: the waiting files keep their bytes until they are written).

use super::bag::{UnitBag, file_rows};
use super::bags::Terms;
use crate::dedup::Params;
use crate::dedup::index::Index;
use crate::scan::lang::Lang;
use anyhow::Result;
use serde_json::Value;

/// What asks the core: a batch's unit rows, in order, to their terms.
pub type Ask = Box<dyn Fn(Vec<Value>) -> Result<Vec<Terms>, String>>;

/// The bytes a request spends beside its unit rows: the envelope's keys,
/// the empty text list and the widest id.
const ENVELOPE: usize = 128;

/// A stale file the walk read, its units' bags still empty.
struct Waiting {
    rel: String,
    src: Vec<u8>,
    lang: Lang,
    foreign: bool,
    bags: Vec<UnitBag>,
}

pub struct Batch {
    p: Params,
    ask: Ask,
    /// The family's item ceiling and the protocol's line in bytes.
    caps: (usize, usize),
    waiting: Vec<Waiting>,
    units: Vec<Value>,
    /// The waiting units' items and bytes, and the source bytes held.
    spent: (usize, usize),
    held: usize,
}

impl Batch {
    /// A batch asking the process's core, by the ceilings it states.
    pub fn new(p: Params) -> Batch {
        let caps = &crate::tables::get().limits.caps;
        let ask: Ask = Box::new(|units| super::bags::ask(units, &[]).map(|(terms, _)| terms));
        Batch::asking(p, (caps.bags_items, caps.line_bytes), ask)
    }

    /// A batch asking through `ask`, by `caps` (items, bytes).
    pub fn asking(p: Params, caps: (usize, usize), ask: Ask) -> Batch {
        Batch {
            p,
            ask,
            caps,
            waiting: Vec::new(),
            units: Vec::new(),
            spent: (0, 0),
            held: 0,
        }
    }

    /// One stale file waits its turn — the batch so far asked and written
    /// first when this file would take it past a ceiling.
    pub fn push(
        &mut self,
        idx: &mut Index,
        (rel, src, lang): (String, Vec<u8>, Lang),
        foreign: bool,
    ) -> Result<()> {
        let rows = if foreign {
            Vec::new()
        } else {
            file_rows(&String::from_utf8_lossy(&src), lang)
        };
        let size = rows.iter().map(|(_, row)| cost(row)).fold((0, 0), add);
        let (items, bytes) = add(self.spent, size);
        let line = self.caps.1;
        let over = items > self.caps.0 || bytes + ENVELOPE > line || self.held + src.len() > line;
        if over && !self.waiting.is_empty() {
            self.finish(idx)?;
        }
        self.spent = add(self.spent, size);
        self.held += src.len();
        let (bags, units): (Vec<UnitBag>, Vec<Value>) = rows.into_iter().unzip();
        self.units.extend(units);
        self.waiting.push(Waiting {
            rel,
            src,
            lang,
            foreign,
            bags,
        });
        Ok(())
    }

    /// Ask for every waiting file's bags at once (nothing to ask when no
    /// waiting file has a unit), then write each file in walk order.
    pub fn finish(&mut self, idx: &mut Index) -> Result<()> {
        let waiting = std::mem::take(&mut self.waiting);
        let units = std::mem::take(&mut self.units);
        (self.spent, self.held) = ((0, 0), 0);
        let sent = units.len();
        let terms = match sent {
            0 => Vec::new(),
            _ => (self.ask)(units).map_err(anyhow::Error::msg)?,
        };
        anyhow::ensure!(
            terms.len() == sent,
            "{}: wire skew: one bag per unit sent",
            super::bags::CAP
        );
        let mut terms = terms.into_iter();
        for mut w in waiting {
            for (bag, t) in w.bags.iter_mut().zip(terms.by_ref()) {
                bag.terms = t;
            }
            idx.write_file(&w.rel, &w.src, w.lang, self.p, w.foreign, &w.bags)?;
        }
        Ok(())
    }
}

/// One unit row's cost: the items the bags/1 contract counts for it (the
/// row, and every element of its callee, literal, structure and doc
/// lists) and its bytes on the line (with the separating comma).
fn cost(row: &Value) -> (usize, usize) {
    let lists = (3..=6)
        .map(|i| row[i].as_array().map_or(0, Vec::len))
        .sum::<usize>();
    (1 + lists, row.to_string().len() + 1)
}

fn add(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    (a.0 + b.0, a.1 + b.1)
}

#[cfg(test)]
#[path = "../../tests/unit/similar/batch.rs"]
mod tests;
