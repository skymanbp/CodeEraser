//! SARIF 2.1.0 envelope — the shared skeleton under every family's
//! `--format sarif` projection (plan v2.14: the 2026-08-19 ruling
//! retired INTERPRETATION from the CLI; SARIF is another ENCODING of
//! the same judged facts, so the ruling stands unoffended). Pure
//! projection by construction: the builders re-spell findings that
//! arrive already judged — no verdict, no wire, no policy lives
//! here. A machine face like the report JSONs: never translated
//! (i18n.rs charter).

use serde_json::{Value, json};

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
