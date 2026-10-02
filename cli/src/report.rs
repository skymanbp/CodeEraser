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

/// A family whose report is its document read back (plan v2.32 step
/// 4): the document itself is the machine print, the reader's console
/// the rest. `bound!` writes the impl.
pub trait Bound: serde::de::DeserializeOwned {
    fn doc(&self) -> &serde_json::Value;
    fn keep(&mut self, doc: serde_json::Value);
    fn console(&self);
}

macro_rules! bound {
    ($t:ty, $console:path) => {
        impl crate::report::Bound for $t {
            fn doc(&self) -> &serde_json::Value {
                &self.doc
            }
            fn keep(&mut self, doc: serde_json::Value) {
                self.doc = doc;
            }
            fn console(&self) {
                $console(self)
            }
        }
    };
}
pub(crate) use bound;

/// The bound document read back, the document kept beside it.
pub fn read_bound<T: Bound>(doc: serde_json::Value, family: &str) -> anyhow::Result<T> {
    let mut r: T = crate::document::read(&doc, family)?;
    r.keep(doc);
    Ok(r)
}

/// `[code, value]` rows as the console's `code:value` list.
pub fn colon_pairs(rows: &[[i64; 2]]) -> String {
    let pairs: Vec<String> = rows.iter().map(|[c, v]| format!("{c}:{v}")).collect();
    pairs.join(" ")
}

/// The document under --format json, the console otherwise.
pub fn print_bound<T: Bound>(r: &T, as_json: bool) {
    print_doc(as_json, || r.doc().clone(), || r.console());
}

/// A `ce graph` face over a document the core lays out (plan v2.32
/// step 4): the document itself under --format json, the reader `T`
/// to the console otherwise; any failure is exit 2, named.
pub fn print_read<T: serde::de::DeserializeOwned>(
    doc: anyhow::Result<serde_json::Value>,
    json: bool,
    family: &str,
    console: impl FnOnce(T),
) -> std::process::ExitCode {
    let read = doc.and_then(|doc| {
        if !json {
            console(crate::document::read(&doc, family)?);
        } else {
            println!("{doc}");
        }
        Ok(())
    });
    match read {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ce graph: {err:#}");
            std::process::ExitCode::from(2)
        }
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
