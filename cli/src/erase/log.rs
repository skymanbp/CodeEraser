//! The audit trail's READER (plan v2.29 step 9, O50). `.ce/erase-log.ndjson`
//! has had one writer since M9 batch 3 (apply.rs) and no face: a record
//! nobody can read back is a record nobody checks. One document, three
//! faces — `ce erase --log`, the MCP tool `erase_log`, the GUI erase
//! screen's log section — and this module never writes. A line it
//! cannot read is COUNTED and named by line number, never skipped in
//! silence: the file is the apply contract's evidence, not telemetry.

use crate::erase::model::LOG_SCHEMA;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// The reader's document schema; bump on shape change (plan §7.1).
/// Named for the TRAIL, not the record: the registry keys report ids
/// by family name and `erase-log` is the record schema's (LOG_SCHEMA).
pub const REPORT_SCHEMA: &str = "ce.erase-trail-report/0.1.0";

/// Where apply.rs appends, root-relative (append_log).
pub const LOG_REL: &str = ".ce/erase-log.ndjson";

/// One record exactly as apply.rs writes it — the same eight fields,
/// read strictly: a writer that drifted is a NAMED unreadable line,
/// never a silently partial row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// One line → one record, or the reason it is not one.
fn record(line: &str) -> std::result::Result<Record, String> {
    let r: Record = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if r.schema != LOG_SCHEMA {
        return Err(format!("schema {:?} is not {LOG_SCHEMA}", r.schema));
    }
    Ok(r)
}

/// The document every face renders: the records, the refused lines
/// by number, and the counts a reader checks first.
pub fn report_json(l: &Log) -> Value {
    let mut by_class: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &l.rows {
        *by_class.entry(r.class.as_str()).or_insert(0) += 1;
    }
    json!({
        "schema": REPORT_SCHEMA,
        "log": LOG_REL,
        "present": l.present,
        "rows": l.rows,
        "unreadable": l
            .unreadable
            .iter()
            .map(|(line, why)| json!({"line": line, "why": why}))
            .collect::<Vec<_>>(),
        "counts": {
            "rows": l.rows.len(),
            "unreadable": l.unreadable.len(),
            "by_class": by_class,
        },
    })
}

/// The console face: one line per record (UTC stamp, class, target,
/// provenance, plan hash), every refused line by number, one summary.
pub fn print(l: &Log, as_json: bool) {
    if as_json {
        println!("{}", report_json(l));
        return;
    }
    for r in &l.rows {
        let span = match r.span {
            Some((s, e)) => format!(":{s}-{e}"),
            None => String::new(),
        };
        println!(
            "{} {} {}{} ({}) plan {}",
            utc_stamp(r.ts_ms),
            r.class,
            r.path,
            span,
            r.provenance,
            r.plan
        );
    }
    for (line, why) in &l.unreadable {
        println!(
            "{}",
            crate::i18n::line("unreadable line {}: {}", "第 {} 行不可读：{}", &[line, why])
        );
    }
    let summary = if l.present {
        crate::i18n::line(
            "erase log: {} record(s) in {} ({} unreadable)",
            "擦除日志：{} 条记录在 {}（{} 行不可读）",
            &[&l.rows.len(), &LOG_REL, &l.unreadable.len()],
        )
    } else {
        crate::i18n::line(
            "erase log: no {} yet — nothing has been applied here",
            "擦除日志：尚无 {}——这里还没执行过擦除",
            &[&LOG_REL],
        )
    };
    println!("{summary}");
}

/// `YYYY-MM-DDTHH:MM:SSZ` from epoch milliseconds. The record stores
/// the integer (the machine face); this is the console's reading of
/// it, and the GUI derives the same string from the same integer.
/// The crate carries no date dependency, so the civil-date arithmetic
/// is written once here.
pub fn utc_stamp(ms: u64) -> String {
    let secs = ms / 1000;
    let (y, m, d) = civil((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Days since 1970-01-01 → proleptic-Gregorian (year, month, day)
/// (Howard Hinnant's `civil_from_days`, integer-exact).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
#[path = "../../tests/unit/erase/log.rs"]
mod tests;
