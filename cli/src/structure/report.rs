//! The structure document read back (plan v2.32 step 4): the core
//! lays the document out (structure/document.rs, CE.Structure.Document)
//! and, since step 5, its console lines too (CE.Structure.Lines); this
//! reader serves the library's callers that read the document typed.

use serde::Deserialize;
use serde_json::Value;

/// The document read typed; `doc` is the document itself.
#[derive(Debug, Deserialize)]
pub struct Report {
    pub score: i64,
    #[serde(rename = "scoreScale")]
    pub scale: i64,
    pub entropy: Vec<[i64; 2]>,
    pub axes: Vec<[i64; 2]>,
    pub findings: Vec<Finding>,
    pub dirs: usize,
    pub divergence: Option<i64>,
    pub deviations: Vec<Deviation>,
    #[serde(rename = "declaredDirs")]
    pub declared: usize,
    pub deep: bool,
    pub days: Option<u32>,
    pub split: bool,
    #[serde(rename = "splitCandidates", default)]
    pub split_candidates: Vec<Candidate>,
    #[serde(rename = "sizeExempt", default)]
    pub size_exempt: Vec<Exempt>,
    #[serde(skip)]
    pub doc: Value,
}

#[derive(Debug, Deserialize)]
pub struct Finding {
    pub dir: String,
    pub axis: i64,
}

#[derive(Debug, Deserialize)]
pub struct Deviation {
    pub dir: String,
    pub kind: i64,
}

/// A viable seam: the file, the line it falls after, the unit before
/// it, and both sides of the price.
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub path: String,
    #[serde(rename = "afterLine")]
    pub after: u64,
    pub unit: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

/// A file past the soft line with no viable seam (0/0 = no seam).
#[derive(Debug, Deserialize)]
pub struct Exempt {
    pub path: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

crate::report::bound!(Report);
