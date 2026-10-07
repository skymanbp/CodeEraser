//! The arch/1 leg (design booklet §3's wire table, §7): the path form
//! out — since plan v2.33 W1 item 3 the core builds the tree and the five
//! tables from the paths (CE.Arch.Tables) — the five tables it built, each
//! directory's path, the six answer tables and the counts back. Nothing
//! here judges — the feedback arc set, the layers, the clusters, the
//! misplaced files, the impact walk and the metrics are the core's
//! (CE.Arch); this side checks the reply is an answer to what it sent
//! before any row is labelled. Three roads are not a judgment and say so
//! by name: a core that cannot be started or answer, a core without the
//! family (never read as an empty answer), and a request past the caps,
//! which the core degrades and this side names from the tables it built.
//! A focus path naming no measured file is the core's fault, printed as
//! the error it is, and a skewed reply is refused.

use crate::corelink::judged;
use crate::document::Held;
use crate::structure::rows::Arcs;
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The capability the core must offer, the request kind, and the
/// proto that minted the family (for the named absence).
pub const CAP: &str = "arch/1";
pub const KIND: &str = "arch";
const SINCE: &str = "7.6.0";

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

/// What the request carries by path: the measured files and their arcs
/// (structure::rows), each file's line count in the same order, and the
/// `--impact` paths as given.
pub struct Ask {
    pub arcs: Arcs,
    pub lines: Vec<i64>,
    pub focus: Vec<String>,
}

/// The four row tables the core built from the paths and their row
/// widths, `key width` per line (design booklet §7.1): `files [F, D,
/// lines]` with F the path-order slot, `dirs [D, parent]` with the root
/// row 0 at parent −1, `edges [F, G, w]` with F ≠ G and w the arcs
/// between them, `pkgEdges [F, D, w]` F's references to the package at
/// directory D.
pub const BUILT: &str = "files 3\ndirs 2\nedges 3\npkgEdges 3";

/// What the core built from the paths: the BUILT tables, each ascending
/// as the wire demands, the `--impact` files (ascending, distinct) and
/// each directory's path (the root `""`). Empty when no core answered.
#[derive(Default)]
pub struct Built {
    pub tables: BTreeMap<&'static str, Vec<Vec<i64>>>,
    pub focus: Vec<i64>,
    pub dir_paths: Vec<String>,
}

impl Built {
    /// One built table's rows (`built` read every BUILT key).
    pub fn rows(&self, key: &str) -> &[Vec<i64>] {
        self.tables.get(key).map_or(&[], Vec::as_slice)
    }
}

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

pub fn request_body(a: &Ask) -> Value {
    json!({
        "paths": a.arcs.paths,
        "lines": a.lines,
        "arcs": a.arcs.files,
        "sections": a.arcs.sections,
        "packages": a.arcs.packages,
        "focusPaths": a.focus,
    })
}

/// One arch.request over the face's link, the tables the core built and
/// its reply consumed; a request that failed spends the link
/// (crate::document::Held).
pub fn judge(held: &mut Held, a: &Ask) -> Result<(Built, Judgment)> {
    let link = match held {
        Ok(link) => link,
        Err(why) => return Ok((Built::default(), Err(why.clone()))),
    };
    let reply = match judged::ask(link, CAP, SINCE, KIND, request_body(a)) {
        Ok(reply) => reply,
        Err(why) => {
            *held = Err(why.clone());
            return Ok((Built::default(), Err(why)));
        }
    };
    if let Some(fault) = reply["fault"].as_str() {
        bail!("{fault}");
    }
    let built = built(&reply, a)?;
    if reply["degraded"] == Value::Bool(true) {
        let why = why_too_large(&built);
        return Ok((built, Err(why)));
    }
    let r = consume(&reply, &built)?;
    Ok((built, Ok(r)))
}

/// The tables the core built, and that they are the request's: one file
/// row per path, one path per directory row.
fn built(reply: &Value, a: &Ask) -> Result<Built> {
    let tables = &reply["tables"];
    ensure!(
        tables.is_object(),
        "arch reply carries no built tables — a core without the path form (9.0.0)"
    );
    let b = Built {
        tables: read_tables(tables, BUILT)?,
        focus: crate::lockstep::reply_rows(tables, "focus")?,
        dir_paths: crate::lockstep::reply_rows(reply, "dirNames")?,
    };
    let (files, dirs) = (b.rows("files").len(), b.rows("dirs").len());
    ensure!(
        files == a.arcs.paths.len() && b.dir_paths.len() == dirs,
        "arch reply: {files} file rows for {} paths, {} names for {dirs} directories",
        a.arcs.paths.len(),
        b.dir_paths.len(),
    );
    Ok(b)
}

/// The tables of a `key width` spec read off a reply object, every row
/// at its width.
fn read_tables(v: &Value, spec: &'static str) -> Result<BTreeMap<&'static str, Vec<Vec<i64>>>> {
    let mut tables = BTreeMap::new();
    for (key, width) in spec.lines().filter_map(|l| l.split_once(' ')) {
        let rows: Vec<Vec<i64>> = crate::lockstep::reply_rows(v, key)?;
        let width: usize = width.parse().context("a table width")?;
        ensure!(
            rows.iter().all(|r| r.len() == width),
            "arch reply: a {key} row is not {width} wide"
        );
        tables.insert(key, rows);
    }
    Ok(tables)
}

/// The request past either cap — CE.Arch.Cost's fileCap / refCap (the
/// files, and the two reference tables together), read off the
/// definition package — named by what the core built.
fn why_too_large(b: &Built) -> String {
    let caps = &crate::tables::get().limits.caps;
    let files = b.rows("files").len();
    let refs = b.rows("edges").len() + b.rows("pkgEdges").len();
    format!(
        "arch_too_large: {files} files (cap {}), {refs} references (cap {})",
        caps.arch_files, caps.arch_refs
    )
}

/// The reply, consumed strictly: every table at its width, the counts
/// echoing what was built and tallying the tables, every id one this
/// side can label, every focus file at depth 0.
pub fn consume(reply: &Value, t: &Built) -> Result<Reply> {
    let r = Reply {
        tables: read_tables(reply, TABLES)?,
        counts: counts(reply)?,
    };
    let clusters: BTreeSet<i64> = r.rows("clusters").iter().map(|x| x[1]).collect();
    let tallies = [
        t.rows("files").len(),
        t.rows("dirs").len(),
        t.rows("edges").len(),
        t.rows("pkgEdges").len(),
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
fn held(r: &Reply, t: &Built) -> Result<()> {
    let n = |universe: &str| match universe {
        "f" => t.rows("files").len() as i64,
        _ => t.rows("dirs").len() as i64,
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
