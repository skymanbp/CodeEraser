//! The lexed program (design booklet §4.4): every source's tokens in
//! one stream — the built-in prelude first, then the rules file, then
//! the ad hoc query — and the tables the labelling side reads back
//! through: the program predicates by code, the globs by set id, the
//! names by hash, the variables by clause and number. `locate` turns
//! a token index the core reported back into `where line:column`,
//! the one text-shaped thing about an error.

use super::legend;
use super::lexer::{Kind, Lexer};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Where a token came from, in wire order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Prelude,
    Rules,
    Query,
}

const SOURCE_WORDS: [&str; 3] = ["prelude", "rules", "query"];

impl Source {
    pub fn word(self) -> &'static str {
        SOURCE_WORDS[self as usize]
    }
}

/// One token: its wire kind and value, where it came from, and the
/// spelling it stands for (a name, a number, a glob, a variable, a
/// punctuation) — the labelling side reads heads and errors by it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: i64,
    pub value: i128,
    pub src: Source,
    pub line: u32,
    pub col: u32,
    pub spell: String,
}

impl Token {
    /// `[kind, value]` as the wire carries it.
    pub fn wire(&self) -> Value {
        json!([self.kind, num(self.value)])
    }
}

/// A wire number: a hash rides as the unsigned word it is, an
/// integer literal as itself (the one kind that may be negative).
pub fn num(v: i128) -> Value {
    if v >= 0 {
        Value::from(v as u64)
    } else {
        Value::from(v as i64)
    }
}

/// A lexical fault: where, and what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub src: Source,
    pub line: u32,
    pub col: u32,
    pub what: String,
}

impl Fault {
    pub fn describe(&self) -> String {
        format!(
            "{} {}:{}: {}",
            self.src.word(),
            self.line,
            self.col,
            self.what
        )
    }
}

/// The lexed program and its legends.
#[derive(Default, Debug)]
pub struct Program {
    pub tokens: Vec<Token>,
    /// Clauses over every source, and how many of them are the prelude's.
    pub clauses: usize,
    pub prelude_clauses: usize,
    /// Program predicate names by code − 1000.
    pub preds: Vec<String>,
    /// Glob strings by set id.
    pub sets: Vec<String>,
    /// Every name constant spelled, by hash.
    pub names: BTreeMap<u64, String>,
    /// Per clause, its variable names by number (`_` for an anonymous one).
    pub vars: Vec<Vec<String>>,
}

impl Program {
    /// The three sources in wire order; the first lexical fault
    /// stops the whole program (the core never sees it).
    pub fn lex(prelude: &str, rules: Option<&str>, query: Option<&str>) -> Result<Program, Fault> {
        let mut p = Program::default();
        Lexer::new(&mut p, Source::Prelude, prelude).run()?;
        p.prelude_clauses = p.clauses;
        if let Some(text) = rules {
            Lexer::new(&mut p, Source::Rules, text).run()?;
        }
        if let Some(text) = query {
            Lexer::new(&mut p, Source::Query, text).run()?;
        }
        Ok(p)
    }

    /// A program predicate's code: by first appearance from 1000.
    pub(super) fn idb_code(&mut self, name: &str) -> u32 {
        legend::IDB_FLOOR + intern(&mut self.preds, name) as u32
    }

    /// A glob's set id: by first appearance from 0.
    pub(super) fn set_id(&mut self, glob: &str) -> i128 {
        intern(&mut self.sets, glob) as i128
    }

    /// The token stream as the request carries it.
    pub fn wire(&self) -> Vec<Value> {
        self.tokens.iter().map(Token::wire).collect()
    }

    /// Where a token index the core reported sits in the text; the
    /// index past the last token is the program's end.
    pub fn locate(&self, idx: usize) -> String {
        match self.tokens.get(idx) {
            Some(t) => format!("{} {}:{}", t.src.word(), t.line, t.col),
            None => "end of program".to_string(),
        }
    }

    /// The schema predicates the program reads — the fact tables the
    /// request must carry.
    pub fn referenced(&self) -> BTreeSet<u32> {
        self.tokens
            .iter()
            .filter(|t| t.kind == Kind::Pred.code() && t.value < i128::from(legend::IDB_FLOOR))
            .map(|t| t.value as u32)
            .collect()
    }

    /// A predicate's name by code: the schema's, or the program's own.
    pub fn pred_name(&self, code: i64) -> String {
        if let Some(p) = u32::try_from(code).ok().and_then(legend::pred_of) {
            return p.name.to_string();
        }
        usize::try_from(code - i64::from(legend::IDB_FLOOR))
            .ok()
            .and_then(|i| self.preds.get(i))
            .cloned()
            .unwrap_or_else(|| format!("pred#{code}"))
    }

    /// A clause's variable name by number.
    pub fn var_name(&self, clause: usize, n: usize) -> String {
        self.vars
            .get(clause)
            .and_then(|v| v.get(n))
            .cloned()
            .unwrap_or_else(|| format!("V{n}"))
    }
}

/// The index of `name` in `list`, appended when new: every
/// request-local identity is handed out by first appearance.
fn intern(list: &mut Vec<String>, name: &str) -> usize {
    list.iter().position(|x| x == name).unwrap_or_else(|| {
        list.push(name.to_string());
        list.len() - 1
    })
}

#[cfg(test)]
#[path = "../../tests/unit/query/program.rs"]
mod tests;
