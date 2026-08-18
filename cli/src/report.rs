//! Shared judgment-report shapes and emission — ONE pair-hit shape,
//! ONE report envelope and ONE console posture for every judgment
//! family. The repo's own ratchet caught the second family re-growing
//! the first's structs and print function token for token (bite
//! seventeen); the generic forms exist so a third family cannot
//! re-grow them either.

use serde::Serialize;

/// One reported pair: the two endpoint names plus the family's own
/// metric block, flattened into the row's JSON.
#[derive(Serialize)]
pub struct Pair<M: Serialize> {
    pub a: String,
    pub b: String,
    #[serde(flatten)]
    pub m: M,
}

/// One family's judgment report: reported pairs + the counts ledger.
pub struct Report<M: Serialize, C: Serialize> {
    pub hits: Vec<Pair<M>>,
    pub counts: C,
}

/// The one --format gate for families whose Report does not fit the
/// Pair/counts mold (join, trend): print the JSON document or run
/// the console closure — the `if as_json {…; return}` skeleton was
/// the P4 ratchet's cross-family token twin.
pub fn print_doc(as_json: bool, doc: impl FnOnce() -> serde_json::Value, console: impl FnOnce()) {
    if as_json {
        println!("{}", doc());
    } else {
        console();
    }
}

/// Print one family's report: the JSON envelope `{schema, <key>,
/// counts}` under --format json, otherwise one templated line per
/// hit plus the counts flattened as `key value` pairs. The hit line
/// is a `{field}` template over the pair's serialized fields — the
/// family contributes DATA, never another print function.
pub fn emit<M: Serialize, C: Serialize>(
    head: (&str, &str),
    r: &Report<M, C>,
    as_json: bool,
    template: &str,
) {
    if as_json {
        println!("{}", envelope(head, r));
        return;
    }
    let (_schema, key) = head;
    for h in &r.hits {
        println!(
            "{}",
            render(template, &serde_json::to_value(h).expect("hit"))
        );
    }
    let v = serde_json::to_value(&r.counts).expect("counts");
    let parts: Vec<String> = v
        .as_object()
        .expect("counts object")
        .iter()
        .map(|(k, n)| format!("{k} {n}"))
        .collect();
    println!("{key}: {}", parts.join(", "));
}

/// The deadcode report as its wire JSON document — one serialization
/// for the CLI's --format json and the MCP report face (lifted out
/// of the binary at M7-P2 and housed with the other shared report
/// shapes; a second copy in a consumer is the drift the ratchet
/// bites).
pub fn deadcode_json(r: &crate::graph::deadcode::Report) -> serde_json::Value {
    use serde_json::json;
    json!({
        "schema": "ce.deadcode-report/0.1.0",
        "dead": r.dead.iter().map(|(n, v, w)| {
            json!({"name": n, "verdict": v, "why": w})
        }).collect::<Vec<_>>(),
        "reported": r.reported.iter().map(|(n, v)| {
            json!({"name": n, "verdict": v})
        }).collect::<Vec<_>>(),
        "counts": {"nodes": r.nodes, "kept_edges": r.kept},
        "unresolved_sites": r.unresolved_sites,
        "degraded": r.degraded,
    })
}

/// The JSON half of emit as a value — the MCP report face returns
/// this instead of printing, so the envelope stays one authority.
pub fn envelope<M: Serialize, C: Serialize>(
    (schema, key): (&str, &str),
    r: &Report<M, C>,
) -> serde_json::Value {
    let mut doc = serde_json::Map::new();
    doc.insert("schema".into(), schema.into());
    doc.insert(key.into(), serde_json::to_value(&r.hits).expect("hits"));
    doc.insert(
        "counts".into(),
        serde_json::to_value(&r.counts).expect("counts"),
    );
    serde_json::Value::Object(doc)
}

/// Substitute every `{field}` in the template with the object's
/// field, strings bare and numbers in decimal.
fn render(template: &str, v: &serde_json::Value) -> String {
    let mut out = template.to_string();
    for (k, val) in v.as_object().expect("hit object") {
        let s = match val {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out = out.replace(&format!("{{{k}}}"), &s);
    }
    out
}
