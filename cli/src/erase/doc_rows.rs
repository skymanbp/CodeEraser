//! The erase plan's verbatim-doc candidates (split from gather.rs at the
//! 300-line wall, release 1.10.0): of each core-reported pair, which
//! side is the candidate and the facts its row carries. Nothing here
//! decides safety; the core's erase/1 predicate does.

use super::gather::TextCache;
use super::model::{Candidate, Source};
use super::section::{self, Links};
use crate::docdup::judge::Doc;
use crate::docdup::judge::candidates::SegRow;
use anyhow::Result;

/// Verbatim doc duplicates: for every core-reported pair, the side
/// whose section FEWER links land on is the candidate (release 1.10.0,
/// erase/section.rs: a linked section is referenced), and on equal
/// counts the path-lexicographically later occurrence, as before
/// (erase.md: the survivor is chosen deterministically, never judged
/// "better"). The candidate's count rides to the core, which refuses a
/// linked candidate as `linked_section`.
pub(super) fn doc_candidates(
    segs: &[SegRow],
    dups: &[(usize, usize, Doc)],
    links: &Links,
    cache: &mut TextCache,
) -> Result<Vec<Candidate>> {
    let mut out = Vec::new();
    for (a, b, m) in dups {
        let key = |i: usize| (segs[i].path.as_str(), segs[i].start_line);
        let mut linked = |i: usize| -> Result<i64> {
            let s = &segs[i];
            Ok(section::links_to(
                links,
                &s.path,
                cache.text(&s.path)?,
                s.start_line,
            ))
        };
        let (la, lb) = (linked(*a)?, linked(*b)?);
        let a_first = key(*a) <= key(*b);
        let (keep, gone, n) = match la.cmp(&lb) {
            std::cmp::Ordering::Less => (*b, *a, la),
            std::cmp::Ordering::Greater => (*a, *b, lb),
            _ if a_first => (*a, *b, lb),
            _ => (*b, *a, la),
        };
        out.push(doc_candidate(&segs[keep], &segs[gone], m, n, cache)?);
    }
    Ok(out)
}

/// One verbatim doc candidate: `t` erased in favour of `s`, its
/// section linked from `linked` places.
fn doc_candidate(
    s: &SegRow,
    t: &SegRow,
    m: &Doc,
    linked: i64,
    cache: &mut TextCache,
) -> Result<Candidate> {
    let eq = cache.slice(&s.path, s.start_line, s.end_line)?
        == cache.slice(&t.path, t.start_line, t.end_line)?;
    let mut provenance = format!(
        "docdup: twin of {}:{}-{} (J {}/{}, verbatim {} words)",
        s.path, s.start_line, s.end_line, m.inter, m.union, m.verbatim
    );
    if linked > 0 {
        provenance += &format!("; section linked from {linked} places");
    }
    Ok(Candidate {
        class: 1,
        facts: [m.verbatim as i64, s.words, t.words, i64::from(eq)],
        path: t.path.clone(),
        span: Some((t.start_line, t.end_line)),
        provenance,
        source: Source::Doc {
            twin: s.path.clone(),
            span: (s.start_line, s.end_line),
            jaccard: (m.inter as i64, m.union as i64),
            verbatim: m.verbatim as i64,
            linked,
        },
        // a doc twin's licence is byte equality, never liveness
        sites: 0,
    })
}
