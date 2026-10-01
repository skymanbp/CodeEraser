//! The one document of `ce query`, `ce rules`, the MCP tools and the
//! GUI screen (design booklet §4.6): the program as lexed, every
//! goal with its columns and sorts, every answer labelled through the
//! request's own tables, every proof row named, every error at its
//! `where line:column`, the core's counts, and — when the core did
//! not judge — the named reason. The core judges (query/1, wire.rs)
//! and lays the document out (document/1, CE.Query.Document); this
//! side sends the program's facts, its own faults (a lexical one, a
//! glob it could not read), the goals as spelled and the core's
//! tables back, and puts the positions, names and values back
//! (crate::document). A document with a `degraded` reason carries no
//! verdict this side reached.

use super::columns::{self, GoalHead};
use super::facts::{self, Labels};
use super::program::Program;
use super::{PRELUDE, wire};
use crate::document::{self, Held, Request, Resolve, Why};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// What a face asks: a rules file (its label and text), a question,
/// whether queries carry proofs. No question = the rules document.
pub struct Ask {
    pub rules: Option<(String, String)>,
    pub query: Option<String>,
    pub why: bool,
}

/// The document request's tables.
const TABLES: [&str; 7] = [
    "faults", "heads", "goals", "preds", "answers", "proof", "errors",
];

/// The two documents' schema ids (CE.Query.Document); read here by the
/// facts registry and the tests.
pub const SCHEMA_ID: &str = "ce.query-report/0.1.0";
pub const RULES_SCHEMA_ID: &str = "ce.rules-report/0.1.0";

/// A question as the user typed it, in query form: `?-` in front and
/// `.` behind unless written.
pub fn question(text: &str) -> String {
    let t = text.trim();
    let head = if t.starts_with("?-") { "" } else { "?- " };
    let tail = if t.ends_with('.') { "" } else { "." };
    format!("{head}{t}{tail}")
}

/// The whole leg: lexed, assembled, judged, laid out.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, ask: &Ask) -> Result<Value> {
    let query = ask.query.as_deref().map(question);
    let rules = ask.rules.as_ref().map(|(_, text)| text.as_str());
    let mut names = Names {
        rules_file: ask.rules.as_ref().map(|(label, _)| label.clone()),
        query: query.clone(),
        program: None,
        heads: Vec::new(),
        labels: None,
        why: Why::default(),
    };
    let family = if ask.query.is_some() {
        "query"
    } else {
        "rules"
    };
    let mut req = Request::new(family)
        .fact("askWhy", u8::from(ask.why))
        .fact("rulesFile", u8::from(names.rules_file.is_some()))
        .fact("query", u8::from(query.is_some()));
    let mut held = Err(String::new());
    let ground = Ground {
        root,
        db,
        core,
        why: ask.why,
    };
    match Program::lex(PRELUDE, rules, query.as_deref()) {
        Ok(program) => (req, held) = judge(ground, program, &mut names, req)?,
        Err(f) => {
            let at = format!("{} {}:{}", f.src.word(), f.line, f.col);
            req = req.rows("faults", [[names.why.add(at), names.why.add(f.what)]]);
        }
    }
    document::assemble_over(core, held, finish(req, &names), &names)
}

/// Every table and fact the road did not fill, empty or zero, and the
/// two ranges: the reason texts, and the token positions an error may
/// name (the stream's length plus one, the end of input).
fn finish(req: Request, names: &Names) -> Request {
    let at = names.program.as_ref().map_or(0, |p| p.tokens.len() + 1);
    req.empty(&TABLES)
        .zero(&[
            "tokens",
            "clauses",
            "prelude",
            "askWhy",
            "rulesFile",
            "query",
        ])
        .zero(&wire::COUNTS)
        .range("why", names.why.count())
        .range("at", at)
}

/// Where a program is judged: the tree, its index, the core, and
/// whether the queries carry their proofs.
struct Ground<'a> {
    root: &'a Path,
    db: Option<PathBuf>,
    core: &'a str,
    why: bool,
}

