//! The audit trail's READER (plan v2.29 step 9, O50). `.ce/erase-log.ndjson`
//! has had one writer since M9 batch 3 (apply.rs) and no face: a record
//! nobody can read back is a record nobody checks. One document, three
//! faces — `ce erase --log`, the MCP tool `erase_log`, the GUI erase
//! screen's log section — and this module never writes. A line it
//! cannot read is COUNTED and named by line number, never skipped in
//! silence: the file is the apply contract's evidence, not telemetry.
//! The document and the console lines are the core's
//! (erase/document.rs `trail_answer`, CE.Erase.Document `trailDoc`).

use crate::erase::model::{CLASS_NAMES, LOG_SCHEMA};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Where apply.rs appends, root-relative (append_log; the core's
/// `logRel` names the same place in the document).
const LOG_REL: &str = ".ce/erase-log.ndjson";

/// One record exactly as apply.rs writes it — the same eight fields,
/// read strictly: a writer that drifted is a NAMED unreadable line,
/// never a silently partial row.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema: String,
    pub ts_ms: u64,
    pub class: String,
    pub path: String,
    pub span: Option<(i64, i64)>,
    pub provenance: String,
    pub hash: String,
    pub plan: String,
}

#[derive(Debug, Default)]
pub struct Log {
    /// Whether the file exists at all — "no log yet" and "an empty
    /// log" are two different facts.
    pub present: bool,
    pub rows: Vec<Record>,
    /// (1-based line, why) for every line the reader refused.
    pub unreadable: Vec<(usize, String)>,
}

/// Read the whole trail. A missing file is an empty, absent log (no
/// apply ever ran here); any other read failure is an error.
pub fn read(root: &Path) -> Result<Log> {
    let path = root.join(LOG_REL);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Log::default()),
        Err(e) => return Err(e).with_context(|| path.display().to_string()),
    };
    let mut log = Log {
        present: true,
        ..Log::default()
    };
    for (i, line) in text.lines().enumerate() {
        match record(line) {
            Ok(r) => log.rows.push(r),
            Err(why) => log.unreadable.push((i + 1, why)),
        }
    }
    Ok(log)
}

/// One line → one record, or the reason it is not one. A class the
/// writer's table does not hold cannot cross to the core (a record's
/// class goes by code), so it is a refused line whose reason names the
/// class it read (booklet §13 item 42).
fn record(line: &str) -> std::result::Result<Record, String> {
    let r: Record = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if r.schema != LOG_SCHEMA {
        return Err(format!("schema {:?} is not {LOG_SCHEMA}", r.schema));
    }
    if !CLASS_NAMES.contains(&r.class.as_str()) {
        return Err(format!("class {:?} is not an erase class", r.class));
    }
    Ok(r)
}

#[cfg(test)]
#[path = "../../tests/unit/erase/log.rs"]
mod tests;
