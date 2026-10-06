//! The `ce docdup` faces as the core lays them out (plan v2.32 step 5;
//! CE.Docdup.Document, CE.Docdup.Lines): this side sends its live
//! segments over a path table (file, span and kind code), every judged
//! pair with the docdup/1 reply's scores, its verbatim run and verdict
//! bit, its counters and `--check`; the document, the console lines and
//! the veto (`--check` with a reported duplication) come back. A
//! segment in the document is a reference the core spells
//! `path:start-end kind` from its row and the path.

use super::Judged;
use crate::document::{self, Answer, Paths, Request};
use anyhow::Result;

/// The docdup report for one judgment, over the link it was judged
/// over when `held` is whole; `check` is the console's `--check`.
pub(super) fn report(core: &str, j: &Judged, check: bool, held: document::Held) -> Result<Answer> {
    let mut paths = Paths::default();
    let segs: Vec<[i64; 5]> = j
        .segs
        .iter()
        .enumerate()
        .map(|(i, s)| {
            [
                i as i64,
                paths.id(&s.path),
                s.start_line,
                s.end_line,
                s.kind,
            ]
        })
        .collect();
    let (c, t) = (&j.counts, &j.counts.tally);
    let req = Request::new("docdup")
        .range("paths", paths.list.len())
        .range("segs", segs.len())
        .fact("check", u8::from(check))
        .counters(
            "over_cap_segments lsh_pairs seed_pairs hot_bands hot_shingles sent requests \
             judged jaccard_dups exempt_license exempt_allow",
            &[
                t.over_cap_segments,
                t.lsh_pairs,
                t.seed_pairs,
                t.hot_bands,
                t.hot_shingles,
                c.sent,
                c.requests as u64,
                c.judged,
                c.jaccard_dups,
                c.exempt_license,
                c.exempt_allow,
            ],
        )
        .rows("pairs", &j.judged)
        .rows("segs", segs);
    document::assemble_over(core, held, req.text("path", &paths.list))
}
