//! The cache-backed judgment over the sendable pairs (plan v2.30 step
//! 5b-9), split out of mod.rs at the 300-line soft line: the verdict
//! cache answers the pairs it holds for this core's proto and knobs
//! (cache.rs), the rest ride the ONE lockstep machine over one link,
//! and what the core answered is remembered for the next run.

use super::{Outcome, Scored, cache, wire};
use crate::dedup::candidates::PairRow;
use crate::lockstep;
use anyhow::Result;

/// Returns the merged rows, `[judged, prefiltered, cached]` and the
/// request count.
pub(super) fn judge(
    core: &str,
    conn: &rusqlite::Connection,
    built: &[Outcome],
    sendable: &[&PairRow],
) -> Result<(Scored, [u64; 3], usize)> {
    let mut link = lockstep::open_family(core, wire::CAP)?;
    cache::open_generation(conn, link.proto())?;
    let held = cache::load(conn)?;
    let keys = cache::Keys::of(built);
    let (mut rows, send, cached) = cache::replay(&held, &keys, sendable);
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
    Ok((rows, [judged, prefiltered, cached], requests))
}
