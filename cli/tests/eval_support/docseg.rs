//! docdup instrument support — the per-file segment row, the frozen
//! constants echo and the exact-oracle pair math shared by the
//! docdup-segments and docdup-oracle instruments. The oracle drives
//! the SAME extraction and shingle throats the product cache writes
//! (F29): nothing here re-derives what codeeraser::docdup computes.

use codeeraser::docdup::{self, exempt, spec};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The frozen pair identity every docdup instrument echoes verbatim
/// (D5): the segment geometry IS the identity — spans were frozen by
/// the 3d extraction, so no key/nth resolution is ever re-derived.
pub const DOCDUP_IDENTITY: [&str; 10] = [
    "corpus", "tip", "a_path", "a_kind", "a_start", "a_end", "b_path", "b_kind", "b_start", "b_end",
];

/// The docdup family's mount into the ONE review registry.
pub fn docdup_review_doc(corpus: &str) -> Value {
    super::review_of("docdup", corpus)
}

pub fn docdup_constants() -> Value {
    json!({
        "min_doc_tokens": spec::MIN_DOC_TOKENS,
        "doc_shingle": spec::DOC_SHINGLE,
        "verbatim_floor": spec::VERBATIM_FLOOR,
        "license_head_lines": spec::LICENSE_HEAD_LINES,
        "doc_line_cap": spec::DOC_LINE_CAP,
        "kinds": spec::KIND_NAMES,
        "exempt": exempt::EXEMPT_NAMES,
        "docdup_rev": docdup::DOCDUP_REV,
        // the exemption routes with structurally-zero counts state
        // WHY here (3d exit: nonzero or explained, never silent)
        "route_notes": {
            "path": "walk::in_scope refuses excluded paths before extraction runs",
            "baseline": "the baseline exemption stock arrives with ce baseline (3i)",
            "inline_allow": "the ce:allow(docdup) vocabulary is CE-specific; the route is \
                 proven live by the seeded counterfactual battery, not by corpus incidence",
        },
    })
}

/// One file's frozen segment row — a pure function of (path, lang,
/// text) through the product doc_facts throat.
pub fn docdup_row(path: &str, code: &str, text: &str) -> Value {
    let facts = docdup::doc_facts(text, super::lang_of(code));
    let mut segs_by: BTreeMap<&str, u64> = BTreeMap::new();
    let mut exempt_by: BTreeMap<&str, u64> = BTreeMap::new();
    let mut live = 0u64;
    for s in &facts.segs {
        *segs_by
            .entry(spec::KIND_NAMES[s.kind as usize])
            .or_insert(0) += 1;
        if s.exempt == exempt::EXEMPT_LIVE {
            live += 1;
        } else {
            *exempt_by
                .entry(exempt::EXEMPT_NAMES[s.exempt as usize])
                .or_insert(0) += 1;
        }
    }
    json!({
        "path": path,
        "lang": code,
        "sha256": super::content_sha(text),
        "segs_by": segs_by,
        "live": live,
        "exempt_by": exempt_by,
        "ledger": ledger_obj(&facts.ledger),
        "seg_sha256": super::content_sha(&seg_canon(&facts.segs)),
    })
}

/// Nonzero ledger counters only — most rows shed nothing.
fn ledger_obj(lg: &exempt::Ledger) -> Value {
    let rows = [
        ("skeleton_line", lg.skeleton_line),
        ("allow_missing_why", lg.allow_missing_why),
        ("below_floor", lg.below_floor),
        ("indented_code_lines", lg.md.indented),
        ("html_line", lg.md.html),
        ("fenced_code_line", lg.fenced_code_line),
        ("overlong_line", lg.overlong_line),
    ];
    Value::Object(
        rows.into_iter()
            .filter(|(_, n)| *n > 0)
            .map(|(k, n)| (k.to_string(), json!(n)))
            .collect(),
    )
}

/// Canonical text of a file's admitted segments — ONE digest fixes
/// geometry, word counts, exemption codes and full shingle sets
/// without freezing thousands of integers.
fn seg_canon(segs: &[docdup::SegFact]) -> String {
    segs.iter()
        .map(|s| {
            let shingles: Vec<String> = s.shingles.iter().map(u64::to_string).collect();
            format!(
                "{}|{}|{}|{}|{}|{}",
                s.kind,
                s.start_line,
                s.end_line,
                s.words.len(),
                s.exempt,
                shingles.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Family summary with row-level conservation inlined: every admitted
/// segment is live or exempt, nothing else.
pub fn docdup_summarize(rows: &[Value]) -> Value {
    let (mut segs_by, mut exempt_by) = (BTreeMap::new(), BTreeMap::new());
    let mut ledger: BTreeMap<String, u64> = BTreeMap::new();
    let (mut live, mut with_segs) = (0u64, 0u64);
    for row in rows {
        let total = super::sum_obj_into(&row["segs_by"], &mut segs_by);
        let ex = super::sum_obj_into(&row["exempt_by"], &mut exempt_by);
        super::sum_obj_into(&row["ledger"], &mut ledger);
        let row_live = row["live"].as_u64().expect("live");
        assert_eq!(
            total,
            row_live + ex,
            "{}: admitted != live + exempt",
            row["path"]
        );
        live += row_live;
        with_segs += u64::from(total > 0);
    }
    json!({
        "files_with_segments": with_segs,
        "segs_by": segs_by,
        "live": live,
        "exempt_by": exempt_by,
        "ledger": ledger,
    })
}
