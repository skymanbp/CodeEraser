//! The report documents the core lays out (plan v2.32 step 3; design
//! booklet docs/reference/authority-track.md §5): a face sends the
//! integer tables its judgment answered, the facts it measured and the
//! size of every universe its document refers into, as one
//! `document.request` (`document/1`, proto 7.8.0); the core answers the
//! whole document with every repository string a reference
//! `{"$": [class, integers…]}`, and `bind` puts the strings back
//! through the face's own `Resolve`. The fields, their order, the
//! counts and the schema id are the core's; this side holds the
//! strings. A judgment that did not happen is asked too, `degraded`
//! naming the reason text, and a document the core does not answer —
//! no `document/1`, a refusal, a document over its row cap — is an
//! error by name, never a document printed from this side. Since 7.10.0
//! (plan v2.32 step 5) the reply also carries the console lines in the
//! language this process speaks and the face's veto: every face gets
//! one `Answer`, and `emit` prints it. Since plan v2.33 W7 a face sends
//! its strings too (`Request::text`, by reference class): the core
//! spells every reference and fills every hole itself, and this side
//! prints what it gets. Two faces still bind here — the PreToolUse
//! guard (guard/speech.rs, the hook path) and query (its program's
//! strings stay this side until the program crosses) — through
//! `assemble_bound` and the `Resolve` below.

pub mod lines;
mod paths;

pub use paths::Paths;

use crate::corelink::Link;
use anyhow::{Context, Result, anyhow, bail};
use lines::{Line, Mode};
use serde_json::{Map, Value, json};

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "document/1";
pub const KIND: &str = "document";
const SINCE: &str = "7.8.0";

/// A face's strings, by the class and the integers a reference
/// carries; None = a class it does not hold or an integer outside it.
pub trait Resolve {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String>;
}

/// One request: the family, its ranges, rows and facts, and the reason
/// text's index when the judgment did not happen.
pub struct Request {
    family: &'static str,
    ranges: Map<String, Value>,
    rows: Map<String, Value>,
    facts: Map<String, Value>,
    degraded: Option<usize>,
    strings: Option<Map<String, Value>>,
}

impl Request {
    pub fn new(family: &'static str) -> Self {
        Request {
            family,
            ranges: Map::new(),
            rows: Map::new(),
            facts: Map::new(),
            degraded: None,
            strings: None,
        }
    }

    /// The strings references of `class` name, one JSON array level
    /// per integer such a reference carries (a class with none: the
    /// string itself). The core spells them in.
    pub fn text(mut self, class: &str, strings: impl serde::Serialize) -> Self {
        let v = serde_json::to_value(strings).expect("strings serialize");
        self.strings
            .get_or_insert_with(Map::new)
            .insert(class.into(), v);
        self
    }

    /// A class's strings from an iterator, in its order.
    pub fn texts<T: serde::Serialize>(
        self,
        class: &str,
        items: impl IntoIterator<Item = T>,
    ) -> Self {
        self.text(class, items.into_iter().collect::<Vec<_>>())
    }

    /// Two classes from one pass: each row's pair, its halves in order.
    pub fn text_columns<A: serde::Serialize, B: serde::Serialize>(
        self,
        (a, b): (&str, &str),
        rows: impl IntoIterator<Item = (A, B)>,
    ) -> Self {
        let (left, right): (Vec<A>, Vec<B>) = rows.into_iter().unzip();
        self.text(a, left).text(b, right)
    }

    /// A face whose document names no string: still answered spelled.
    pub fn spelled(mut self) -> Self {
        self.strings.get_or_insert_with(Map::new);
        self
    }

    /// The size of the universe `class` refers into.
    pub fn range(mut self, name: &str, n: usize) -> Self {
        self.ranges.insert(name.into(), n.into());
        self
    }

    /// One table, its rows as the face holds them (any integer rows).
    pub fn rows(mut self, name: &str, rows: impl serde::Serialize) -> Self {
        let v = serde_json::to_value(rows).expect("integer rows serialize");
        self.rows.insert(name.into(), v);
        self
    }

    /// A one-row table `[[v]]` when the value is there, absent when
    /// not (a statement's optional single, `rows <t> 1 …`).
    pub fn single(self, name: &str, v: Option<impl serde::Serialize>) -> Self {
        match v {
            Some(v) => self.rows(name, [[v]]),
            None => self,
        }
    }

