//! The `ce similar` document as the core lays it out (plan v2.32 step
//! 5; CE.Similar.Document, CE.Similar.Lines): this side sends every
//! candidate in display order — its seat, ordinal, BM25 score, the six
//! channel hits, the shape bit, the arm that reached it and the core's
//! role bit (2 where the core did not judge) — the query's term count,
//! the switch and the bags' revision, over the link the judgment used.
//! The query's label, each seat's place and key, and the reason a
//! judgment did not happen are this side's strings. Advisory: no veto.

use super::SIMILAR_REV;
use super::face::{self, Report};
use super::query::Ask;
use crate::document::{self, Answer, Held, Request, Resolve, Why};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Index refreshed, query resolved, both arms ranked and judged, the
/// document laid out.
pub fn answer(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    ask: &Ask,
    widen: bool,
) -> Result<Answer> {
    let (report, held) = face::judged(root, db, core, ask, widen)?;
    laid_out(core, &report, held)
}

fn laid_out(core: &str, r: &Report, held: Held) -> Result<Answer> {
    let cands: Vec<[i64; 12]> = (r.rows.iter().enumerate())
        .map(|(seat, x)| {
            let [n, p, c, d, s, l] = x.hits.map(i64::from);
            let role = x.role.map_or(2, i64::from);
            let (shape, wide) = (i64::from(x.shape_equal), i64::from(x.widened));
            [
                seat as i64,
                x.nth,
                x.score,
                n,
                p,
                c,
                d,
                s,
                l,
                shape,
                wide,
                role,
            ]
        })
        .collect();
    let mut why = Why::default();
    let mut req = Request::new("similar")
        .range("seats", cands.len())
        .fact("terms", r.terms)
        .fact("widen", u8::from(r.widen))
        .fact("similarRev", SIMILAR_REV)
        .rows("candidates", cands);
    if let Some(reason) = &r.degraded {
        req = req.degraded(why.add(reason.clone()));
    }
    let strings = Strings {
        label: r.label.clone(),
        at: r.rows.iter().map(|x| x.at.clone()).collect(),
        key: r.rows.iter().map(|x| x.key.clone()).collect(),
        why,
    };
    document::assemble_over(core, held, req.range("why", strings.why.count()), &strings)
}

/// The query's label, each seat's place and key, the reason texts.
struct Strings {
    label: String,
    at: Vec<String>,
    key: Vec<String>,
    why: Why,
}

impl Resolve for Strings {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("label", []) => Some(self.label.clone()),
            ("at", _) => document::at(&self.at, ints),
            ("key", _) => document::at(&self.key, ints),
            ("why", _) => self.why.at(ints),
            _ => None,
        }
    }
}
