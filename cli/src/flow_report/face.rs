//! The one document of `ce flow`, the MCP tool `flow` and the GUI's
//! hub family (design booklet §5.4): every judged-language file the
//! walk admits lowered (flow::lower), its units judged over flow/1
//! (flow::wire::judge, the batches and the refusal-driven exclusion
//! included), and every finding placed back through its unit's legend
//! — the path, the unit, the kind by name, the lines, the variable,
//! and whether the finding is judged or advisory. Labelling only: the
//! findings and the refusals are the core's, and a document whose
//! `degraded` names a reason carries no finding this side reached.

use super::{KINDS, Placed, judged, kind_name, place, unit_at};
use crate::corelink::Link;
use crate::flow::lower::{Lowered, lower_file};
use crate::flow::wire::{self, Verdict};
use anyhow::{Result, anyhow};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const SCHEMA_ID: &str = "ce.flow-report/0.1.0";

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FindingFace {
    pub path: String,
    pub unit: String,
    pub kind: &'static str,
    pub line: u32,
    #[serde(rename = "lineEnd")]
    pub line_end: u32,
    pub var: Option<String>,
    pub judged: bool,
    #[serde(skip)]
    pub nth: usize,
    #[serde(skip)]
    pub code: u8,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RefusedFace {
    pub path: String,
    pub unit: String,
    pub reason: String,
    #[serde(skip)]
    pub nth: usize,
}

pub struct Report {
    pub counts: BTreeMap<&'static str, u64>,
    pub findings: Vec<FindingFace>,
    pub refused: Vec<RefusedFace>,
    pub degraded: Option<String>,
}

impl Report {
    /// The findings the gate reads: judged ones, whatever `--kind`
    /// showed (the filter shapes the listing, never the verdict).
    pub fn judged(&self) -> u64 {
        self.counts.get("judged").copied().unwrap_or(0)
    }
}

/// The whole leg: walked, lowered, judged, placed; `kinds` narrows the
/// listed findings (empty = all), and an unknown kind name is refused.
pub fn run(root: &Path, core: &str, kinds: &[String]) -> Result<Report> {
    let shown = shown_kinds(kinds)?;
    let (_, lowered) = crate::scan::walk::each_surviving(root, |path, lang, bytes| {
        let text = String::from_utf8(bytes).ok();
        let rel = crate::scan::walk::rel_str(root, path);
        Ok(text.and_then(|t| lower_file(&t, lang)).map(|f| (rel, f)))
    })?;
    let (paths, files): (Vec<String>, Vec<Lowered>) = lowered.into_iter().flatten().unzip();
    let judgment = ask(core, &files)?;
    Ok(assemble(&paths, &files, judgment, shown.as_ref()))
}

/// The kinds a face asked to see, by name; None = every kind.
pub fn shown_kinds(kinds: &[String]) -> Result<Option<BTreeSet<u8>>> {
    let names: Vec<&str> = kinds
        .iter()
        .flat_map(|k| k.split(','))
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .collect();
    if names.is_empty() {
        return Ok(None);
    }
    names
        .iter()
        .map(|n| match KINDS.iter().position(|k| k == n) {
            Some(i) => Ok(i as u8),
            None => Err(anyhow!(
                "unknown kind {n:?}: expected {}",
                KINDS.join(" | ")
            )),
        })
        .collect::<Result<BTreeSet<u8>>>()
        .map(Some)
}

/// The core asked once over every lowered unit: the outer error is a
/// wire skew (a malformed reply is never a healthy one), the inner one
/// a named non-judgment — no core, or a core without the family. A
/// tree with no unit asks nothing.
fn ask(core: &str, files: &[Lowered]) -> Result<Result<Verdict, String>> {
    if files.iter().all(|f| f.units.is_empty()) {
        return Ok(Ok(Verdict::default()));
    }
    let mut link = match Link::open(core) {
        Ok((link, _)) => link,
        Err(why) => return Ok(Err(why)),
    };
    if !link.has(wire::CAP) {
        return Ok(Err(format!("core offers no {}", wire::CAP)));
    }
    wire::judge(&mut link, files)
        .map(Ok)
        .map_err(|e| anyhow!(e))
}

/// The report over the placed verdict (or the named degradation):
/// the counts whole, the findings in (path, unit, kind, line) order
/// and narrowed to `shown`, the refusals in (path, unit) order.
pub fn assemble(
    paths: &[String],
    files: &[Lowered],
    judgment: Result<Verdict, String>,
    shown: Option<&BTreeSet<u8>>,
) -> Report {
    let (verdict, degraded) = match judgment {
        Ok(v) => (v, None),
        Err(why) => (Verdict::default(), Some(why)),
    };
    let mut findings: Vec<FindingFace> = verdict
        .findings
        .iter()
        .filter_map(|(f, nth, x)| {
            Some(face(
                &paths[*f],
                files[*f].lang,
                place(&files[*f], *nth, x)?,
            ))
        })
        .collect();
    findings
        .sort_by(|a, b| (&a.path, a.nth, a.code, a.line).cmp(&(&b.path, b.nth, b.code, b.line)));
    let refused = refusals(paths, files, &verdict);
    let mut counts = tables(files);
    counts.insert("findings", findings.len() as u64);
    counts.insert("dynamicUnits", verdict.dynamic_units);
    counts.insert("refused", refused.len() as u64);
    counts.insert(
        "judged",
        findings.iter().filter(|f| f.judged).count() as u64,
    );
    findings.retain(|f| shown.is_none_or(|s| s.contains(&f.code)));
    counts.insert("shown", findings.len() as u64);
    Report {
        counts,
        findings,
        refused,
        degraded,
    }
}

fn face(path: &str, lang: crate::scan::lang::Lang, p: Placed) -> FindingFace {
    FindingFace {
        path: path.to_string(),
        judged: judged(lang, p.kind),
        kind: kind_name(p.kind),
        unit: p.unit,
        line: p.line,
        line_end: p.line_end,
        var: p.var,
        nth: p.nth,
        code: p.kind,
    }
}

/// The rows every lowered unit carries into the four tables.
fn tables(files: &[Lowered]) -> BTreeMap<&'static str, u64> {
    let units = files.iter().flat_map(|f| &f.units);
    let sum = |rows: fn(&crate::flow::lower::Unit) -> usize| {
        units.clone().map(rows).sum::<usize>() as u64
    };
    BTreeMap::from([
        ("units", sum(|_| 1)),
        ("stmts", sum(|u| u.stmts.len())),
        ("vars", sum(|u| u.vars.len())),
        ("uses", sum(|u| u.uses.len())),
    ])
}

/// Every unit left unjudged, with its reason: the lowering's (a unit
/// with no legal shape, never sent) and the core's (a unit it refused
/// by contract, or one heavier than the cap).
fn refusals(paths: &[String], files: &[Lowered], v: &Verdict) -> Vec<RefusedFace> {
    let face = |f: usize, nth: usize, unit: &str, reason: &str| RefusedFace {
        path: paths[f].clone(),
        unit: unit.to_string(),
        reason: reason.to_string(),
        nth,
    };
    let mut out = Vec::new();
    for (f, file) in files.iter().enumerate() {
        out.extend(
            file.unlowered
                .iter()
                .map(|u| face(f, u.nth, &u.name, &u.reason)),
        );
    }
    for (f, nth, why) in &v.refused {
        let unit = unit_at(&files[*f], *nth).map_or("", |u| u.name.as_str());
        out.push(face(*f, *nth, unit, why));
    }
    out.sort_by_key(|r| (r.path.clone(), r.nth));
    out
}

pub fn report_json(r: &Report) -> Value {
    serde_json::json!({
        "schema": SCHEMA_ID,
        "counts": r.counts,
        "findings": r.findings,
        "refused": r.refused,
        "degraded": r.degraded,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/flow_report/face.rs"]
mod tests;