    /// Every table of `names` not yet given, sent with no row.
    pub fn empty(mut self, names: &[&str]) -> Self {
        for name in names {
            self.rows.entry(*name).or_insert_with(|| json!([]));
        }
        self
    }

    /// Every fact of `names` not yet given, sent as zero.
    pub fn zero(mut self, names: &[&str]) -> Self {
        for name in names {
            self.facts.entry(*name).or_insert_with(|| json!(0));
        }
        self
    }

    pub fn fact(mut self, name: &str, n: impl Into<Value>) -> Self {
        self.facts.insert(name.into(), n.into());
        self
    }

    /// A measuring side's counters as facts: `names` space-separated,
    /// one value each, in order.
    pub fn counters(self, names: &str, values: &[u64]) -> Self {
        let names: Vec<&str> = names.split_whitespace().collect();
        assert_eq!(names.len(), values.len(), "one value per counter name");
        names
            .into_iter()
            .zip(values)
            .fold(self, |req, (name, n)| req.fact(name, *n))
    }

    /// The judgment did not happen; `why` indexes the reason texts.
    pub fn degraded(mut self, why: usize) -> Self {
        self.degraded = Some(why);
        self
    }

    /// The request as it goes out; `lang` is this process's language,
    /// set here and nowhere else (ruling R2).
    pub(crate) fn body(self) -> Value {
        let mut body = json!({
            "family": self.family, "ranges": self.ranges, "rows": self.rows,
            "facts": self.facts, "degraded": self.degraded, "lang": lines::lang(),
        });
        if let Some(strings) = self.strings {
            body["strings"] = Value::Object(strings);
        }
        body
    }
}

/// What the core answered a face (ruling R1): the bound document, the
/// console lines in this process's language bound through the same
/// `Resolve`, and the face's veto. MCP and the GUI read `document`; the
/// CLI prints (`emit`) and exits 1 iff `fail`.
#[derive(Debug)]
pub struct Answer {
    pub document: Value,
    pub lines: Vec<Line>,
    pub fail: bool,
}

/// A face's link to the core: opened once, judged over, then laid out
/// over, so a run starts one core process. `Err` = the reason it could
/// not be opened, or a link a failed request spent (the pipe may hold
/// half a reply); the document then opens a fresh one.
pub type Held = std::result::Result<Link, String>;

/// A link to the core at `core`.
pub fn open(core: &str) -> Held {
    Link::open(core).map(|(link, _)| link)
}

/// The request answered by a fresh core at `core`, spelled.
pub fn assemble(core: &str, req: Request) -> Result<Answer> {
    assemble_over(core, Err(String::new()), req)
}

/// The request answered over `held` when it is a whole link, else over
/// a fresh one to `core`, the core spelling every string it carries:
/// the document and the lines as they print, and the veto.
pub fn assemble_over(core: &str, held: Held, req: Request) -> Result<Answer> {
    let req = req.spelled();
    let (family, mut reply) = asked(core, held, req)?;
    let (lines, fail) =
        lines::spelled_lines(&reply).map_err(|e| anyhow!("{family} document: {e:#}"))?;
    Ok(Answer {
        document: reply["document"].take(),
        lines,
        fail,
    })
}

/// The request answered over `held` (else a fresh link to `core`) with
/// its references left for this side, bound through `r` — query's
/// road, whose program's strings stay this side.
pub fn assemble_bound(core: &str, held: Held, req: Request, r: &dyn Resolve) -> Result<Answer> {
    let (family, mut reply) = asked(core, held, req)?;
    let named = |why: String| anyhow!("{family} document: {why}");
    let (lines, fail) = lines::bind_lines(&reply, r).map_err(|e| named(format!("{e:#}")))?;
    let document = bind(reply["document"].take(), r)?;
    Ok(Answer {
        document,
        lines,
        fail,
    })
}

/// The core's reply to `req`, refused by name when the core cannot lay
/// the document out.
fn asked(core: &str, held: Held, req: Request) -> Result<(&'static str, Value)> {
    let family = req.family;
    let named = |why: String| anyhow!("{family} document: {why}");
    let mut link = match held {
        Ok(link) => link,
        Err(_) => open(core).map_err(named)?,
    };
    if !link.has(CAP) {
        return Err(named(format!("core offers no {CAP} (pre-{SINCE})")));
    }
    let reply = link.request(KIND, req.body()).map_err(named)?;
    if reply["degraded"] != Value::Bool(false) {
        let reason = reply["reason"].as_str().unwrap_or("degraded");
        bail!("{family} document: the core did not lay it out: {reason}");
    }
    Ok((family, reply))
}

