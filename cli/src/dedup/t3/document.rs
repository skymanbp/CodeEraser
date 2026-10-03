//! The `ce clone` faces as the core lays them out (plan v2.32 step 5;
//! CE.Clone.Document, CE.Clone.Lines): this side sends every judged
//! pair with the clone/1 reply's scores and verdict bit (the cache's
//! replays among them), its counters as facts and the size of the unit
//! universe; `--units` sends the cached unit universe, one row per unit
//! over a path table. A unit is a reference both ways; the document,
//! the console lines and the (empty) veto come back.

use super::Judged;
use crate::dedup::unitcache::UnitRow;
use crate::document::{self, Answer, Lists, Paths, Request};
use anyhow::Result;

/// The T3 report for one judgment, over the link it was judged over
/// when `held` is whole.
pub(super) fn report(core: &str, j: &Judged, held: document::Held) -> Result<Answer> {
    let c = &j.counts;
    let pairs: Vec<[i64; 6]> = j
        .judged
        .iter()
        .map(|&(a, b, (ted, n1, n2, v))| [a as i64, b as i64, ted, n1, n2, i64::from(v)])
        .collect();
    let req = Request::new("clone")
        .range("units", j.units.len())
        .counters(
            "over_cap_units forest_units survivors s5_windowed s5_pruned_label s5_already \
             s5_new pairs_dropped_over_cap pairs_dropped_forest sent requests prefiltered \
             judged cached",
            &[
                c.over_cap_units as u64,
                c.forest_units as u64,
                c.survivors,
                c.s5_windowed,
                c.s5_pruned_label,
                c.s5_already,
                c.s5_new,
                c.pairs_dropped_over_cap,
                c.pairs_dropped_forest,
                c.sent,
                c.requests as u64,
                c.prefiltered,
                c.judged,
                c.cached,
            ],
        )
        .rows("pairs", pairs);
    // a unit by its `path:key#nth`
    let units = j
        .units
        .iter()
        .map(|u| format!("{}:{}#{}", u.path, u.key, u.nth))
        .collect();
    document::assemble_over(core, held, req, &Lists(vec![("unit", units)]))
}

/// The `--units` listing over the rows the identity check passed.
pub fn units_answer(core: &str, units: &[UnitRow]) -> Result<Answer> {
    let mut paths = Paths::default();
    let rows: Vec<[i64; 4]> = units
        .iter()
        .enumerate()
        .map(|(i, u)| [i as i64, paths.id(&u.path), u.nth, u.nodes])
        .collect();
    let req = Request::new("clone-units")
        .range("paths", paths.list.len())
        .range("units", units.len())
        .rows("units", rows);
    let keys = units.iter().map(|u| u.key.clone()).collect();
    document::assemble(core, req, &Lists(vec![("path", paths.list), ("key", keys)]))
}
