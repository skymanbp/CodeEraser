//! The faces of the asked families — a question, a unit, a kind list,
//! a path list — and the pack text (plan v2.33 step 10), split from
//! faces/mod.rs at the E01 line when the CLI joined the one road. Same
//! charter as every sibling: read-only, each answer the core's.

use super::Plumb;
use crate::document::Answer;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

/// The schema of the pack-text document (`ce query --prelude --format
/// json`, the MCP `query` tool's `prelude`, the GUI's Prelude button):
/// a face document of this side's — the strings are the built-in
/// prelude and the program the core renders from `[rules.pack]`.
pub const PRELUDE_SCHEMA: &str = "ce.query-prelude/0.1.0";

/// The same-role advisor's document (plan v2.29 step 6): one ask — a
/// unit at `file:line`, a unit by key, free text — and the associative
/// view under `widen`. Advisory like the deadcode symbol table: order
/// and role bits are the core's answer over similar/1, a core that
/// cannot answer is NAMED in the document, and nothing here reaches
/// `ce check`.
pub fn similar(
    root: &Path,
    core: &str,
    ask: &crate::similar::query::Ask,
    widen: bool,
    p: Plumb,
) -> Result<Answer> {
    crate::similar::document::answer(root, p.db, core, ask, widen)
}

/// One question over the index's facts (plan v2.31 step 2): the rules
/// file's rules (the one named, else the project's) under the question,
/// judged by the core's query/1 and labelled back. Report-only: a
/// program error or a core that could not judge is named in the
/// document, never a verdict this side reached.
pub fn query(
    root: &Path,
    core: &str,
    (body, why): (&str, bool),
    file: Option<&Path>,
    p: Plumb,
) -> Result<Answer> {
    asked(root, core, (Some(body.to_string()), why), file, p)
}

/// The rules file judged (plan v2.31 step 2): every assertion's
/// violations with their derivations; the exit on a violation is the
/// core's veto the CLI reads, the document a report everywhere.
pub fn rules(root: &Path, core: &str, file: Option<&Path>, why: bool, p: Plumb) -> Result<Answer> {
    asked(root, core, (None, why), file, p)
}

/// The one road of both: the rules file resolved, the core asked.
fn asked(
    root: &Path,
    core: &str,
    (query, why): (Option<String>, bool),
    file: Option<&Path>,
    p: Plumb,
) -> Result<Answer> {
    let rules = crate::query::rules_source(root, file)?;
    let ask = crate::query::face::Ask { rules, query, why };
    crate::query::face::run(root, p.db, core, &ask)
}

/// The text a question builds on: the built-in prelude and the program
/// the core renders from the project's `[rules.pack]` (null when none
/// is declared) — what `ce query --prelude` prints, as one document.
pub fn pack_text(root: &Path, core: &str) -> Result<Value> {
    let pack = crate::query::face::pack_text(root, core)?;
    Ok(json!({
        "schema": PRELUDE_SCHEMA,
        "prelude": crate::query::PRELUDE,
        "pack": pack,
    }))
}

road!(
    /// The dead code inside functions (plan v2.31 step 5): every unit of
    /// the tree judged over flow/1 and placed back through its legend,
    /// `kinds` narrowing the listed findings (`--kind`; the counts stay
    /// whole). `--check` is the console's gate at `[flow] tier` deny: the
    /// tier is read here on every road (an unreadable ce.toml is this
    /// face's named error), and both facts reach only the lines and the
    /// veto — the document is the same on every face.
    flow(root, core, p, kinds: &[String]) => {
        let config = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
        let gate = (p.check, config.flow.tier() == "deny");
        crate::flow_report::face::run(root, core, kinds, gate)
    }
);

/// The flow kinds as the core's catalogue lists them (plan v2.32 step 5,
/// R8): each kind's name and its label in both languages, read off the
/// definition package the caller loaded — the GUI hub's chips and kind
/// column carry no label map of their own.
pub fn flow_kinds() -> Value {
    let rows = crate::flow_report::kinds().iter();
    Value::Array(
        rows.map(|k| json!({"name": k.name, "en": k.en, "zh": k.zh}))
            .collect(),
    )
}

road!(
    /// The clone-merge suggestions (plan v2.31 step 7): every clone group
    /// anti-unified by the core's merge/1 and labelled back. Advisory: a
    /// core that cannot judge is named in the document, and no gate reads
    /// it. `--group` slices the console alone (the document stays whole).
    merge(root, core, p) => crate::merge::face::run(root, p.db, core, p.group)
);

road!(
    /// The architecture of the tree (plan v2.31 step 9): layers, the arcs
    /// to cut out of the directory cycles, clusters, misplaced files, the
    /// impact of `impact`'s files (`./x` and `a\b` read as the index
    /// spells them, on every face) and per-directory metrics, judged by
    /// the core's arch/1. Advisory: nothing here reaches a gate.
    arch(root, core, p, impact: &[String]) => crate::arch::face::run(root, p.db, core, impact)
);
