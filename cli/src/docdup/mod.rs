//! docdup — document/comment duplication domain (M5-3, plan §4.1
//! docdup row). The 3b footprint is deliberately minimal: the schema
//! lifecycle is ONE lifecycle, so the domain owns its CREATE-only DDL
//! and its extraction revision from day one; the segment extractor,
//! exemption filters and MinHash pipeline land in their own batches
//! (design vol.2 §5). Nothing here judges anything yet.

/// Bump when segment-extraction semantics change: it sits in the meta
/// cache key (schema v5), so stale docsegs rows are wiped.
pub const DOCDUP_REV: i64 = 1;

/// CREATE-only DDL (the DROP half lives in dedup/schema.rs). `kind`
/// = segment kind code (md_para / comment_block / docstring — coded
/// when the extractor lands); `shingles` = sorted shingle u64s LE;
/// `exempt` = exemption class code, 0 = live.
pub const DOCSEGS_SCHEMA: &str = "
CREATE TABLE docsegs (
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  words INTEGER NOT NULL, shingles BLOB NOT NULL, exempt INTEGER NOT NULL);
CREATE INDEX idx_docsegs_file ON docsegs(file_id);
";
