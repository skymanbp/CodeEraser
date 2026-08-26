//! Shared primitives for the GENERATED bench surfaces (the render
//! tests): one owner for the parsed contract, field access, the
//! latest-version read, the measured-at label and the
//! iterate-a-table shape — the dedup gate caught every one of them
//! growing a twin when the dashboard renderer landed beside the
//! chip renderer.

use serde_json::Value;

/// contracts/bench/bench.json, parsed.
pub fn doc() -> Value {
    let text = std::fs::read_to_string(super::bench_path()).expect("bench.json");
    serde_json::from_str(&text).expect("bench.json parses")
}

/// String field or empty — absent fields render blank, never panic.
pub fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// The newest series version present (rows are sorted by version).
pub fn latest(d: &Value) -> &str {
    d["rows"]
        .as_array()
        .and_then(|rows| rows.last())
        .map(|row| s(row, "version"))
        .unwrap_or("")
}

/// The measured-at label with the dirty suffix — one spelling on
/// every surface (dates are digits and dashes, safe in md and html).
pub fn measured(row: &Value) -> String {
    format!(
        "{}{}",
        s(row, "measured_at"),
        if row["dirty"] == Value::Bool(true) {
            " (dirty)"
        } else {
            ""
        }
    )
}

/// Concatenate one rendered line per row of `d[key]` — the table
/// shape every generated surface repeats.
pub fn rows_with(d: &Value, key: &str, f: impl Fn(&Value) -> String) -> String {
    d[key].as_array().expect(key).iter().map(f).collect()
}
