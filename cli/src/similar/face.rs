//! The one document of `ce similar`, MCP `similar_units` and the GUI
//! screen (spec §六): the query as resolved, the candidates in the
//! ORDER the core answered with the ROLE bit the core answered, the
//! six-channel evidence row per candidate, and — under `widen` — the
//! associative view: the candidates the PPMI-widened query reaches
//! that the bare query does not, tagged. Report-only and advisory in
//! booklet 13's posture: nothing here is a condition bit or reaches
//! `ce check`. Rust ranks off its own tables and measures; ordering
//! and the same-role conjunction come back over similar/1 (wire.rs);
//! a core that cannot answer makes a NAMED degraded document whose
//! role column is null and whose order is the measured one, unjudged
//! — never a verdict this side reached alone (A9f). The document and
//! the console lines are the core's (document.rs, plan v2.32 step 5).

use super::bm25::{self, Hit, QueryTerm};
use super::query::{self, Ask, Resolved, place};
use super::reader::Reader;
use super::{K, ppmi, wire};
use crate::corelink::Link;
use crate::document::Held;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// One candidate as measured and judged. (The document's field names
/// keep the GUI hub's generic five-column projection — alphabetical
/// scalars `at`, `key`, `nth`, `role`, `score`; CE.Similar.Document.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub at: String,
    pub key: String,
    pub nth: i64,
    /// The core's same-role bit; None on a degraded document.
    pub role: Option<bool>,
    /// The BM25 score's integer part (the frozen eval docs' number).
    pub score: i64,
    /// Distinct spelled terms shared per channel `[N,P,C,D,S,L]`.
    pub hits: [u32; 6],
    pub shape_equal: bool,
    /// Reached by the widened query only (the associative view).
    pub widened: bool,
}

pub struct Report {
    pub label: String,
    pub widen: bool,
    pub terms: usize,
    pub rows: Vec<Row>,
    /// Why the core did not judge, when it did not.
    pub degraded: Option<String>,
}

/// Index refreshed, query resolved, both arms ranked and judged; the
/// link the judgment used, for the document (or why there is none).
pub(super) fn judged(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    ask: &Ask,
    widen: bool,
) -> Result<(Report, Held)> {
    let (idx, _db) = crate::dedup::refreshed_index(root, db)?;
    let reader = Reader::open(&idx)?;
    let q: Resolved = query::resolve(&reader, ask)?;
    let bare = bm25::top_k(&reader, &q.terms, K, q.seat)?;
    let mut judge = Judge::open(core);
    let mut rows = judge.rows(&reader, &q.terms, &bare, false);
    if widen {
        let mut wide_terms = q.terms.clone();
        ppmi::expand(&reader, &mut wide_terms)?;
        let seen: HashSet<usize> = bare.iter().map(|h| h.doc).collect();
        let added: Vec<Hit> = bm25::top_k(&reader, &wide_terms, K, q.seat)?
            .into_iter()
            .filter(|h| !seen.contains(&h.doc))
            .collect();
        rows.extend(judge.rows(&reader, &wide_terms, &added, true));
    }
    let report = Report {
        label: q.label,
        widen,
        terms: q.terms.len(),
        rows,
        degraded: judge.degraded.clone(),
    };
    Ok((report, judge.held()))
}

/// The core's side of the document: a link, or the named reason there
/// is none. The first failure names the whole document degraded; the
/// rows after it keep the measured order and a null role.
struct Judge {
    link: Option<Link>,
    degraded: Option<String>,
}

impl Judge {
    /// The link, when no request failed over it (a failed request may
    /// leave half a reply in the pipe).
    fn held(self) -> Held {
        match (self.link, self.degraded) {
            (Some(link), None) => Ok(link),
            (_, why) => Err(why.unwrap_or_else(|| "core unavailable".into())),
        }
    }

    fn open(core: &str) -> Judge {
        match Link::open(core) {
            Ok((link, _)) => Judge {
                link: Some(link),
                degraded: None,
            },
            Err(why) => Judge {
                link: None,
                degraded: Some(why),
            },
        }
    }

    /// One arm's hits as rows: in the core's order with its role bits,
    /// or — degraded — in the measured order with none.
    fn rows(
        &mut self,
        reader: &Reader<'_>,
        q: &[QueryTerm],
        hits: &[Hit],
        widened: bool,
    ) -> Vec<Row> {
        if hits.is_empty() {
            return Vec::new();
        }
        let judged = match (&mut self.link, &self.degraded) {
            (Some(link), None) => wire::judge(link, q, hits),
            (_, Some(why)) => Err(why.clone()),
            (None, None) => Err("core unavailable".into()),
        };
        let (order, roles): (Vec<usize>, Vec<Option<bool>>) = match judged {
            Ok(j) => (j.order, j.roles.into_iter().map(Some).collect()),
            Err(why) => {
                self.degraded = Some(why);
                ((0..hits.len()).collect(), vec![None; hits.len()])
            }
        };
        order
            .into_iter()
            .map(|i| {
                let h = &hits[i];
                let s = &reader.seats()[h.doc];
                Row {
                    at: place(s),
                    key: s.key.clone(),
                    nth: s.nth,
                    role: roles[i],
                    score: h.score,
                    hits: h.hits,
                    shape_equal: h.shape_equal,
                    widened,
                }
            })
            .collect()
    }
}
