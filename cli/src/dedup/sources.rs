//! What the T3 candidate sources read off this side (design vol.2
//! §4.2): since plan v2.33 W3 the core generates every source
//! (candidates/1, CE.Candidates.Sources) — this module only gathers the
//! one input the core cannot: the S1 near runs, the verified near-miss
//! token runs (25 <= len < 50) the T1/T2 report threshold drops, which
//! the T1/T2 extension finds over the token streams (pairs.rs, the
//! PreToolUse probe's own machinery, stays here).

use super::candidates::Unit;
use super::index::Instance;
use super::{Params, pairs, walkidx};
use std::collections::BTreeMap;
use std::path::Path;

/// Each file's unit ids, in unit order.
pub(crate) fn by_file(units: &[Unit]) -> BTreeMap<&str, Vec<usize>> {
    let mut out: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (id, u) in units.iter().enumerate() {
        out.entry(&u.path).or_default().push(id);
    }
    out
}

/// One near run's two line anchors: (file, start line) each side.
pub(super) type NearRun = [(String, i64); 2];

/// S1's input: the near runs the T1/T2 extension verifies below the
/// report threshold, each as its two (file, start line) anchors.
/// READ-ONLY (M5-close review HIGH-1): the candidate pass must never
/// write the index — its load_streams predecessor silently re-hashed
/// mid-run edits and orphaned their cascade-dropped edges;
/// read_streams makes no claim on drifted files instead.
pub(super) fn near_runs(root: &Path, instances: &[Instance]) -> Vec<NearRun> {
    let p = Params::default();
    let files = pairs::candidate_files(instances);
    let streams = walkidx::read_streams(root, &files);
    let filter = pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: 0,
    };
    let (_, near) = pairs::clone_blocks_near(instances, &streams, filter, p.kgram);
    near.into_iter()
        .map(|run| {
            [
                (run.a_file, run.a_start as i64),
                (run.b_file, run.b_start as i64),
            ]
        })
        .collect()
}