/// A lexed program: its faults, or the core's judgment of it, with the
/// link it was judged over (spent when the request failed).
fn judge(g: Ground, program: Program, names: &mut Names, req: Request) -> Result<(Request, Held)> {
    let req = req
        .fact("tokens", program.tokens.len())
        .fact("clauses", program.clauses)
        .fact("prelude", program.prelude_clauses);
    let faults = glob_faults(g.root, &program);
    if !faults.is_empty() {
        let rows: Vec<[usize; 2]> = faults
            .into_iter()
            .map(|(at, what)| [names.why.add(at), names.why.add(what)])
            .collect();
        names.program = Some(program);
        return Ok((req.rows("faults", rows), Err(String::new())));
    }
    let facts = facts::assemble(g.root, g.db, g.core, &program)?;
    let body = wire::body(
        &program.wire(),
        &facts.tables,
        program.prelude_clauses,
        g.why,
        false,
    );
    let mut held = document::open(g.core);
    let reply = match &mut held {
        Ok(link) => wire::ask(link, body),
        Err(why) => Err(why.clone()),
    };
    names.heads = columns::goals(&program);
    names.labels = Some(facts.labels);
    let tokens = program.tokens.len();
    names.program = Some(program);
    let reply = match reply {
        Ok(reply) => reply,
        Err(why) => return Ok((req.degraded(names.why.add(why)), Err(String::new()))),
    };
    let j = wire::consume(&reply, tokens).map_err(|e| anyhow!("{e}"))?;
    Ok((answered(req, names, j)?, held))
}

/// The core's answer: its counts, then its named reason or its tables.
fn answered(req: Request, names: &mut Names, j: wire::Judged) -> Result<Request> {
    let req = j.counts.iter().fold(req, |q, (k, n)| q.fact(k, *n));
    if let Some(why) = j.degraded {
        return Ok(req.degraded(names.why.add(why)));
    }
    super::rows::judged(req, &names.heads, &j)
}

/// Every glob the program spelled, compiled through the exclude
/// list's dialect before any table is built: a glob it cannot read
/// is a program error at the glob's token, as (position, message).
fn glob_faults(root: &Path, program: &Program) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (id, glob) in program.sets.iter().enumerate() {
        let Err(why) =
            crate::scan::globs::compile_inclusions(root, std::slice::from_ref(glob), "glob")
        else {
            continue;
        };
        let at = program
            .tokens
            .iter()
            .position(|t| t.kind == super::lexer::Kind::Set.code() && t.value == id as i128)
            .map_or_else(|| "?".to_string(), |i| program.locate(i));
        out.push((at, why.trim_start_matches("ce.toml ").to_string()));
    }
    out
}

/// The query document's strings: the positions, the goals' names and
/// columns, the values under their sorts, the predicate names, the
/// rules file, the question, and this side's own texts.
struct Names {
    rules_file: Option<String>,
    query: Option<String>,
    program: Option<Program>,
    heads: Vec<GoalHead>,
    labels: Option<Labels>,
    why: Why,
}

impl Names {
    fn head(&self, g: i128) -> Option<&GoalHead> {
        usize::try_from(g).ok().and_then(|g| self.heads.get(g))
    }

    /// The classes the program holds: a token's place, a predicate.
    fn of_program(&self, class: &str, ints: &[i128]) -> Option<String> {
        let program = self.program.as_ref()?;
        match (class, ints) {
            ("at", [t]) => Some(program.locate(usize::try_from(*t).ok()?)),
            ("pred", [p]) => Some(program.pred_name(i64::try_from(*p).ok()?)),
            _ => None,
        }
    }

    /// The classes the goals and their answers hold.
    fn of_goals(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("goal_name", [g]) => self.head(*g)?.name.clone(),
            ("column", [g, i]) => self
                .head(*g)?
                .columns
                .get(usize::try_from(*i).ok()?)
                .cloned(),
            ("value", [s, v]) => Some(self.labels.as_ref()?.render(i64::try_from(*s).ok()?, *v)),
            _ => None,
        }
    }
}

impl Resolve for Names {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("why", _) => self.why.at(ints),
            ("rules_file", []) => self.rules_file.clone(),
            ("query", []) => self.query.clone(),
            _ => self
                .of_program(class, ints)
                .or_else(|| self.of_goals(class, ints)),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/query/face.rs"]
mod tests;
