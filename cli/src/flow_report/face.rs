//! The one document of `ce flow`, the MCP tool `flow` and the GUI's
//! hub family (design booklet §5.4): every judged-language file the
//! walk admits lowered (flow::lower), its units judged over flow/1
//! (flow::wire::judge, the batches and the refusal-driven exclusion
//! included), and every finding placed back through its unit's legend.
//! The core lays the document out (document/1, CE.Flow.Document): the
//! kind names, which findings are judged, the order, the `--kind`
//! filter and the counts are its. This side sends the placed findings
//! (lines and variable index), the refusals, each file's language,
//! and the paths, unit names, variable names and reasons, which the
//! core orders and spells in (crate::document). A document whose
//! `degraded` names a reason carries no finding this side reached.

use super::{place, unit_at};
use crate::document::{self, Answer, Held, Request, Why};
use crate::flow::lower::{Lowered, lower_file};
use crate::flow::wire::{self, Verdict};
use anyhow::{Result, anyhow};
use std::path::Path;

/// The whole leg: walked, lowered, judged, placed, laid out; `kinds`
/// narrows the listed findings (empty = all), and a kind name the
/// catalogue does not list is the core's refusal, named here by the
/// name as given. `gate` is the console's `--check` and whether
/// `[flow] tier` is deny: facts only the lines and the veto read (a
/// machine face sends neither).
pub fn run(root: &Path, core: &str, kinds: &[String], gate: (bool, bool)) -> Result<Answer> {
    let shown = shown_kinds(kinds);
    let (_, lowered) = crate::scan::walk::each_surviving(root, |path, lang, bytes| {
        let text = String::from_utf8(bytes).ok();
        let rel = crate::scan::walk::rel_str(root, path);
        Ok(text.and_then(|t| lower_file(&t, lang)).map(|f| (rel, f)))
    })?;
    let (paths, files): (Vec<String>, Vec<Lowered>) = lowered.into_iter().flatten().unzip();
    let mut held = document::open(core);
    let judgment = ask(&mut held, &files)?;
    let (check, deny) = gate;
    let req = request((&paths, &files), judgment, shown.as_deref())
        .fact("check", u8::from(check))
        .fact("deny", u8::from(deny));
    document::assemble_over(core, held, req).map_err(|e| named_kind(e, shown.as_deref()))
}

