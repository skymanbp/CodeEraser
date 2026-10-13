//! SARIF 2.1.0 envelope — the shared skeleton under every family's
//! `--format sarif` projection (plan v2.14: the 2026-08-19 ruling
//! retired INTERPRETATION from the CLI; SARIF is another ENCODING of
//! the same judged facts, so the ruling stands unoffended). Pure
//! projection by construction: the builders re-spell findings that
//! arrive already judged — no verdict, no wire, no policy lives
//! here. A machine face like the report JSONs: never translated
//! (i18n.rs charter). The finding-shaped faces (`ce scan`, `ce dedup`)
//! print through `emit` below, the one road their three formats take.

use serde_json::{Value, json};

/// The finding-shaped faces' formats: beside console and json, sarif —
/// offered only by a command whose report carries physical locations
/// (sarif in the help of any other would be a promise nothing renders).
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Findings {
    Console,
    Json,
    Sarif,
}

/// A finding-shaped answer printed in `format`: the console lines, the
/// document, or the SARIF projection of its `key` rows by `finding` —
/// each followed by the stream-1 lines.
pub fn emit(
    family: &str,
    (key, finding): (&str, fn(&Value) -> Value),
    format: Findings,
    answer: &crate::document::Answer,
) -> anyhow::Result<()> {
    match format {
        Findings::Console => crate::document::emit(family, answer, false),
        Findings::Json => crate::document::emit(family, answer, true),
        Findings::Sarif => {
            let text = projected(&answer.document, key, finding)?;
            crate::document::emit_projected(answer, &text);
            Ok(())
        }
    }
}

/// The one report envelope, at the minimum GitHub code scanning —
/// the consuming face this revival bought — ingests: schema,
/// version, one run, one driver.
pub fn report(results: Vec<Value>) -> Value {
    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {
                "name": "CodeEraser",
                "informationUri": "https://codeeraser.dev",
                "version": env!("CARGO_PKG_VERSION"),
            }},
            "results": results,
        }]
    })
}

/// A bound document's `key` table projected one row to one finding
/// (`result` below), the envelope around them, printed indented — the
/// shape every family's `--format sarif` face shares.
pub fn projected(doc: &Value, key: &str, finding: fn(&Value) -> Value) -> anyhow::Result<String> {
    let rows = doc[key].as_array().map(Vec::as_slice).unwrap_or(&[]);
    Ok(serde_json::to_string_pretty(&report(
        rows.iter().map(finding).collect(),
    ))?)
}

/// A document row's integer field (0 when absent).
pub fn num(row: &Value, key: &str) -> usize {
    row[key].as_u64().unwrap_or(0) as usize
}

/// A document row's string field (empty when absent).
pub fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or("")
}

/// One finding. `level` is SARIF vocabulary ("error" / "warning" /
/// "note"), spelled by the projecting family from its own judged
/// grade; `related` locations ride only when non-empty (a clone
/// block's second span — most findings have none).
pub fn result(rule: &str, level: &str, text: &str, primary: Value, related: Vec<Value>) -> Value {
    let mut r = json!({
        "ruleId": rule,
        "level": level,
        "message": {"text": text},
        "locations": [primary],
    });
    if !related.is_empty() {
        r["relatedLocations"] = Value::Array(related);
    }
    r
}

/// A physical location; `end <= start` collapses to a single line.
pub fn location(uri: &str, start: usize, end: usize) -> Value {
    let region = if end > start {
        json!({"startLine": start, "endLine": end})
    } else {
        json!({"startLine": start})
    };
    json!({"physicalLocation": {
        "artifactLocation": {"uri": uri},
        "region": region,
    }})
}
