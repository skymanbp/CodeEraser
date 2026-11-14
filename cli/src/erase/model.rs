//! The erase plan's data model — types only, importing nothing from
//! its siblings, so every other erase file can reference it DOWNWARD
//! (the house convention: children never `super::`, shared shapes
//! live in a leaf — an intra-module `use super::` web is an import
//! cycle by the graph family's own axis-6 measure, and this module
//! got charged for its first draft).

use serde::Serialize;
use std::collections::BTreeMap;

/// The audit trail's record schema — apply.rs writes it, log.rs reads
/// it back (plan v2.29 step 9, O50). Bump-log: 0.1.0 = the M9 batch-3
/// landing shape.
pub const LOG_SCHEMA: &str = "ce.erase-log/0.1.0";

/// The class names by their frozen codes (the wire's `class` field,
/// erase/1) — the core's table (CE.Erase.Cost `classNames`, read from
/// the definition package's erase document entry). The measuring side
/// keeps no copy since release 1.10.0: the plan carries the codes the
/// core answered and the document names them; this side names a class
/// only where it writes text of its own (the trail record, the diff's
/// hunk header, the convergence refusal).
pub fn class_names() -> &'static [&'static str] {
    crate::tables::get().document.erase.classes
}

/// The reason names by their frozen codes (erase/1's `reason`), the
/// core's table (CE.Erase.Cost `reasonNames`), as `class_names`.
pub fn reason_names() -> &'static [&'static str] {
    crate::tables::get().document.erase.reasons
}

/// Where a candidate comes from, as facts (release 1.10.0): the core
/// renders the provenance clause from them in the console's language,
/// and the plan document carries them beside the English `provenance`.
#[derive(Clone, Debug)]
pub enum Source {
    /// A dead file: the graph's verdict code (1..4) and reason code.
    Dead { verdict: i64, why: i64 },
    /// A verbatim doc twin: the surviving copy and its span, the
    /// Jaccard numerator and denominator, the verbatim word count, and
    /// the links the candidate's section receives.
    Doc {
        twin: String,
        span: (i64, i64),
        jaccard: (i64, i64),
        verbatim: i64,
        linked: i64,
    },
    /// A whole-unit T1 twin: the other copy and its span, the run's
    /// tokens and the first covered unit.
    Twin {
        twin: String,
        span: (i64, i64),
        tokens: i64,
        unit: String,
    },
}

/// The one out-of-class kind gather.rs counts: a verified T1/T2 clone
/// block covering no whole unit has no deterministic-safe erase, and
/// the family that judged it is where the reader goes next.
pub const T1T2_NO_WHOLE_UNIT: &str = "t1t2_block_no_whole_unit";

/// One candidate as measured: the dense facts the wire carries plus
/// the labels the wire deliberately does not (row index is identity;
/// the CLI re-labels the verdicts on return).
pub struct Candidate {
    pub class: usize,
    pub facts: [i64; 4],
    pub path: String,
    /// 1-based inclusive line span; None = the whole file.
    pub span: Option<(i64, i64)>,
    pub provenance: String,
    /// The facts the provenance sentence is written from.
    pub source: Source,
    /// Unresolved reference sites in the row's language — the fact
    /// behind `language_unresolved`, carried for the reader (FIELD-TEST,
    /// plan v2.25); the wire sees it only where a class's facts do.
    pub sites: i64,
}

impl Candidate {
    /// The links the candidate's section receives (a doc twin's), the
    /// fact erase/1's `linked` table carries; 0 for every other row.
    pub fn linked(&self) -> i64 {
        match &self.source {
            Source::Doc { linked, .. } => *linked,
            _ => 0,
        }
    }
}

/// One judged row as the wire returns it (erase/1): the predicate's
/// pair and, since 7.2.0, the target closure's bit — whether this
/// row STANDS for its target (CE.Erase.Cost.keptRows; plan v2.30
/// step 7b). A row that does not stand never becomes a plan row.
#[derive(Debug)]
pub struct Verdict {
    pub eraseable: bool,
    pub reason: i64,
    pub kept: bool,
}

/// One judged plan row. `class` and `reason` are the core's names for
/// the codes beside them (`codes`, what the document request sends);
/// the serialized row — the plan hash's bytes — is unchanged by the
/// codes and the source facts riding beside it.
#[derive(Serialize, Clone)]
pub struct Row {
    pub class: &'static str,
    pub eraseable: bool,
    pub reason: &'static str,
    #[serde(skip)]
    pub codes: (usize, i64),
    pub path: String,
    pub span: Option<(i64, i64)>,
    pub provenance: String,
    #[serde(skip)]
    pub source: Source,
    /// Unresolved sites in this row's language (Candidate::sites).
    pub sites: i64,
    /// fnv1a64 of the target file's bytes at plan time — apply
    /// refuses a file that moved since planning (plans are not
    /// portable across edits).
    pub hash: u64,
}

#[derive(Serialize)]
pub struct Counts {
    pub candidates: usize,
    pub eraseable: usize,
    pub advisory: usize,
    /// Findings OUTSIDE the three planned classes, named
    /// in aggregate: the plan hands the reader their family commands
    /// instead of pretending they have a safe erase.
    pub out_of_class: BTreeMap<&'static str, usize>,
}

#[derive(Serialize)]
pub struct Plan {
    pub rows: Vec<Row>,
    pub counts: Counts,
}

impl Plan {
    /// The rows apply will act on, in plan order.
    pub fn eraseable(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|r| r.eraseable)
    }
}
