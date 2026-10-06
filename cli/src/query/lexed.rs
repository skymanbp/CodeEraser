//! The lexed program as the core reads it back (plan v2.33 W1): the
//! scanner, the goal heads and the predicates' names are the core's
//! (query/1 `lex`, CE.Query.Lex and CE.Query.Front); this side sends the
//! three texts — the built-in prelude, the rules file, the ad hoc query —
//! and keeps what it labels by: every token's `where line:column`, the
//! predicate names by code, the globs and where each was spelled, the
//! names by hash, the fact tables the program reads, and every goal's
//! head as spelled. A lexical fault comes back as its place and what.

use super::wire;
use crate::corelink::Link;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One goal as the program spelled it, in goal order: an assertion's
/// predicate name (a query has none) and its column names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GoalHead {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

/// The lexed program, as the core answered it.
#[derive(Debug, Default, Deserialize)]
pub struct Lexed {
    pub tokens: usize,
    pub clauses: usize,
    pub prelude: usize,
    at: Vec<String>,
    preds: Vec<(i64, String)>,
    pub sets: Vec<String>,
    #[serde(rename = "setAt")]
    pub set_at: Vec<String>,
    names: Vec<(u64, String)>,
    pub referenced: BTreeSet<u32>,
    pub heads: Vec<GoalHead>,
}

/// The core's answer to a lex request: the program, or the fault that
/// stopped it (`where line:column`, what).
pub enum Lex {
    Program(Lexed),
    Fault(String, String),
}

/// The three texts in wire order (`null` for an absent source).
pub fn texts(prelude: &str, rules: Option<&str>, query: Option<&str>) -> Value {
    json!([prelude, rules, query])
}

/// The program lexed by the core over `link`.
pub fn lex(link: &mut Link, texts: &Value) -> Result<Lex, String> {
    let reply = wire::ask(link, json!({"texts": texts, "lex": true}))?;
    let lexed = &reply["lexed"];
    if let Some([at, what]) = lexed["fault"].as_array().map(Vec::as_slice) {
        let text = |v: &Value| v.as_str().map(str::to_string);
        return match (text(at), text(what)) {
            (Some(at), Some(what)) => Ok(Lex::Fault(at, what)),
            _ => Err("wire skew: a lexical fault is [place, what]".into()),
        };
    }
    let p: Lexed = serde_json::from_value(lexed.clone())
        .map_err(|e| format!("wire skew: the lexed program: {e}"))?;
    if p.at.len() != p.tokens || p.set_at.len() != p.sets.len() {
        return Err("wire skew: the lexed program's tables disagree with its counts".into());
    }
    Ok(Lex::Program(p))
}

impl Lexed {
    /// Where a token index the core reported sits in the text; the
    /// index past the last token is the program's end.
    pub fn locate(&self, idx: usize) -> String {
        self.at
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "end of program".to_string())
    }

    /// A predicate's name by code: the schema's, or the program's own.
    pub fn pred_name(&self, code: i64) -> String {
        self.preds
            .iter()
            .find(|(c, _)| *c == code)
            .map_or_else(|| format!("pred#{code}"), |(_, n)| n.clone())
    }

    /// The fact schema's codes by name (the codes below the program's).
    pub fn schema(&self) -> BTreeMap<String, u32> {
        self.preds
            .iter()
            .filter_map(|(c, n)| Some((n.clone(), u32::try_from(*c).ok().filter(|c| *c < 1000)?)))
            .collect()
    }

    /// Every name the program spelled, by hash.
    pub fn names(&self) -> impl Iterator<Item = (u64, &String)> {
        self.names.iter().map(|(h, n)| (*h, n))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/query/lexed.rs"]
mod tests;
