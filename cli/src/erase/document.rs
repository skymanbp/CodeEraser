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
//! planning), and the core places it as one reference per file; every
//! string is sent and the core spells it in.

use crate::document::{self, Answer, Held, Paths, Request};
use crate::erase::log::Log;
use crate::erase::model::{Plan, Row, Source, T1T2_NO_WHOLE_UNIT, class_names};
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

    /// The whole diff, one file after another, each ended by a newline
    /// — the GUI preview's `diff` (faces::erase_preview).
    pub fn unified(&self) -> String {
        self.0.iter().map(|(_, d)| d.clone() + "\n").collect()
    }

    /// Each of the plan's `rows` rows: its file's diff at the file's
    /// first row (a `diff` reference's key), nothing elsewhere.
    fn by_row(&self, rows: usize) -> Vec<Option<&String>> {
        let mut out = vec![None; rows];
        for (i, d) in self.0.iter().rev() {
            if let Some(slot) = out.get_mut(*i) {
                *slot = Some(d);
            }
        }
        out
    }
}

/// The code of `name` in the core's class table (a trail record names
/// its class; the trail document sends it by code).
fn code(table: &[&str], name: &str) -> Result<i128> {
    let at = table.iter().position(|n| *n == name);
    Ok(at.with_context(|| format!("erase: {name:?} has no code"))? as i128)
}

/// Each row's provenance facts (release 1.10.0), one table per source
/// family keyed by the row's place: the core renders the clause from
/// them in the console's language and lays them out in the document.
#[derive(Default)]
struct Sources {
    dead: Vec<[i128; 3]>,
    doc: Vec<[i128; 8]>,
    twin: Vec<[i128; 5]>,
    units: Vec<String>,
}

impl Sources {
    /// Row `i`'s facts into its family's table, and its covered unit
    /// (empty for a row of another family: the `unit` reference is
    /// keyed by row, like `provenance`).
    fn push(&mut self, i: usize, r: &Row, paths: &mut Paths) {
        let i = i as i128;
        if !matches!(r.source, Source::Twin { .. }) {
            self.units.push(String::new());
        }
        match &r.source {
            Source::Dead { verdict, why } => self.dead.push([i, (*verdict).into(), (*why).into()]),
            Source::Doc {
                twin,
                span,
                jaccard,
                verbatim,
                linked,
            } => self.doc.push([
                i,
                paths.id(twin).into(),
                span.0.into(),
                span.1.into(),
                jaccard.0.into(),
                jaccard.1.into(),
                (*verbatim).into(),
                (*linked).into(),
            ]),
            Source::Twin {
                twin,
                span,
                tokens,
                unit,
            } => {
                self.twin.push([
                    i,
                    paths.id(twin).into(),
                    span.0.into(),
                    span.1.into(),
                    (*tokens).into(),
                ]);
                self.units.push(unit.clone());
            }
        }
    }
}

/// A row's optional span as its three wire columns: spanned, start, end.
fn span_cols(span: Option<(i64, i64)>) -> [i128; 3] {
    let (s, e) = span.unwrap_or((0, 0));
    [i128::from(span.is_some()), i128::from(s), i128::from(e)]
}

/// The plan document, its lines and its veto, over `held` (the link
/// the plan was judged over) when it is whole. Class and reason go by
/// the codes the core answered (`Row::codes`).
pub fn answer(core: &str, held: Held, p: &Plan, diffs: &Diffs, run: Run) -> Result<Answer> {
    let mut paths = Paths::default();
    let mut cands = Vec::with_capacity(p.rows.len());
    let mut sources = Sources::default();
    for r in &p.rows {
        let [spanned, s, e] = span_cols(r.span);
        cands.push([
            r.codes.0 as i128,
            i128::from(paths.id(&r.path)),
            spanned,
            s,
            e,
            i128::from(r.sites),
            i128::from(r.hash),
            i128::from(r.eraseable),
            i128::from(r.codes.1),
            1,
        ]);
        sources.push(cands.len() - 1, r, &mut paths);
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
        .rows("cands", cands)
        .rows("provDead", sources.dead)
        .rows("provDoc", sources.doc)
        .rows("provTwin", sources.twin);
    // a `diff` reference names the file's first and last row; the
    // first is the key
    let req = req
        .text("path", &paths.list)
        .texts("provenance", p.rows.iter().map(|r| &r.provenance))
        .texts("unit", sources.units.iter())
        .text("diff", diffs.by_row(p.rows.len()));
    document::assemble_over(core, held, req)
}

/// The trail document, its lines and its veto (a refused line).
pub fn trail_answer(core: &str, l: &Log) -> Result<Answer> {
    let mut paths = Paths::default();
    let mut records = Vec::with_capacity(l.rows.len());
    for (i, r) in l.rows.iter().enumerate() {
        let [spanned, s, e] = span_cols(r.span);
        let path = i128::from(paths.id(&r.path));
        let class = code(class_names(), &r.class)?;
        records.push([i as i128, i128::from(r.ts_ms), class, path, spanned, s, e]);
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
    let column = |f: fn(&crate::erase::log::Record) -> &String| -> Vec<&String> {
        l.rows.iter().map(f).collect()
    };
    let req = req
        .text("path", &paths.list)
        .text("provenance", column(|r| &r.provenance))
        .text("hash", column(|r| &r.hash))
        .text("plan", column(|r| &r.plan))
        .texts("unreadable", l.unreadable.iter().map(|(_, why)| why));
    document::assemble(core, req)
}
