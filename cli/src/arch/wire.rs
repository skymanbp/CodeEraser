//! The arch/1 leg (design booklet §3's wire table, §7): the five
//! request tables out, the six answer tables and the counts back.
//! Nothing here judges — the feedback arc set, the layers, the
//! clusters, the misplaced files, the impact walk and the metrics are
//! the core's (CE.Arch); this side checks the reply is an answer to
//! what it sent before any row is labelled. Three roads are not a
//! judgment and say so by name: a core that cannot be started or
//! answer, a core without the family (never read as an empty
//! answer), and a request past the caps, refused here before the
//! core sees it. A core that degrades a request this side priced
//! inside the caps is a cap-mirror drift, and a skewed reply is
//! refused: both are errors.

use super::tables::Tables;
use crate::corelink::{Link, judged};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The capability the core must offer, the request kind, and the
/// proto that minted the family (for the named absence).
pub const CAP: &str = "arch/1";
pub const KIND: &str = "arch";
const SINCE: &str = "7.6.0";

/// Mirrors of CE.Arch.Cost.fileCap / refCap: the files, and the two
/// reference tables together.
pub const ARCH_FILE_CAP: usize = 131072;
pub const ARCH_REF_CAP: usize = 524288;

/// The counts object's keys, space-separated: the five request
/// tables, then the four answer tallies (read through `count_keys`).
const COUNTS: &str = "files dirs edges pkgEdges focus cuts clusters misplaced impact";

/// The counts object's keys in the wire's order.
pub fn count_keys() -> std::str::SplitWhitespace<'static> {
    COUNTS.split_whitespace()
}

/// The six answer tables and their row widths, `key width` per line:
/// `layers [D, level]`, `cuts [from, to, w, exact]`, `clusters [F,
/// cluster]`, `misplaced [F, majorityDir]`, `impact [F, depth]`,
/// `metrics [D, fanIn, fanOut, instability]`.
const TABLES: &str = "layers 2\ncuts 4\nclusters 2\nmisplaced 2\nimpact 2\nmetrics 4";

/// Which column of which table names a file (`f`) or a directory
/// (`d`), `table column universe` per line — every id the faces
/// subscript with.
const IDS: &str = "cuts 0 d\ncuts 1 d\nclusters 1 f\nmisplaced 0 f\nmisplaced 1 d\nimpact 0 f";

/// The tables with one row per slot, in slot order.
const DENSE: &str = "layers d\nmetrics d\nclusters f";

/// The core's answer, raw: every table ascending by its first column.
pub struct Reply {
    pub tables: BTreeMap<&'static str, Vec<Vec<i64>>>,
    pub counts: BTreeMap<&'static str, i64>,
}

impl Reply {
    /// One answer table's rows (consume checked every key is present).
    pub fn rows(&self, key: &str) -> &[Vec<i64>] {
        self.tables.get(key).map_or(&[], Vec::as_slice)
    }
}

/// An answer, or the named reason there is none.
pub type Judgment = std::result::Result<Reply, String>;

/// The local refusal: a request past either cap never leaves.
pub fn over_cap(t: &Tables) -> Option<String> {
    let refs = t.edges.len() + t.pkg_edges.len();
    (t.files.len() > ARCH_FILE_CAP || refs > ARCH_REF_CAP).then(|| {
        format!(
            "arch_too_large: {} files (cap {ARCH_FILE_CAP}), {refs} references (cap {ARCH_REF_CAP})",
            t.files.len()
        )
    })
}

pub fn request_body(t: &Tables) -> Value {
    json!({
        "files": t.files,
        "dirs": t.dirs,
        "edges": t.edges,
        "pkgEdges": t.pkg_edges,
        "focus": t.focus,
    })
}

/// One arch.request over one link, its reply consumed.
pub fn judge(core: &str, t: &Tables) -> Result<Judgment> {
    if let Some(why) = over_cap(t) {
        return Ok(Err(why));
    }
    let mut link = match Link::open(core) {
        Ok((link, _)) => link,
        Err(why) => return Ok(Err(why)),
    };
    let reply = match judged::ask(&mut link, CAP, SINCE, KIND, request_body(t)) {
        Ok(reply) => reply,
        Err(why) => return Ok(Err(why)),
    };
    crate::lockstep::refuse_degraded(&reply, "arch/wire.rs vs Arch/Cost.hs")?;
    consume(&reply, t).map(Ok)
}

/// The reply, consumed strictly: every table at its width, the counts
/// echoing what was sent and tallying the tables, every id one this
/// side can label, every focus file at depth 0.
pub fn consume(reply: &Value, t: &Tables) -> Result<Reply> {
    let mut tables = BTreeMap::new();
    for (key, width) in TABLES.lines().filter_map(|l| l.split_once(' ')) {
        let rows: Vec<Vec<i64>> = crate::lockstep::reply_rows(reply, key)?;
        let width: usize = width.parse().context("a table width")?;
        ensure!(
            rows.iter().all(|r| r.len() == width),
            "arch reply: a {key} row is not {width} wide"
        );
        tables.insert(key, rows);
    }
    let r = Reply {
        tables,
        counts: counts(reply)?,
    };
    let clusters: BTreeSet<i64> = r.rows("clusters").iter().map(|x| x[1]).collect();
    let tallies = [
        t.files.len(),
        t.dirs.len(),
        t.edges.len(),
        t.pkg_edges.len(),
        t.focus.len(),
        r.rows("cuts").len(),
        clusters.len(),
        r.rows("misplaced").len(),
        r.rows("impact").len(),
    ];
    for (key, n) in count_keys().zip(tallies) {
        ensure!(
            r.counts[key] == n as i64,
            "arch reply: counts.{key} is {} where the tables hold {n}",
            r.counts[key]
        );
    }
    held(&r, t)?;
    Ok(r)
}

fn counts(reply: &Value) -> Result<BTreeMap<&'static str, i64>> {
    count_keys()
        .map(|k| {
            let n = reply["counts"][k].as_i64();
            n.map(|n| (k, n))
                .ok_or_else(|| anyhow::anyhow!("arch reply: counts.{k} missing"))
        })
        .collect()
}

/// Every id in range, the dense tables once per slot in order, the
/// exact flag a bit, the focus at depth 0.
fn held(r: &Reply, t: &Tables) -> Result<()> {
    let n = |universe: &str| match universe {
        "f" => t.files.len() as i64,
        _ => t.dirs.len() as i64,
    };
    for spec in IDS.lines() {
        let [key, col, universe] = spec.split(' ').collect::<Vec<_>>()[..] else {
            continue;
        };
        let col: usize = col.parse().context("an id column")?;
        let bad = r
            .rows(key)
            .iter()
            .find(|x| !(0..n(universe)).contains(&x[col]));
        ensure!(
            bad.is_none(),
            "arch reply: {key} row {bad:?} names an id out of range"
        );
    }
    for spec in DENSE.lines() {
        let Some((key, universe)) = spec.split_once(' ') else {
            continue;
        };
        let firsts: Vec<i64> = r.rows(key).iter().map(|x| x[0]).collect();
        ensure!(
            firsts == (0..n(universe)).collect::<Vec<_>>(),
            "arch reply: {key} must carry one row per slot in order"
        );
    }
    ensure!(
        r.rows("cuts").iter().all(|c| c[3] == 0 || c[3] == 1),
        "arch reply: a cut's exact flag is neither 0 nor 1"
    );
    for f in &t.focus {
        ensure!(
            r.rows("impact").contains(&vec![*f, 0]),
            "arch reply: focus file {f} has no depth-0 impact row"
        );
    }
    Ok(())
}
