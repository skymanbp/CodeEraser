//! Shared report reading: the bound-document readers and the graph
//! faces' one exit road. The `--format` gate, the console's
//! named-failure suffix and the pair envelope the clone and docdup
//! reports shared left for the core with the consoles (plan v2.32
//! step 5: CE.Text.Check / CE.Text.Scan `failed`, CE.Document.Envelope).

/// A family whose document is read back into a typed reader (plan
/// v2.32 step 4): the reader keeps the bound document beside it. Since
/// step 5 the console is the core's lines (`document::emit`); the
/// readers serve the library's own callers. `bound!` writes the impl.
pub trait Bound: serde::de::DeserializeOwned {
    fn keep(&mut self, doc: serde_json::Value);
}

macro_rules! bound {
    ($t:ty) => {
        impl crate::report::Bound for $t {
            fn keep(&mut self, doc: serde_json::Value) {
                self.doc = doc;
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

/// A `ce graph` face over a document the core lays out (plan v2.32
/// steps 4-5): its lines on the console, the document itself under
/// --format json; any failure is exit 2, named `ce graph:`. These
/// families state no veto.
pub fn graph_face(
    family: &str,
    answer: anyhow::Result<crate::document::Answer>,
    json: bool,
) -> std::process::ExitCode {
    match answer.and_then(|a| crate::document::emit(family, &a, json)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ce graph: {err:#}");
            std::process::ExitCode::from(2)
        }
    }
}
