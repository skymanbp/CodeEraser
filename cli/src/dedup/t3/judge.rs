//! The cache-backed judgment over the sendable pairs (plan v2.30 step
//! 5b-9), split out of mod.rs at the 300-line soft line: the verdict
//! cache answers the pairs it holds for this core's proto and knobs
//! (cache.rs), the rest ride the ONE lockstep machine over one link,
//! and what the core answered is remembered for the next run. A
//! replayed scored row still passes the core's decision (clone/1
//! `decide`): a cached bit the core would not give is a corrupt cache
//! row, refused by name.

use super::{Outcome, Scored, cache, wire};
use crate::dedup::candidates::PairRow;
use crate::lockstep;
use anyhow::Result;

/// Over the link the candidate pass opened: returns the merged rows,
/// `[judged, prefiltered, cached]`, the request count and the link,
/// whole, for the report.
pub(super) fn judge(
    mut link: crate::corelink::Link,
    core: &str,
    conn: &rusqlite::Connection,
    built: &[Outcome],
    sendable: &[&PairRow],
) -> Result<(Scored, [u64; 3], usize, crate::corelink::Link)> {
    cache::open_generation(conn, link.proto())?;
    let held = cache::load(conn)?;
    let keys = cache::Keys::of(built);
    let (mut rows, send, cached) = cache::replay(&held, &keys, sendable);
    decided(&mut link, &rows)?;
    let (scored, judged, prefiltered, requests) = lockstep::lockstep_scores(
        &mut link,
        &wire::family(core),
        &send,
        |chunk| {
            let ab: Vec<(usize, usize)> = chunk.iter().map(|p| (p.a, p.b)).collect();
            wire::chunk_request(&ab, |g| match &built[g] {
                Outcome::Tree(t) => t,
                _ => unreachable!("sendable pairs reference built trees only"),
            })
        },
        wire::parse_result,
    )?;
    cache::remember(
        conn,
        &cache::fresh(&keys, &send, &scored),
        &keys.live(),
        &held,
    )?;
    rows.extend(scored);
    rows.sort_unstable();
    Ok((rows, [judged, prefiltered, cached], requests, link))
}

/// Every replayed row's cached bit against the core's own decision.
fn decided(link: &mut crate::corelink::Link, rows: &Scored) -> Result<()> {
    let metrics: Vec<[i64; 3]> = rows.iter().map(|r| [r.2.0, r.2.1, r.2.2]).collect();
    let bits = wire::decide(link, &metrics)?;
    for (&(_, _, (ted, n1, n2, v)), core) in rows.iter().zip(bits) {
        anyhow::ensure!(
            v == core,
            "the verdict cache's bit ({v}) disagrees with the core's decision at ted {ted} nodes {n1}/{n2} — a corrupt t3ted row (delete .ce/index.db to rebuild)"
        );
    }
    Ok(())
}
