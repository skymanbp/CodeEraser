//! `ce graph --mentions`: the pass's own console/JSON face — report-
//! only, the header counters, the convergence facts and the K23
//! per-language census of the veto, so the universe is observable
//! before any judgment consumes it (K39–K42 are library legs; this is
//! the operator's window on the same numbers). The core lays the
//! document and its console lines out (document/1,
//! CE.Mention.Document, CE.Mention.Lines) from this side's numbers.

use super::LangRates;
use crate::document::{self, Answer, Request};
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(root: &Path, db: Option<PathBuf>, core: &str, json: bool) -> ExitCode {
    crate::report::graph_face("mentions", document(root, db, core), json)
}

/// The judged index first (the `outside` counters compare against
/// its `files` table and the census reads its declarations), then
/// the mention pass over the same tree, then the veto counted per
/// language — and the document the core lays out over them.
pub fn document(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Answer> {
    let (idx, _db) = crate::dedup::refreshed_index(root, db)?;
    let stats = super::refresh(root, &idx)?;
    let rates = super::rates::census(root, &idx)?;
    document::assemble(core, request(&stats, &rates)?)
}

/// The header as facts — every counter of `Stats` under its dotted
/// path (`skipped.signed`), a flag as 0 / 1 — beside the index
/// revision, and one census row per language.
pub(super) fn request(
    s: &super::Stats,
    rates: &BTreeMap<&'static str, LangRates>,
) -> Result<Request> {
    let mut facts = vec![("mention_rev".to_string(), super::MENTION_REV)];
    flatten("", &serde_json::to_value(s)?, &mut facts)?;
    let rows = rates
        .iter()
        .map(|(name, r)| {
            let lang = Lang::ALL
                .iter()
                .find(|l| l.name() == *name)
                .with_context(|| format!("census language {name} has no code"))?;
            let v = &r.vetoed;
            Ok([
                *lang as i64,
                r.declared.all as i64,
                r.declared.exported as i64,
                r.unmentioned.all as i64,
                r.unmentioned.exported as i64,
                v.other as i64,
                v.fold as i64,
                v.self_text as i64,
                v.collision_saved as i64,
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    let req = facts
        .into_iter()
        .fold(Request::new("mentions"), |req, (k, n)| req.fact(&k, n));
    Ok(req.rows("rates", rows))
}

/// `v`'s integer leaves under their dotted paths.
fn flatten(at: &str, v: &Value, out: &mut Vec<(String, i64)>) -> Result<()> {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let path = if at.is_empty() {
                    k.clone()
                } else {
                    format!("{at}.{k}")
                };
                flatten(&path, x, out)?;
            }
        }
        Value::Bool(b) => out.push((at.to_string(), i64::from(*b))),
        _ => out.push((
            at.to_string(),
            v.as_i64().with_context(|| format!("{at}: {v}"))?,
        )),
    }
    Ok(())
}
