//! The `unmentioned` table's producer (sealed criterion §3.1 domain,
//! §2 veto, §5.1 wire; plan v2.17 L round piece (6)): every judged
//! declaration whose name no other file of the corpus spells, keyed
//! the way the wire wants it — `[node, vis, conv]` — with the names
//! behind each key riding beside it for the renderer and never onto
//! the wire (`request_body` serializes the keys alone; K6).
//!
//! The veto is asked in the safe order, cheapest first, and any yes
//! ends it: another file holds the identity hash; a Rust name past the
//! fold gate has its fold key in another file; the declaring file's
//! own exception regions spell it (selfref). Nothing here reads a
//! visibility bit or a category: which rows the core judges is the
//! core's (`unmentionedVisMask`, `exemptCategories`), so a private
//! declaration must be mentioned exactly like a public one to stay
//! off the table (W2-F2). The domain unit is `(file, name)` across
//! `nth` (H7): same-named declarations of one file fold to one entry
//! at their first line, visibility and category words OR'd — a
//! surface any of them offers, an exemption any of them earns.
//!
//! One read snapshot: the symbols and the mention tables are read in
//! ONE transaction, the `graph_rows` discipline.

use super::conv::name::{PathWords, name_bits, text_bits};
use super::name::mention_name;
use super::selfref::SelfText;
use super::store;
use super::token::{FOLD_MIN_CHARS, fold, segments};
use crate::dedup::index::Index;
use crate::dedup::tokens::fnv1a;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

/// The name payload beside one wire triple: which declaration, where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryName {
    pub symbol: String,
    pub line: i64,
}

/// The table as the wire carries it: key = `[node, vis, conv]`
/// (strictly ascending and deduped by the map), value = the names
/// behind it — non-empty by construction, the only writer being the
/// `entry().or_default().push()` below (W8-F2).
pub type Names = BTreeMap<[i64; 3], Vec<AdvisoryName>>;

/// The producer's own cut of the table, equal to the core's soft cap
/// (`CE.Graph.Cost.unmentionedCap`, pinned source-to-source by
/// docs_consts K46): a smaller value would truncate silently, a
/// larger one would resurrect a table the core drops.
pub const UNMENTIONED_SOFT_CAP: usize = 131_072;

/// One `(file, name)` of the domain before the veto.
struct Decl {
    lang: Lang,
    line: i64,
    vis: i64,
    conv: i64,
}

/// The table for the index's judged declarations, in wire order, cut
/// at the soft cap by whole entries (a key never loses its names).
pub fn unmentioned(root: &Path, idx: &Index, ids: &BTreeMap<(&str, &str), usize>) -> Result<Names> {
    let txn = idx.raw().unchecked_transaction()?;
    let decls = domain(&txn)?;
    let mut words = PathWords::new(root);
    let mut texts: BTreeMap<&str, SelfText> = BTreeMap::new();
    let mut out = Names::new();
    for ((path, name), d) in &decls {
        if store::mentioned_by_other(&txn, fnv1a(name.as_bytes()) as i64, path)? {
            continue;
        }
        if d.lang == Lang::Rust
            && segments(name) >= 2
            && name.chars().count() >= FOLD_MIN_CHARS
            && store::folded_by_other(&txn, fnv1a(fold(name).as_bytes()) as i64, path)?
        {
            continue;
        }
        let file = texts
            .entry(path)
            .or_insert_with(|| SelfText::read(root, path));
        if file.mentions(name) {
            continue;
        }
        let node = ids
            .get(&(path.as_str(), ""))
            .with_context(|| format!("declaring file {path} not a node — index skew"))?;
        let conv = d.conv | words.bits(path) | text_bits(file.text());
        out.entry([*node as i64, d.vis, conv])
            .or_default()
            .push(AdvisoryName {
                symbol: name.clone(),
                line: d.line,
            });
    }
    txn.finish()?; // read-only: closing the snapshot, nothing to write
    Ok(out.into_iter().take(UNMENTIONED_SOFT_CAP).collect())
}

/// Every judged declaration with a mention name, folded to the
/// `(file, name)` unit.
fn domain(conn: &rusqlite::Connection) -> Result<BTreeMap<(String, String), Decl>> {
    let rows: Vec<(String, String, i64, i64, i64)> = crate::graph::load::rows(
        conn,
        "SELECT f.path, s.key, s.start_line, s.flags, s.conv
         FROM symbols s JOIN files f ON f.id = s.file_id
         ORDER BY f.path, s.start_line, s.key",
        crate::graph::load::t5,
    )?;
    let mut decls: BTreeMap<(String, String), Decl> = BTreeMap::new();
    for (path, key, line, vis, conv) in rows {
        let (Some(name), Some(lang)) = (
            mention_name(&path, &key),
            Lang::judged_path(Path::new(&path)),
        ) else {
            continue;
        };
        let word = conv | name_bits(lang, &path, &key, &name);
        let d = decls.entry((path, name)).or_insert(Decl {
            lang,
            line,
            vis: 0,
            conv: 0,
        });
        d.line = d.line.min(line);
        d.vis |= vis;
        d.conv |= word;
    }
    Ok(decls)
}

#[cfg(test)]
#[path = "candidates_tests.rs"]
mod tests;
