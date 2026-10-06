//! The facts the core's rungs ask of the measuring side (resolve/1, plan
//! v2.33 W2-text stage E; the Rust ops since stage F), read: each is a
//! question of the file system or of a file's syntax tree, answered as
//! its request row `[op, a, b, state, payload]` —
//!   op 0, a path's text: 0 no file, 1 a file that does not read as
//!   UTF-8, 2 its text;
//!   op 1, whether a path is a file;
//!   op 2, whether `node_modules/<b>` under the directory `a` is a
//!   directory;
//!   op 3, a path's TOML document: 0 no file, 1 a file that does not
//!   read as UTF-8 TOML, 2 the document decoded (a library read on this
//!   side, as the compile databases' JSON is: a TOML table is an
//!   object, a string a string);
//!   op 4, a Rust file's answers at the 0-based row `b` and op 5, its
//!   top-level surface: 0 the file does not read or parse, 2 the answers
//!   (ladder/rs_cst.rs).
//! Which facts are asked, in which order, and what an answer means are
//! the core's (CE.Resolve.TsFacts).

use crate::graph::ladder::rs_cst;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// One reader per batch of facts: a Rust file asked about at several
/// rows is read and parsed once.
pub struct Reader<'r> {
    root: &'r Path,
    parsed: HashMap<String, Option<(String, tree_sitter::Tree)>>,
}

impl<'r> Reader<'r> {
    pub fn new(root: &'r Path) -> Self {
        Reader {
            root,
            parsed: HashMap::new(),
        }
    }

    /// One fact as its request row.
    pub fn fact(&mut self, op: i64, a: &str, b: &str) -> Value {
        let path = self.root.join(a);
        let (state, payload) = match op {
            0 if !path.is_file() => (0, None),
            0 => std::fs::read_to_string(&path).map_or((1, None), |t| (2, Some(json!(t)))),
            1 => (i64::from(path.is_file()), None),
            2 => (i64::from(path.join("node_modules").join(b).is_dir()), None),
            3 if !path.is_file() => (0, None),
            3 => toml_document(&path).map_or((1, None), |d| (2, Some(d))),
            _ => self.syntax(op, a, b).map_or((0, None), |v| (2, Some(v))),
        };
        json!([op, a, b, state, payload])
    }

    /// A Rust fact (op 4 at a row, op 5 the surface); none when the file
    /// does not read or parse, or the row is no number.
    fn syntax(&mut self, op: i64, a: &str, b: &str) -> Option<Value> {
        let root = self.root;
        let (text, tree) = self
            .parsed
            .entry(a.to_string())
            .or_insert_with(|| rs_cst::parsed(root, a))
            .as_ref()?;
        if op == 4 {
            Some(rs_cst::at(tree, text, b.parse().ok()?))
        } else {
            Some(rs_cst::surface(tree, text))
        }
    }
}

/// A file's TOML document decoded, none when it does not read as UTF-8
/// TOML.
fn toml_document(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    let doc: toml::Table = text.parse().ok()?;
    serde_json::to_value(doc).ok()
}

/// One fact read on its own.
pub fn fact(root: &Path, op: i64, a: &str, b: &str) -> Value {
    Reader::new(root).fact(op, a, b)
}

/// The facts asked for, read and added to the request's fact table — a
/// fact the table already carries is not read again (the core refuses a
/// fact carried twice: `ts.fact i: asked twice`; it never asks for one it
/// carries, the measuring side's up-front facts may repeat one).
pub fn answer_facts(
    body: &mut Value,
    root: &Path,
    wanted: &[(i64, String, String)],
) -> Result<(), String> {
    if wanted.is_empty() {
        return Ok(());
    }
    if body["ts"]["facts"].is_null() {
        body["ts"]["facts"] = json!([]);
    }
    let carried = body["ts"]["facts"]
        .as_array_mut()
        .ok_or("resolve/1: request has no fact table")?;
    let mut held: HashSet<(i64, String, String)> = carried
        .iter()
        .filter_map(|r| Some((r[0].as_i64()?, r[1].as_str()?.into(), r[2].as_str()?.into())))
        .collect();
    let mut reader = Reader::new(root);
    for (op, a, b) in wanted {
        if held.insert((*op, a.clone(), b.clone())) {
            carried.push(reader.fact(*op, a, b));
        }
    }
    Ok(())
}
