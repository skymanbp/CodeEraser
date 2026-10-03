//! The erase faces as the core lays them out (plan v2.32 step 5;
//! CE.Erase.Document, CE.Erase.Lines, CE.Erase.TrailLines): the plan
//! sends its rows — each class and reason by code, its target over one
//! path table, the unresolved sites and the content hash, the
//! predicate's verdict — the out-of-class counts by kind code, and the
//! console's `--check` and `--apply`; the trail sends each record's
//! stamp, class code and target and each refused line's number. The
//! documents, the console lines and the veto come back. The unified
//! diff stays this side's measurement: rendered per file before
//! anything is applied (the hash check names a file that moved since
//! planning), and the core places it as one reference per file.

use crate::document::{self, Answer, Held, Paths, Request, Resolve};
use crate::erase::log::Log;
use crate::erase::model::{CLASS_NAMES, Plan, REASON_NAMES, T1T2_NO_WHOLE_UNIT};
use anyhow::{Context, Result};
use std::path::Path;

/// What ran beside the plan: `--check`, and the rows `--apply` erased.
#[derive(Clone, Copy, Default)]
pub struct Run {
    pub check: bool,
    pub applied: Option<usize>,
}

/// The plan's unified diff, one text per file: the place of the file's
/// first eraseable row in the plan, and its hunks.
pub struct Diffs(Vec<(usize, String)>);

impl Diffs {
    /// Every eraseable file of `p` rendered against the tree at `root`.
    pub fn of(root: &Path, p: &Plan) -> Result<Diffs> {
        Ok(Diffs(crate::erase::render::file_diffs(root, p)?))
    }
}

/// The code of `name` in a frozen name table.
fn code(table: &[&str], name: &str) -> Result<i128> {
    let at = table.iter().position(|n| *n == name);
    Ok(at.with_context(|| format!("erase: {name:?} has no code"))? as i128)
}

/// The plan document, its lines and its veto, over `held` (the link
/// the plan was judged over) when it is whole.
pub fn answer(core: &str, held: Held, p: &Plan, diffs: &Diffs, run: Run) -> Result<Answer> {
    let mut paths = Paths::default();
    let mut cands = Vec::with_capacity(p.rows.len());
    for r in &p.rows {
        let (spanned, (s, e)) = (r.span.is_some(), r.span.unwrap_or((0, 0)));
        cands.push([
            code(&CLASS_NAMES, r.class)?,
            i128::from(paths.id(&r.path)),
            i128::from(spanned),
            i128::from(s),
            i128::from(e),
            i128::from(r.sites),
            i128::from(r.hash),
            i128::from(r.eraseable),
            code(&REASON_NAMES, r.reason)?,
            1,
        ]);
    }
    let kinds = p.counts.out_of_class.iter().map(|(k, n)| {
        let ok = *k == T1T2_NO_WHOLE_UNIT;
        anyhow::ensure!(ok, "erase: out-of-class kind {k:?} has no code");
        Ok([0, *n as i128])
    });
    let req = Request::new("erase")
        .range("paths", paths.list.len())
        .range("cands", cands.len())
        .fact("check", u8::from(run.check))
        .fact("apply", u8::from(run.applied.is_some()))
        .fact("applied", run.applied.unwrap_or(0))
        .rows("outOfClass", kinds.collect::<Result<Vec<_>>>()?)
        .rows("cands", cands);
    let strings = PlanStrings {
        lists: document::Lists(vec![
            ("path", paths.list),
            (
                "provenance",
                p.rows.iter().map(|r| r.provenance.clone()).collect(),
            ),
        ]),
        diffs,
    };
    document::assemble_over(core, held, req, &strings)
}

/// The plan's strings: paths and each row's provenance by index, and
/// each file's diff (a `diff` reference names the file's first and last
/// row; the first is the key).
struct PlanStrings<'a> {
    lists: document::Lists,
    diffs: &'a Diffs,
}

impl Resolve for PlanStrings<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        if class != "diff" {
            return self.lists.resolve(class, ints);
        }
        let first = *ints.first()?;
        let found = self.diffs.0.iter().find(|(i, _)| *i as i128 == first);
        found.map(|(_, d)| d.clone())
    }
}

/// The trail document, its lines and its veto (a refused line).
pub fn trail_answer(core: &str, l: &Log) -> Result<Answer> {
    let mut paths = Paths::default();
    let mut records = Vec::with_capacity(l.rows.len());
    for (i, r) in l.rows.iter().enumerate() {
        let (spanned, (s, e)) = (r.span.is_some(), r.span.unwrap_or((0, 0)));
        records.push([
            i as i128,
            i128::from(r.ts_ms),
            code(&CLASS_NAMES, &r.class)?,
            i128::from(paths.id(&r.path)),
            i128::from(spanned),
            i128::from(s),
            i128::from(e),
        ]);
    }
    let unreadable: Vec<[usize; 2]> = (l.unreadable.iter().enumerate())
        .map(|(k, (line, _))| [k, *line])
        .collect();
    let req = Request::new("erase-trail")
        .range("paths", paths.list.len())
        .range("records", records.len())
        .range("unread", unreadable.len())
        .fact("present", u8::from(l.present))
        .rows("records", records)
        .rows("unreadable", unreadable);
    let column = |f: fn(&crate::erase::log::Record) -> &String| {
        l.rows.iter().map(|r| f(r).clone()).collect()
    };
    let lists = document::Lists(vec![
        ("path", paths.list),
        ("provenance", column(|r| &r.provenance)),
        ("hash", column(|r| &r.hash)),
        ("plan", column(|r| &r.plan)),
        (
            "unreadable",
            l.unreadable.iter().map(|(_, why)| why.clone()).collect(),
        ),
    ]);
    document::assemble(core, req, &lists)
}
