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
//! error by name, never a document printed from this side.

use crate::corelink::Link;
use anyhow::{Context, Result, anyhow, bail};
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
}

impl Request {
    pub fn new(family: &'static str) -> Self {
        Request {
            family,
            ranges: Map::new(),
            rows: Map::new(),
            facts: Map::new(),
            degraded: None,
        }
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

    /// The judgment did not happen; `why` indexes the reason texts.
    pub fn degraded(mut self, why: usize) -> Self {
        self.degraded = Some(why);
        self
    }

    pub(crate) fn body(self) -> Value {
        json!({
            "family": self.family, "ranges": self.ranges, "rows": self.rows,
            "facts": self.facts, "degraded": self.degraded,
        })
    }
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

/// The request answered by a fresh core at `core`, bound through `r`.
pub fn assemble(core: &str, req: Request, r: &dyn Resolve) -> Result<Value> {
    assemble_over(core, Err(String::new()), req, r)
}

/// The request answered over `held` when it is a whole link, else over
/// a fresh one to `core`, and bound through `r`.
pub fn assemble_over(core: &str, held: Held, req: Request, r: &dyn Resolve) -> Result<Value> {
    let family = req.family;
    let named = |why: String| anyhow!("{family} document: {why}");
    let mut link = match held {
        Ok(link) => link,
        Err(_) => open(core).map_err(named)?,
    };
    if !link.has(CAP) {
        return Err(named(format!("core offers no {CAP} (pre-{SINCE})")));
    }
    let mut reply = link.request(KIND, req.body()).map_err(named)?;
    if reply["degraded"] != Value::Bool(false) {
        let reason = reply["reason"].as_str().unwrap_or("degraded");
        bail!("{family} document: the core did not lay it out: {reason}");
    }
    bind(reply["document"].take(), r)
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

    /// The text a `why` reference names.
    pub fn at(&self, i: &[i128]) -> Option<String> {
        at(&self.0, i)
    }
}

/// The string at `i` of `list`, for a resolver.
pub fn at(list: &[String], i: &[i128]) -> Option<String> {
    match i {
        [i] => usize::try_from(*i).ok().and_then(|i| list.get(i)).cloned(),
        _ => None,
    }
}

/// Each string's place in their joint sort order (ties share one).
pub fn ranks<'a>(strings: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let all: Vec<&str> = strings.into_iter().collect();
    let mut sorted = all.clone();
    sorted.sort_unstable();
    all.iter()
        .map(|s| sorted.partition_point(|x| x < s))
        .collect()
}

#[cfg(test)]
#[path = "../tests/unit/document.rs"]
mod tests;
