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

/// The console's named-failure suffix (plan v2.18 step #14, O36):
/// the held conditions VERBATIM in the core's own order (Verdict.hs
/// failConditions — ratchet_over, discrete_added, floor,
/// dedup_budget, knobs_digest; degraded on that road), never sorted
/// or filtered, empty when nothing held so a pass line keeps its
/// bytes. `ratchet: … -> FAIL` used to exit 1 without saying that
/// `knobs_digest` alone was why; the JSON face always had the names.
/// Housed here, not in a family: the score AND scan consoles print
/// it, and scan importing score would be a module cycle the graph
/// axis itself bills.
pub fn fail_suffix(failed: &[String]) -> String {
    if failed.is_empty() {
        return String::new();
    }
    crate::i18n::line(" (failed: {})", "（失败条件：{}）", &[&failed.join(", ")])
}

/// Print one family's report: the JSON envelope `{schema, <key>,
/// counts}` under --format json, otherwise one templated line per
/// hit plus a summary SENTENCE over the counts — both `{field}`
/// templates, so the family contributes DATA, never another print
/// function. Counters the sentence omits still print in the raw
/// `k n` form after it (never silently absent, batch 9 P6) — the
/// raw tail shrinks as the sentence grows, and a new counter can
/// never vanish.
pub fn emit<M: Serialize, C: Serialize>(
    head: (&str, &str),
    r: &Report<M, C>,
    as_json: bool,
    template: &str,
    summary: &str,
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
    let rest: Vec<String> = v
        .as_object()
        .expect("counts object")
        .iter()
        .filter(|(k, _)| !summary.contains(&format!("{{{k}}}")))
        .map(|(k, n)| format!("{k} {n}"))
        .collect();
    let tail = if rest.is_empty() {
        String::new()
    } else {
        format!(" | {}", rest.join(", "))
    };
    println!("{key}: {}{tail}", render(summary, &v));
}

/// The deadcode report's schema id. 0.2.0 (2.32.0, H3): dead rows
/// carry the confidence column (null on a legacy reply without the
/// ledger); 0.3.0 (6.2.0): the `unmentioned` advisory rows,
/// `unmentioned_dropped` (the core dropped the table) and
/// `unmentioned_cut` (the producer cut the candidate set, so the rows
/// are a prefix), present exactly when the road was asked (K43) — a
/// document from a road that never asked carries none of the three,
/// so "not asked", "asked and clean", "cut" and "dropped" stay
/// distinct; 0.4.0 (plan v2.25, O23): dead rows carry `whyCode`
/// beside the English `why` — the code every face renders in its own
/// language. Named, not inline: the derived-fact registry (plan
/// v2.21) scans cli/src for value-shaped ids.
const DEADCODE_SCHEMA: &str = "ce.deadcode-report/0.4.0";

/// The deadcode report as its wire JSON document — one serialization
/// for the CLI's --format json and the MCP report face (lifted out
/// of the binary at M7-P2 and housed with the other shared report
/// shapes; a second copy in a consumer is the drift the ratchet
/// bites).
pub fn deadcode_json(r: &crate::graph::deadcode::Report) -> serde_json::Value {
    use crate::graph::deadcode::UnmentionedFace;
    use serde_json::json;
    let mut doc = json!({
        "schema": DEADCODE_SCHEMA,
        "dead": r.dead.iter().map(|d| {
            json!({"name": d.path, "verdict": d.verdict, "why": d.why(), "whyCode": d.why_code, "confidence": d.conf})
        }).collect::<Vec<_>>(),
        "reported": r.reported.iter().map(|(n, v)| {
            json!({"name": n, "verdict": v})
        }).collect::<Vec<_>>(),
        "counts": {"nodes": r.nodes, "kept_edges": r.kept},
        "unresolved_sites": r.unresolved_sites,
        "degraded": r.degraded,
    });
    if let Some(face) = &r.unmentioned {
        let (rows, dropped, cut) = match face {
            UnmentionedFace::Rows { rows, cut } => (rows.as_slice(), false, *cut),
            UnmentionedFace::Dropped => (&[][..], true, false),
        };
        let obj = doc.as_object_mut().expect("json! object literal");
        obj.insert(
            "unmentioned".into(),
            rows.iter()
                .map(|a| json!({"name": a.name, "symbol": a.symbol, "line": a.line, "code": a.code, "why": a.why}))
                .collect(),
        );
        obj.insert("unmentioned_dropped".into(), json!(dropped));
        obj.insert("unmentioned_cut".into(), json!(cut));
    }
    doc
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