/// Per file, `[nth, …]` in the order the file holds them: each lowered
/// unit's name (`units`), each unit the lowering left out by its own
/// name (`unlowered`), each lowered unit's variable names (`vars`). A
/// unit reference spells the first lowered unit at its `nth`, else the
/// first left out, else nothing (one the core refused that is no
/// lowered unit).
fn unit_strings(files: &[Lowered]) -> [(&'static str, serde_json::Value); 3] {
    let per = |f: fn(&Lowered) -> serde_json::Value| -> serde_json::Value {
        files.iter().map(f).collect()
    };
    [
        (
            "units",
            per(|f| {
                f.units
                    .iter()
                    .map(|u| serde_json::json!([u.nth, u.name]))
                    .collect()
            }),
        ),
        (
            "unlowered",
            per(|f| {
                f.unlowered
                    .iter()
                    .map(|u| serde_json::json!([u.nth, u.name]))
                    .collect()
            }),
        ),
        (
            "vars",
            per(|f| {
                f.units
                    .iter()
                    .map(|u| serde_json::json!([u.nth, u.legend.var_name]))
                    .collect()
            }),
        ),
    ]
}

/// The kinds a face asked to see, by name, in the order given and each
/// once: a name the package's flow catalogue lists as its code, any
/// other as −1, which the core refuses with the names it does list.
/// None = every kind.
pub fn shown_kinds(kinds: &[String]) -> Option<Vec<(String, i64)>> {
    let known = super::kinds();
    let mut shown: Vec<(String, i64)> = Vec::new();
    let names = kinds.iter().flat_map(|k| k.split(',')).map(str::trim);
    for name in names.filter(|k| !k.is_empty()) {
        if shown.iter().all(|(seen, _)| seen != name) {
            let code = known.iter().position(|row| row.name == name);
            shown.push((name.to_string(), code.map_or(-1, |i| i as i64)));
        }
    }
    (!shown.is_empty()).then_some(shown)
}

/// The core's refusal of a `shown` row it does not list, named by the
/// kind as the face was given it; any other error as it is.
fn named_kind(e: anyhow::Error, shown: Option<&[(String, i64)]>) -> anyhow::Error {
    let text = e.to_string();
    let Some((head, listed)) = text.split_once(": unknown kind; ") else {
        return e;
    };
    let row = head
        .rsplit("shown ")
        .next()
        .and_then(|i| i.parse::<usize>().ok());
    match row.and_then(|i| shown?.get(i)) {
        Some((name, _)) => anyhow!("flow document: unknown kind {name:?}; {listed}"),
        None => e,
    }
}

/// The core asked once over every lowered unit, over the face's link:
/// the outer error is a wire skew (a malformed reply is never a healthy
/// one), the inner one a named non-judgment — no core, or a core
/// without the family. A tree with no unit asks nothing.
fn ask(held: &mut Held, files: &[Lowered]) -> Result<Result<Verdict, String>> {
    if files.iter().all(|f| f.units.is_empty()) {
        return Ok(Ok(Verdict::default()));
    }
    let link = match held {
        Ok(link) => link,
        Err(why) => return Ok(Err(why.clone())),
    };
    if !link.has(wire::CAP) {
        return Ok(Err(format!("core offers no {}", wire::CAP)));
    }
    wire::judge(link, files).map(Ok).map_err(|e| anyhow!(e))
}

/// The document request over the verdict (or the named degradation):
/// every finding a unit's legend places as [file, nth, kind, line,
/// lineEnd, variable or −1], the refusals as [file, nth, reason], each
/// file's language, the kinds shown, the measured tallies, and the
/// strings: the paths, the reasons and the units' (`unit_strings`).
fn request(
    (paths, files): (&[String], &[Lowered]),
    judgment: Result<Verdict, String>,
    shown: Option<&[(String, i64)]>,
) -> Request {
    let mut why = Why::default();
    let (verdict, degraded) = match judgment {
        Ok(v) => (v, None),
        Err(reason) => (Verdict::default(), Some(why.add(reason))),
    };
    let found = placed(files, &verdict);
    let (unlowered, refused) = reasons(files, &mut why, &verdict);
    let numbered = |xs: Vec<usize>| -> Vec<[usize; 2]> {
        xs.into_iter().enumerate().map(|(f, x)| [f, x]).collect()
    };
    let req = Request::new("flow")
        .range("files", files.len())
        .rows(
            "langs",
            numbered(files.iter().map(|x| x.lang as usize).collect()),
        )
        .rows(
            "shown",
            shown.map_or(Vec::new(), |s| s.iter().map(|(_, k)| [*k]).collect()),
        )
        .rows("unlowered", unlowered)
        .rows("findings", found)
        .rows("refused", refused)
        .fact("dynamicUnits", verdict.dynamic_units);
    let req = tallies(files)
        .into_iter()
        .fold(req, |q, (k, n)| q.fact(k, n));
    let req = unit_strings(files)
        .into_iter()
        .fold(req, |q, (class, s)| q.text(class, s))
        .range("why", why.count())
        .text("path", paths)
        .text("why", why.list());
    match degraded {
        Some(i) => req.degraded(i),
        None => req,
    }
}

/// The units this side could not lower and the units the core refused,
/// each as [file, nth, reason text].
fn reasons(
    files: &[Lowered],
    why: &mut Why,
    verdict: &Verdict,
) -> (Vec<[usize; 3]>, Vec<[usize; 3]>) {
    let mut unlowered = Vec::new();
    for (f, file) in files.iter().enumerate() {
        for u in &file.unlowered {
            unlowered.push([f, u.nth, why.add(u.reason.clone())]);
        }
    }
    let refused = verdict
        .refused
        .iter()
        .map(|(f, nth, reason)| [*f, *nth, why.add(reason.clone())])
        .collect();
    (unlowered, refused)
}

/// Every finding its unit's legend places, as [file, nth, kind, line,
/// lineEnd, variable or −1], in the verdict's order (a finding the
/// legend cannot place is dropped, as it always was).
fn placed(files: &[Lowered], verdict: &Verdict) -> Vec<[i64; 6]> {
    let row = |f: usize, nth: usize, x: &crate::flow::wire::Finding| {
        let p = place(&files[f], nth, x)?;
        let legend = &unit_at(&files[f], nth)?.legend;
        let v = usize::try_from(x.v)
            .ok()
            .filter(|v| *v < legend.var_name.len())
            .map_or(-1, |v| v as i64);
        let (f, nth) = (f as i64, nth as i64);
        Some([
            f,
            nth,
            i64::from(p.kind),
            i64::from(p.line),
            i64::from(p.line_end),
            v,
        ])
    };
    verdict
        .findings
        .iter()
        .filter_map(|(f, nth, x)| row(*f, *nth, x))
        .collect()
}

/// The rows every lowered unit carries into the four tables.
fn tallies(files: &[Lowered]) -> [(&'static str, u64); 4] {
    let units = files.iter().flat_map(|f| &f.units);
    let sum = |rows: fn(&crate::flow::lower::Unit) -> usize| {
        units.clone().map(rows).sum::<usize>() as u64
    };
    [
        ("units", sum(|_| 1)),
        ("stmts", sum(|u| u.stmts.len())),
        ("vars", sum(|u| u.vars.len())),
        ("uses", sum(|u| u.uses.len())),
    ]
}

#[cfg(test)]
#[path = "../../tests/unit/flow_report/face.rs"]
mod tests;
