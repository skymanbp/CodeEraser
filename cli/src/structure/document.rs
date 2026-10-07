//! The structure document (plan v2.32 step 4; design booklet
//! docs/reference/authority-track.md §5): the core lays it out
//! (document/1, CE.Structure.Document) from the structure reply's
//! rows sent back — the flat directory tree the core built and its
//! directory names (plan v2.33 W1 item 3), and, when the advisory
//! rode, the split candidates with each seam's last line — and the
//! advisory's paths and its unit names, which the core spells into the
//! document and its console lines (CE.Structure.Lines).
//! report.rs reads the bound document for the library's callers.

use super::seams::SeamFacts;
use super::wire::Reply;
use crate::document::{self, Request};
use anyhow::{Result, ensure};

/// The tables a structure request carries.
const TABLES: [&str; 9] = [
    "days",
    "divergence",
    "entropy",
    "axes",
    "tree",
    "findings",
    "deviations",
    "splitCandidates",
    "sizeExempt",
];

/// What the document is assembled from beyond the reply: the switches
/// and the declared-directory count the request carried, and the
/// effective scale from the knob echo.
pub(super) struct Parts<'a> {
    pub reply: &'a Reply,
    pub scale: i64,
    pub declared: usize,
    pub deep: bool,
    pub days: Option<u32>,
    pub seams: Option<&'a SeamFacts>,
}

/// The document over `p`, laid out over the verdict's link `held` (a
/// fresh one to `core` when that link is spent).
pub(super) fn assemble(
    core: &str,
    held: document::Held,
    p: &Parts<'_>,
) -> Result<document::Answer> {
    let r = p.reply;
    let mut req = Request::new("structure")
        .range("dirs", r.tree.len())
        .range("seamFiles", p.seams.map_or(0, |s| s.files.len()))
        .fact("score", r.score)
        .fact("scale", p.scale)
        .fact("declaredDirs", p.declared)
        .fact("deep", i64::from(p.deep))
        .fact("split", i64::from(p.seams.is_some()))
        .rows("entropy", &r.entropy)
        .rows("axes", &r.axes)
        .rows("tree", &r.tree)
        .rows("findings", &r.findings)
        .rows("deviations", &r.deviations)
        .single("days", p.days)
        .single("divergence", r.divergence);
    if let Some(sf) = p.seams {
        req = req
            .rows("splitCandidates", candidates(sf, r)?)
            .rows("sizeExempt", &r.size_exempt);
    }
    let mut req = req.empty(&TABLES).text("dir", &r.dirs);
    if let Some(sf) = p.seams {
        let units: Vec<Vec<&String>> = (sf.unit_names.iter())
            .map(|units| units.iter().map(|u| &u.0).collect())
            .collect();
        req = req
            .texts("path", sf.files.iter().map(|f| &f.0))
            .text("unit", units);
    }
    document::assemble_over(core, held, req)
}

/// structure/1's [file, unit, benefit, cost] with the unit's last
/// line after it; a unit the file does not hold is the reply's
/// defect, named.
fn candidates(sf: &SeamFacts, r: &Reply) -> Result<Vec<[i64; 5]>> {
    r.split_candidates
        .iter()
        .map(|&[f, u, b, c]| {
            let end = usize::try_from(f)
                .ok()
                .and_then(|f| sf.unit_names.get(f))
                .and_then(|units| units.get(usize::try_from(u).ok()?));
            ensure!(end.is_some(), "split reply: unit {u} outside file {f}");
            Ok([f, u, b, c, end.map_or(0, |e| e.1 as i64)])
        })
        .collect()
}
