//! The structure document (plan v2.32 step 4; design booklet
//! docs/reference/authority-track.md §5): the core lays it out
//! (document/1, CE.Structure.Document) from the structure reply's
//! rows sent back, the flat directory tree and — when the advisory
//! rode — the split candidates with each seam's last line; this side
//! puts the directory names, the advisory's paths and its unit names
//! back. report.rs reads the bound document for the console.

use super::seams::SeamFacts;
use super::tree::Tree;
use super::wire::Reply;
use crate::document::{self, Request, Resolve};
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
    pub tree: &'a Tree,
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
) -> Result<serde_json::Value> {
    let r = p.reply;
    let tree: Vec<[i64; 5]> = p
        .tree
        .dirs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let [parent, depth, subdirs, files] = [
                d.parent as i64,
                d.depth.into(),
                d.subdirs.into(),
                d.files.into(),
            ];
            [i as i64, parent, depth, subdirs, files]
        })
        .collect();
    let mut req = Request::new("structure")
        .range("dirs", p.tree.dirs.len())
        .range("seamFiles", p.seams.map_or(0, |s| s.files.len()))
        .fact("score", r.score)
        .fact("scale", p.scale)
        .fact("declaredDirs", p.declared)
        .fact("deep", i64::from(p.deep))
        .fact("split", i64::from(p.seams.is_some()))
        .rows("entropy", &r.entropy)
        .rows("axes", &r.axes)
        .rows("tree", tree)
        .rows("findings", &r.findings)
        .rows("deviations", &r.deviations)
        .single("days", p.days)
        .single("divergence", r.divergence);
    if let Some(sf) = p.seams {
        req = req
            .rows("splitCandidates", candidates(sf, r)?)
            .rows("sizeExempt", &r.size_exempt);
    }
    let names = Names {
        dirs: names_by_id(p.tree),
        seams: p.seams,
    };
    document::assemble_over(core, held, req.empty(&TABLES), &names)
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

/// Directory names by dense id, the root as `.`.
fn names_by_id(t: &Tree) -> Vec<String> {
    let mut names = vec![String::from("."); t.dirs.len()];
    for (path, &id) in &t.ids {
        if !path.is_empty() {
            names[id] = path.clone();
        }
    }
    names
}

/// The structure document's strings: the directory names, and the
/// advisory's file paths and unit names.
struct Names<'a> {
    dirs: Vec<String>,
    seams: Option<&'a SeamFacts>,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let file = |f: &i128| self.seams?.files.get(usize::try_from(*f).ok()?);
        match (class, ints) {
            ("dir", _) => document::at(&self.dirs, ints),
            ("path", [f]) => file(f).map(|f| f.0.clone()),
            ("unit", [f, u]) => {
                let units = self.seams?.unit_names.get(usize::try_from(*f).ok()?)?;
                units.get(usize::try_from(*u).ok()?).map(|u| u.0.clone())
            }
            _ => None,
        }
    }
}
