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
    let (schema, key) = head;
    if as_json {
        let mut doc = serde_json::Map::new();
        doc.insert("schema".into(), schema.into());
        doc.insert(key.into(), serde_json::to_value(&r.hits).expect("hits"));
        doc.insert(
            "counts".into(),
            serde_json::to_value(&r.counts).expect("counts"),
        );
        println!("{}", serde_json::Value::Object(doc));
        return;
    }
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