/// An answer printed as its face prints it (ruling R3): the console
/// face every line to its stream; the `--format json` face the document
/// on stdout — indented when the catalogue states the family `pretty` —
/// then its stream-1 lines (stdout is the document there).
pub fn emit(family: &str, answer: &Answer, json: bool) -> Result<()> {
    if !json {
        lines::print(&answer.lines, Mode::Console);
        return Ok(());
    }
    emit_projected(answer, &rendered(family, &answer.document)?);
    Ok(())
}

/// A family's document as its `--format json` face and its MCP tool
/// print it: indented when the catalogue states the family `pretty`.
pub fn rendered(family: &str, doc: &Value) -> Result<String> {
    let Some(pretty) = crate::tables::get().document.pretty(family) else {
        bail!("{family} document: the catalogue states no `pretty` (a pre-7.10.0 core)");
    };
    Ok(if pretty {
        serde_json::to_string_pretty(doc)?
    } else {
        doc.to_string()
    })
}

/// An answer whose stdout is a text made from its document (the
/// document itself, or its SARIF projection): the text, then the
/// stream-1 lines (ruling R3).
pub fn emit_projected(answer: &Answer, text: &str) {
    println!("{text}");
    lines::print(&answer.lines, Mode::Document);
}

/// A bound document read as the face's reader `T` (the console, the
/// exit); a face keeps the document itself beside it for the machine
/// print.
pub fn read<T: serde::de::DeserializeOwned>(doc: &Value, family: &str) -> Result<T> {
    T::deserialize(doc).with_context(|| format!("{family} document"))
}

/// Every reference replaced by its string; anything else as it is. A
/// reference is an object whose one key is `$`.
pub fn bind(v: Value, r: &dyn Resolve) -> Result<Value> {
    Ok(match v {
        Value::Array(xs) => {
            Value::Array(xs.into_iter().map(|x| bind(x, r)).collect::<Result<_>>()?)
        }
        Value::Object(o) if o.len() == 1 && o.contains_key("$") => resolved(&o["$"], r)?,
        Value::Object(o) => Value::Object(
            o.into_iter()
                .map(|(k, x)| Ok((k, bind(x, r)?)))
                .collect::<Result<_>>()?,
        ),
        other => other,
    })
}

fn resolved(reference: &Value, r: &dyn Resolve) -> Result<Value> {
    let parts = reference.as_array().filter(|p| !p.is_empty());
    let Some((class, ints)) = parts.and_then(|p| Some((p[0].as_str()?, &p[1..]))) else {
        bail!("document: a reference is not [class, integers…]: {reference}");
    };
    let ints: Vec<i128> = ints
        .iter()
        .map(|i| {
            i.as_i64()
                .map(i128::from)
                .or_else(|| i.as_u64().map(i128::from))
        })
        .collect::<Option<_>>()
        .ok_or_else(|| anyhow!("document: {reference} holds a non-integer"))?;
    r.resolve(class, &ints)
        .map(Value::String)
        .ok_or_else(|| anyhow!("document: no string for {reference}"))
}

/// This side's own texts a document names by index (the class `why`):
/// the reason a judgment did not happen, a fault's place and message,
/// a refusal's reason. One owner for the four faces' texts.
#[derive(Default)]
pub struct Why(Vec<String>);

impl Why {
    /// The text's index.
    pub fn add(&mut self, text: String) -> usize {
        self.0.push(text);
        self.0.len() - 1
    }

    /// The range the request declares for the class.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// The texts, for the request's strings (class `why`).
    pub fn list(&self) -> &[String] {
        &self.0
    }

    /// The text a `why` reference names.
    pub fn at(&self, i: &[i128]) -> Option<String> {
        at(&self.0, i)
    }
}

/// A face's strings held as lists, one per reference class, each read
/// by a one-integer reference — the resolver of every face whose
/// references are indices.
pub struct Lists(pub Vec<(&'static str, Vec<String>)>);

impl Resolve for Lists {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let (_, list) = self.0.iter().find(|(c, _)| *c == class)?;
        at(list, ints)
    }
}

/// The string at `i` of `list`, for a resolver.
pub fn at(list: &[String], i: &[i128]) -> Option<String> {
    match i {
        [i] => usize::try_from(*i).ok().and_then(|i| list.get(i)).cloned(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/unit/document.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/document/frozen/mod.rs"]
mod frozen;
