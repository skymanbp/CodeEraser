//! docdup — document/comment duplication domain (M5-3, plan §4.1
//! docdup row). 3d lands the extraction half only: segments
//! (geometry), exempt (policy), shingle (content→numbers) and this
//! orchestration; judgment (MinHash/LSH coarse filter + the Haskell
//! Jaccard re-check) is 3g's batch. The schema lifecycle is ONE
//! lifecycle, so the domain owns its CREATE-only DDL and its
//! extraction revision. Pipeline order is fixed and conservation-
//! clean: extract → skeleton strip → wordize → admission floor →
//! exemption classify → store; raw == below_floor + stored.

pub mod exempt;
pub mod judge;
pub mod segments;
pub mod shingle;
pub mod spec;

use crate::scan::lang::Lang;
use anyhow::Result;
use rusqlite::Transaction;

/// Bump when segment-extraction semantics change: it sits in the meta
/// cache key (schema v5), so stale docsegs rows are wiped. 5 = a
/// comment node's adjacency and end_line read its last content row,
/// so `///` / `//!` runs merge like `//` runs (v2.28 amendment,
/// 2026-09-04) — every Rust doc block changes geometry. 4 = words are
/// NFC-composed and combining marks no longer split them (review
/// 2026-08-19), which changes every shingle hash. 3 = the 2026-08-14
/// attainment-line-B amendment (ccm #842): html_line /
/// fenced_code_line / overlong_line masks.
pub const DOCDUP_REV: i64 = 5;

/// CREATE-only DDL (the DROP half lives in dedup/schema.rs). `kind`
/// and `exempt` are the frozen position codes in spec::KIND_NAMES /
/// exempt::EXEMPT_NAMES; `shingles` = sorted deduped shingle u64s LE.
pub const DOCSEGS_SCHEMA: &str = "
CREATE TABLE docsegs (
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  words INTEGER NOT NULL, shingles BLOB NOT NULL, exempt INTEGER NOT NULL);
CREATE INDEX idx_docsegs_file ON docsegs(file_id);
";

/// One admitted segment's facts — the pure half of refresh_segments
/// and the ONE fact source the docdup-segments/oracle instruments
/// share with the cache writer (the unitcache::unit_facts posture).
pub struct SegFact {
    pub kind: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub words: Vec<u64>,
    pub shingles: Vec<u64>,
    pub exempt: i64,
}

pub struct DocFacts {
    pub segs: Vec<SegFact>,
    pub ledger: exempt::Ledger,
}

/// Every admitted document segment of one file, with the full shed
/// ledger.
pub fn doc_facts(text: &str, lang: Lang) -> DocFacts {
    let (raw, shed) = segments::extract(text, lang);
    let mut ledger = exempt::Ledger {
        md: shed,
        ..Default::default()
    };
    let first_comment = raw.iter().position(|s| s.kind == spec::KIND_COMMENT);
    let mut segs = Vec::new();
    for (i, seg) in raw.iter().enumerate() {
        let kept = exempt::strip_skeleton(seg, &mut ledger);
        let mut words = Vec::new();
        for line in kept {
            shingle::line_words(&line.text, line.mask.as_deref(), &mut words);
        }
        if words.len() < spec::MIN_DOC_TOKENS {
            ledger.below_floor += 1;
            continue;
        }
        segs.push(SegFact {
            kind: seg.kind,
            start_line: seg.start_line,
            end_line: seg.end_line,
            shingles: shingle::shingle_set(&words),
            words,
            exempt: exempt::classify(seg, first_comment == Some(i), &mut ledger),
        });
    }
    DocFacts { segs, ledger }
}

/// Replace one file's docsegs rows with the facts — the additive
/// FIFTH per-file parse (RM7: docdup shares no tree with tokenize;
/// the PERF-BUDGET F11 row carries the cost).
pub fn refresh_segments(tx: &Transaction<'_>, file_id: i64, text: &str, lang: Lang) -> Result<()> {
    crate::dedup::schema::replace_file_rows(
        tx,
        "docsegs",
        file_id,
        "INSERT INTO docsegs (file_id, kind, start_line, end_line, words, shingles, exempt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |ins| {
            for s in doc_facts(text, lang).segs {
                ins.execute((
                    file_id,
                    s.kind,
                    s.start_line,
                    s.end_line,
                    s.words.len() as i64,
                    shingle_blob(&s.shingles),
                    s.exempt,
                ))?;
            }
            Ok(())
        },
    )
}

fn shingle_blob(set: &[u64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 * set.len());
    for v in set {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/docdup.rs"]
mod tests;
