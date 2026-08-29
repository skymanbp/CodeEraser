//! The K23 census (sealed criterion §0 clause 3, §7 K23): the veto is
//! four instruments, one per language family — in the X-1 simulation
//! of the whole rule (veto plus the core's exemptions) 0.64% of Go's
//! exported declarations and 14.9% of TypeScript's were left
//! standing, and at this module's own layer, the veto alone, the same
//! run kept 65.8% and 16.0% — so the operator's window reports the
//! veto per LANGUAGE as rates over the judged domain, never as bare
//! counts. Every number here is the candidate producer's own veto
//! (candidates.rs) run over the same domain and counted where it
//! stopped; the core's mask and exemptions are not applied (that half
//! is `ce deadcode`'s report), so `unmentioned` is the veto's
//! survival, before the core's reading of visibility and category.
//!
//! `collision_saved` names the §6 blindness: a declaration counted as
//! mentioned only because another file DECLARES the same name — every
//! other speller is itself a declarer, so no reference was seen. Its
//! rate over `unmentioned` is the second number K23 asks for.

use super::candidates::{Decl, Veto, domain, veto};
use super::selfref::SelfText;
use super::store;
use crate::dedup::index::Index;
use crate::dedup::tokens::fnv1a;
use crate::fourclass::visibility::VIS_EXPORTED;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The judged domain as the census sees it, keyed `(file, name)` —
/// the domain's public reading, so the K23 corpus instrument
/// (tests/it/eval_mention.rs) measures the tokenizer's arms against
/// the SAME domain the veto judges, never a second extraction of it.
pub fn declarations(idx: &Index) -> Result<BTreeMap<(String, String), Decl>> {
    let txn = idx.raw().unchecked_transaction()?;
    let decls = domain(&txn)?;
    txn.finish()?;
    Ok(decls)
}

/// A count and its exported half (vis bit 0).
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Split {
    pub all: usize,
    pub exported: usize,
}

impl Split {
    fn add(&mut self, exported: bool) {
        self.all += 1;
        self.exported += usize::from(exported);
    }
}

/// Where the veto stopped, by reason (module doc for `collision_saved`).
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(super) struct Vetoed {
    /// Another file spells the name.
    pub other: usize,
    /// A Rust fold key spelled elsewhere (the second chance).
    pub fold: usize,
    /// The declaring file's own exception regions spell it.
    pub self_text: usize,
    /// Of `other`, saved by nothing but a same-name declaration in
    /// another file — the collision blindness, made visible.
    pub collision_saved: usize,
}

/// One language's census: the §3.1 domain, what survived every veto
/// (the candidate table's rows before the core's mask and
/// exemptions), and where the rest stopped.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct LangRates {
    pub declared: Split,
    pub unmentioned: Split,
    pub(super) vetoed: Vetoed,
}

/// The census over the whole judged domain, keyed by the language's
/// report name. One read snapshot, the candidates' discipline.
pub fn census(root: &Path, idx: &Index) -> Result<BTreeMap<&'static str, LangRates>> {
    let txn = idx.raw().unchecked_transaction()?;
    let decls = domain(&txn)?;
    let mut declaring: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (path, name) in decls.keys() {
        declaring.entry(name).or_default().insert(path);
    }
    let mut texts: BTreeMap<&str, SelfText> = BTreeMap::new();
    let mut out: BTreeMap<&'static str, LangRates> = BTreeMap::new();
    for ((path, name), d) in &decls {
        let r = out.entry(d.lang.name()).or_default();
        let exported = d.vis & VIS_EXPORTED != 0;
        r.declared.add(exported);
        match veto(&txn, root, path, name, d.lang, &mut texts)? {
            None => r.unmentioned.add(exported),
            Some(Veto::Other) => {
                r.vetoed.other += 1;
                let declarers = &declaring[name.as_str()];
                if declarers.len() > 1 {
                    let spellers = store::mentioners(&txn, fnv1a(name.as_bytes()) as i64, path)?;
                    r.vetoed.collision_saved +=
                        usize::from(spellers.iter().all(|p| declarers.contains(p.as_str())));
                }
            }
            Some(Veto::Fold) => r.vetoed.fold += 1,
            Some(Veto::SelfText) => r.vetoed.self_text += 1,
        }
    }
    txn.finish()?; // read-only: closing the snapshot, nothing to write
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/unit/mention/rates.rs"]
mod tests;
