//! docdup instrument support — the per-file segment row, the frozen
//! constants echo and the exact-oracle pair math shared by the
//! docdup-segments and docdup-oracle instruments. The oracle drives
//! the SAME extraction and shingle throats the product cache writes
//! (F29): nothing here re-derives what codeeraser::docdup computes.

use codeeraser::docdup::{self, exempt, spec};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Oracle emission floor: J >= 30/100 (decided — instruments §9.6).
pub const JACCARD_UNIVERSE_FLOOR: (u64, u64) = (30, 100);

/// Report floor: J >= 80/100, pre-registered (§9.6 jaccardNum/Den)
/// BEFORE the judge exists — the sample instrument tallies its
/// population bands with this pair, and when CE.Docdup.Cost lands
/// (3g judge batch) the wire's knobs echo holds the product to the
/// same two integers.
pub const JACCARD_REPORT_FLOOR: (u64, u64) = (80, 100);

/// The frozen pair identity every docdup instrument echoes verbatim
/// (D5): the segment geometry IS the identity — spans were frozen by
/// the 3d extraction, so no key/nth resolution is ever re-derived.
pub const DOCDUP_IDENTITY: [&str; 10] = [
    "corpus", "tip", "a_path", "a_kind", "a_start", "a_end", "b_path", "b_kind", "b_start", "b_end",
];

/// Closed audit truth vocabulary (instruments §9.5 C2). paraphrase is
/// single-listed by design: docdup is lexical, so reworded
/// duplication is a designed-in miss for the judge — the class exists
/// so the audit can SAY so instead of folding it away.
pub const DOCDUP_TRUTHS: [&str; 8] = [
    "redundant",
    "paraphrase",
    "license",
    "skeleton",
    "tabular",
    "quoted",
    "deliberate_xref",
    "unrelated",
];

/// F31 cost bound: a corpus with more live segments than this is
/// WITHHELD with a written reason, never silently downsampled.
pub const DOCDUP_ORACLE_SEGCAP: usize = 8192;

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
        ("indented_code_lines", lg.indented_code_lines),
        ("html_line", lg.html_line),
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

/// One live segment under oracle scrutiny, in identity order.
pub struct OracleSeg {
    pub path: String,
    pub kind: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub words: Vec<u64>,
    pub set: Vec<u64>,
    pub seq: Vec<u64>,
}

/// Every LIVE admitted segment of a walked tree (exempt segments are
/// outside the duplication corpus by definition).
pub fn live_segments(walked: &[super::WalkedFile]) -> Vec<OracleSeg> {
    let mut out = Vec::new();
    for (path, code, text) in walked {
        for s in docdup::doc_facts(text, super::lang_of(code)).segs {
            if s.exempt == exempt::EXEMPT_LIVE {
                out.push(OracleSeg {
                    path: path.clone(),
                    kind: s.kind,
                    start_line: s.start_line,
                    end_line: s.end_line,
                    seq: docdup::shingle::shingle_seq(&s.words),
                    set: s.shingles,
                    words: s.words,
                });
            }
        }
    }
    out
}

/// Exact candidate enumeration: J >= 30/100 > 0 requires a shared
/// shingle, and a verbatim run >= 50 words spans >= 46 shingles — so
/// walking the inverted index visits EVERY emittable pair. This is a
/// complete enumerator, not an approximation.
pub fn candidate_pairs(segs: &[OracleSeg]) -> BTreeSet<(usize, usize)> {
    let mut inverted: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (i, s) in segs.iter().enumerate() {
        for h in &s.set {
            inverted.entry(*h).or_default().push(i);
        }
    }
    let mut cand = BTreeSet::new();
    for list in inverted.values() {
        for (a, i) in list.iter().enumerate() {
            for j in &list[a + 1..] {
                cand.insert((*i, *j));
            }
        }
    }
    cand
}

/// |A∩B| of two sorted deduped sets.
pub fn set_inter(a: &[u64], b: &[u64]) -> u64 {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

/// Longest common contiguous run of two shingle sequences, in WORDS
/// (a run of R shingles spans R + DOC_SHINGLE − 1 words; 0 if none).
pub fn verbatim_words(a: &[u64], b: &[u64]) -> usize {
    let mut prev = vec![0usize; b.len() + 1];
    let mut best = 0;
    for &x in a {
        let mut cur = vec![0usize; b.len() + 1];
        for (j, &y) in b.iter().enumerate() {
            if x == y {
                cur[j + 1] = prev[j] + 1;
                best = best.max(cur[j + 1]);
            }
        }
        prev = cur;
    }
    if best == 0 {
        0
    } else {
        best + spec::DOC_SHINGLE - 1
    }
}
