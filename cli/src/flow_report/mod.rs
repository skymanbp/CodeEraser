//! The flow family's faces (plan v2.31 step 5; design booklet
//! docs/reference/analysis-track.md §5.4): `ce flow`, the MCP tool, the
//! GUI's hub family, and the vocabulary its two hook legs share with
//! them — the kind names, the judged reading, a finding read back
//! through its unit's legend. Housed beside cli/src/flow/ rather than
//! in it: that directory answers the precision docs (the provenance
//! gate reads it by path), and a face is no answer. Nothing here
//! judges: every finding is the core's over flow/1, the document its
//! document/1 (CE.Flow.Document), and a finding's line and variable
//! are what the lowering's legend supplies.

pub mod console;
pub mod face;
pub mod report;

use crate::flow::lower::{Lowered, Unit};
use crate::flow::wire::Finding;
use crate::scan::lang::Lang;
use serde_json::{Value, json};

/// The kinds by code as the package's flow catalogue lists them
/// (CE.Flow.Document): `(name, advisory)`.
fn kinds() -> &'static [(&'static str, bool)] {
    crate::tables::get().document.flow.kinds
}

/// A kind's name by code; "?" for a code the catalogue does not list.
pub fn kind_name(kind: u8) -> &'static str {
    kinds()
        .get(usize::from(kind))
        .map_or("?", |(name, _)| *name)
}

/// Whether the catalogue marks the kind advisory in every language
/// (booklet §13 item 8: an unused parameter is often an interface's).
pub fn advisory(kind: u8) -> bool {
    kinds().get(usize::from(kind)).is_some_and(|(_, a)| *a)
}

/// A finding is judged when its language passed the precision gate
/// (`flow::judged_mask`) and its kind is not an advisory one.
pub fn judged(lang: Lang, kind: u8) -> bool {
    !advisory(kind) && lang_judged(lang)
}

/// Whether the precision gate admitted `lang` at all.
pub fn lang_judged(lang: Lang) -> bool {
    crate::flow::judged_mask() & (1 << lang as i64) != 0
}

/// One finding read back through its unit's legend: the unit's name
/// and place, the kind, the source lines (a run's first and last
/// statement; a parameter's own line) and the variable's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub unit: String,
    pub nth: usize,
    pub kind: u8,
    pub line: u32,
    pub line_end: u32,
    pub var: Option<String>,
}

/// The unit of `file` at `nth`.
pub fn unit_at(file: &Lowered, nth: usize) -> Option<&Unit> {
    file.units.iter().find(|u| u.nth == nth)
}

/// A finding placed through the legend of the unit it names; None
/// when the unit or an index is not the file's (consume already held
/// every row to its unit, so None is a caller's mismatch).
pub fn place(file: &Lowered, nth: usize, f: &Finding) -> Option<Placed> {
    let unit = unit_at(file, nth)?;
    let legend = &unit.legend;
    let at = |seq: i64| {
        usize::try_from(seq)
            .ok()
            .and_then(|s| legend.stmt_at.get(s))
    };
    let var = usize::try_from(f.v)
        .ok()
        .and_then(|v| legend.var_name.get(v));
    // a parameter's finding names no statement (declSeq −1, CE.Flow.Cost)
    let (line, line_end) = if f.seq < 0 {
        let v = usize::try_from(f.v).ok()?;
        let l = legend.var_at.get(v)?.0;
        (l, l)
    } else {
        (at(f.seq)?.0, at(f.seq_end)?.0)
    };
    Some(Placed {
        unit: unit.name.clone(),
        nth,
        kind: f.kind,
        line,
        line_end,
        var: var.cloned(),
    })
}

/// Findings counted by kind under the catalogue's kind names, every
/// kind present (zeros kept), as the feeds carry them.
pub fn kinds_json(kinds: impl IntoIterator<Item = u8>) -> Value {
    let names = self::kinds();
    let mut n = vec![0u64; names.len()];
    for k in kinds {
        if let Some(slot) = n.get_mut(usize::from(k)) {
            *slot += 1;
        }
    }
    let counted = names
        .iter()
        .zip(n)
        .map(|((k, _), c)| (k.to_string(), json!(c)));
    Value::Object(counted.collect())
}

#[cfg(test)]
#[path = "../../tests/unit/flow_report/mod.rs"]
mod tests;
