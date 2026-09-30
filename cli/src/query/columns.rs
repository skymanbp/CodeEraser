//! The goals' column names (design booklet §4.5): the core answers a
//! goal's tuples by position and its sorts by column, never a name —
//! the names are the program's own text. A query's columns are the
//! variables its body binds, in the order the core's safety walk
//! binds them (CE.Query.Check.Safety.outerVars: a positive atom's
//! new variables in term order, a `Var = …` binding its variable;
//! `not`, comparisons and an aggregate's body bind nothing outward);
//! an assertion's columns are its head's terms as written.

use super::lexer::Kind;
use super::program::{Program, Token};

/// One goal as the program spelled it, in goal order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalHead {
    pub clause: usize,
    /// `query` or `assert`.
    pub kind: &'static str,
    /// An assertion's predicate name.
    pub name: Option<String>,
    pub columns: Vec<String>,
}

/// Every goal clause's head, in clause order (the core's goal index).
pub fn goals(p: &Program) -> Vec<GoalHead> {
    clauses(&p.tokens)
        .into_iter()
        .enumerate()
        .filter_map(|(clause, toks)| head_of(clause, toks))
        .collect()
}

/// The token slices of the clauses: split at every depth-0 `.`, each
/// `.` kept with its clause.
fn clauses(tokens: &[Token]) -> Vec<&[Token]> {
    split(tokens, ".", true)
}

/// The slices between depth-0 occurrences of `at` (parentheses nest),
/// the separator kept with the slice before it when `keep`; the tail
/// after the last separator is a slice when it holds a token.
fn split<'a>(toks: &'a [Token], at: &str, keep: bool) -> Vec<&'a [Token]> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0usize, 0usize);
    for (i, t) in toks.iter().enumerate() {
        match t.spell.as_str() {
            "(" => depth += 1,
            ")" => depth = depth.saturating_sub(1),
            s if s == at && depth == 0 => {
                out.push(&toks[start..if keep { i + 1 } else { i }]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < toks.len() {
        out.push(&toks[start..]);
    }
    out
}

fn head_of(clause: usize, toks: &[Token]) -> Option<GoalHead> {
    let first = toks.first()?;
    match first.spell.as_str() {
        "?-" => Some(GoalHead {
            clause,
            kind: "query",
            name: None,
            columns: outer_vars(body(&toks[1..])),
        }),
        "assert" => {
            let head_end = toks
                .iter()
                .position(|t| t.spell == ":-")
                .unwrap_or(toks.len());
            let head = &toks[1..head_end];
            Some(GoalHead {
                clause,
                kind: "assert",
                name: head.first().map(|t| t.spell.clone()),
                columns: head
                    .iter()
                    .filter(|t| t.kind < 10)
                    .skip(1)
                    .map(spelled)
                    .collect(),
            })
        }
        _ => None,
    }
}

/// A term as the head wrote it; an anonymous one reads `_`.
fn spelled(t: &Token) -> String {
    if t.kind == Kind::Anon.code() {
        "_".into()
    } else {
        t.spell.clone()
    }
}

/// The body without its closing `.`.
fn body(toks: &[Token]) -> &[Token] {
    match toks.last() {
        Some(t) if t.spell == "." => &toks[..toks.len() - 1],
        _ => toks,
    }
}

/// The literals of a body: split at every depth-0 `,`.
fn literals(toks: &[Token]) -> Vec<&[Token]> {
    split(toks, ",", false)
}

/// The variables a body binds outward, by first binding.
fn outer_vars(toks: &[Token]) -> Vec<String> {
    let mut bound: Vec<String> = Vec::new();
    for lit in literals(toks) {
        let Some(first) = lit.first() else { continue };
        let binds: Vec<&Token> = if first.kind == Kind::Pred.code() {
            lit.iter().filter(|t| t.kind == Kind::Var.code()).collect()
        } else if first.kind == Kind::Var.code() && lit.get(1).is_some_and(|t| t.spell == "=") {
            vec![first]
        } else {
            Vec::new()
        };
        for t in binds {
            if !bound.contains(&t.spell) {
                bound.push(t.spell.clone());
            }
        }
    }
    bound
}

#[cfg(test)]
#[path = "../../tests/unit/query/columns.rs"]
mod tests;
