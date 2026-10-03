//! Shared report emission for the families whose console this side
//! still prints: the one `--format` gate, the bound-document readers
//! and the console's named-failure suffix. The pair envelope the clone
//! and docdup reports shared left for the core (plan v2.32 step 5,
//! CE.Document.Envelope).

/// The one --format gate for families with their own reader (join,
/// trend): print the JSON document or run
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
