//! The one place a face writes the console text the core wrote (plan
//! v2.32 step 5, ruling R3; design booklet
//! docs/reference/authority-track.md §6): a `document.result` carries
//! `lines` — `[stream, text, reference…]`, stream 0 stdout and 1
//! stderr, every `{}` hole of the text filled left to right from the
//! references, which bind through the same `Resolve` as the document —
//! and `exit {fail}`, the face's veto. A hole without its reference, a
//! reference without its hole, or a reference the face cannot resolve
//! is an error by name, never a line with a silent gap.

use super::Resolve;
use anyhow::{Result, anyhow, bail};
use serde_json::Value;

/// The proto whose replies carry `lines` and `exit`.
const SINCE: &str = "7.10.0";

/// The language the core writes a request's lines in: 1 = zh, as the
/// console reads it (i18n::zh), else 0.
pub fn lang() -> u8 {
    u8::from(crate::i18n::zh())
}

/// Where a line goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// One bound line: its stream and its text, every hole filled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub stream: Stream,
    pub text: String,
}

/// What a face prints: every line (the console face), or only the
/// stream-1 lines (a face whose stdout is the document itself).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Console,
    Document,
}

/// The reply's lines, bound through `r`, and its `exit.fail`. A reply
/// without `lines` / `exit` came from a core older than the contract.
pub fn bind_lines(reply: &Value, r: &dyn Resolve) -> Result<(Vec<Line>, bool)> {
    let (lines, fail) = carried(reply)?;
    let lines = lines
        .iter()
        .enumerate()
        .map(|(i, l)| bound(l, r).map_err(|e| anyhow!("document: line {i}: {e}")))
        .collect::<Result<_>>()?;
    Ok((lines, fail))
}

/// The reply's lines as the core spelled them (`[stream, text]`, every
/// hole filled there) and its `exit.fail`.
pub fn spelled_lines(reply: &Value) -> Result<(Vec<Line>, bool)> {
    let (lines, fail) = carried(reply)?;
    let line = |(i, l): (usize, &Value)| {
        match l.as_array().map(Vec::as_slice) {
            Some([s, Value::String(text)]) => stream_of(s).map(|stream| Line {
                stream,
                text: text.clone(),
            }),
            _ => None,
        }
        .ok_or_else(|| anyhow!("document: line {i} is not [stream, text]: {l}"))
    };
    let lines = lines.iter().enumerate().map(line).collect::<Result<_>>()?;
    Ok((lines, fail))
}

/// The reply's `lines` and `exit.fail`; a reply without them came from
/// a core older than the contract.
fn carried(reply: &Value) -> Result<(&Vec<Value>, bool)> {
    match (reply["lines"].as_array(), reply["exit"]["fail"].as_bool()) {
        (Some(lines), Some(fail)) => Ok((lines, fail)),
        _ => bail!("document: the reply carries no lines / exit (a pre-{SINCE} core)"),
    }
}

/// A line's stream code: 0 stdout, 1 stderr.
fn stream_of(code: &Value) -> Option<Stream> {
    match code.as_u64() {
        Some(0) => Some(Stream::Out),
        Some(1) => Some(Stream::Err),
        _ => None,
    }
}

/// One `[stream, text, reference…]`, its holes filled.
fn bound(l: &Value, r: &dyn Resolve) -> Result<Line> {
    let Some([stream, text, refs @ ..]) = l.as_array().map(Vec::as_slice) else {
        bail!("not [stream, text, reference…]: {l}");
    };
    let Some(stream) = stream_of(stream) else {
        bail!("stream {stream} is not 0 or 1");
    };
    let Some(text) = text.as_str() else {
        bail!("text {text} is not a string");
    };
    let holes = text.matches("{}").count();
    if holes != refs.len() {
        bail!("{holes} hole(s) and {} reference(s)", refs.len());
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    for reference in refs {
        let is_ref = reference
            .as_object()
            .is_some_and(|o| o.len() == 1 && o.contains_key("$"));
        let (true, Value::String(s)) = (is_ref, super::bind(reference.clone(), r)?) else {
            bail!("{reference} is not a reference");
        };
        let (head, tail) = rest
            .split_once("{}")
            .expect("one hole per reference, counted");
        out.push_str(head);
        out.push_str(&s);
        rest = tail;
    }
    out.push_str(rest);
    Ok(Line { stream, text: out })
}

/// Every line `mode` prints, to its stream, in order.
pub fn print(lines: &[Line], mode: Mode) {
    for l in lines {
        match (l.stream, mode) {
            (Stream::Err, _) => eprintln!("{}", l.text),
            (Stream::Out, Mode::Console) => println!("{}", l.text),
            (Stream::Out, Mode::Document) => {}
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/document/lines.rs"]
mod tests;
