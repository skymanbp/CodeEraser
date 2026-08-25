//! The symbol edge: a file that references a DECLARATION in another
//! file (plan v2.14, the necessary condition). Two measured facts
//! meet here — the names an import binds (graph/bindings.rs) and the
//! declarations a file makes (the symbols table, carrying vis since
//! the visibility slice) — and an edge exists only where a bound name
//! hits a declaration in the file the ladder already resolved to.
//!
//! That condition is the whole design. Four adversarial reviews
//! (2026-08-24) showed the leaf of an import path is syntactically
//! indistinguishable between a declaration, a submodule and a
//! re-export, so the extractor offers CANDIDATES and the lookup
//! decides. A candidate that misses is a module or a re-export: the
//! file-level edge stands as it always did and no symbol edge is
//! minted. One edge short beats one edge wrong.
//!
//! What is NOT done here, by construction: no same-name matching
//! across the repo. The only input is `bindings`, so a symbol that
//! merely shares a name with something called elsewhere yields
//! nothing — R6 died minting exactly those edges at 38/66 precision,
//! and the difference is that a binding is a syntactic fact about
//! THIS file while a name match is a guess about every file.
//!
//! Known recall limit, stated rather than left to be discovered: the
//! symbols table only holds what fourclass::kinds registers, and
//! Haskell's named type forms are deliberately absent there, so an
//! `import M (SomeType)` misses and degrades to its file edge. That
//! is the same honest direction as a module or a re-export miss, and
//! widening kinds::extra is the one place that would change it.
//!
//! Consumer: the graph request's `symEdges` table at proto 4.1.0,
//! which is the next slice — this one lands the measurement and the
//! independent site audit judges it before any verdict is built on
//! top (plan v2.14 K10: the ladder's own rule, that a slice must
//! pass the measure that killed R6 before it is trusted).

use crate::dedup::index::Index;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

/// One file→declaration reference. `key` is the declaration's own
/// unit key (its identity in the symbols table), `vis` its
/// visibility bits — the public/private axis the graph's verdict
/// codes have always meant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymEdge {
    pub src: String,
    pub dst_path: String,
    pub key: String,
    pub vis: i64,
}

/// The importable name a unit key carries, or None when the key
/// names something no import can bind.
///
/// Key shapes are units.rs's: `name/arity` for functions, a bare
/// name for the extra kinds (consts, types), `impl X` / `impl T for
/// X` for typed kinds, `(T) m/arity` for a Go receiver method, and
/// heading text for Markdown. Only the first two are importable by
/// name; a method is reached through its receiver and an impl block
/// has no name to bind, so both answer None rather than lending
/// their inner word to a lookup.
pub fn declared_name(key: &str) -> Option<&str> {
    let name = match key.rsplit_once('/') {
        // an arity is digits: `a/b` is a heading that happens to
        // hold a slash, not a function of arity `b`
        Some((head, arity)) if !arity.is_empty() && arity.bytes().all(|b| b.is_ascii_digit()) => {
            head
        }
        _ => key,
    };
    // a receiver prefix or an interior space marks a shape no
    // identifier can equal; rejecting them keeps the lookup from
    // reading a word out of a compound key
    let plain = !name.is_empty() && !name.starts_with('(') && !name.contains(' ');
    plain.then_some(name)
}

/// One declaration as the lookup sees it.
struct Decl {
    key: String,
    vis: i64,
}

/// Every symbol edge the corpus supports, deterministically ordered
/// and duplicate-free — two `use` statements naming one declaration
/// are one reference.
///
/// Both reads share ONE snapshot for the reason graph_rows states:
/// a convergent writer (ADR-003) landing between them could hand the
/// join a binding whose target file's symbols it never saw, and a
/// stale hit here would mint an edge a judgment then trusts.
pub fn symbol_edges(idx: &Index) -> Result<Vec<SymEdge>> {
    let txn = idx.raw().unchecked_transaction()?;
    let conn = &*txn;
    let decls = declarations(conn)?;
    let bindings = candidates(conn)?;
    txn.finish()?; // read-only: closing the snapshot, nothing to write
    let mut out = BTreeSet::new();
    for (src, dst_path, target) in bindings {
        for d in decls.get(&dst_path).into_iter().flatten() {
            if declared_name(&d.key) == Some(target.as_str()) {
                out.insert(SymEdge {
                    src: src.clone(),
                    dst_path: dst_path.clone(),
                    key: d.key.clone(),
                    vis: d.vis,
                });
            }
        }
    }
    Ok(out.into_iter().collect())
}

/// Declarations by declaring file. Overloads are kept apart: a TS
/// signature pair lands `f/1` and `f/2`, and an import of `f`
/// references both — the overload set IS what the name denotes.
fn declarations(conn: &rusqlite::Connection) -> Result<BTreeMap<String, Vec<Decl>>> {
    let mut by_file: BTreeMap<String, Vec<Decl>> = BTreeMap::new();
    let read: Vec<(String, String, i64)> = super::load::rows(
        conn,
        "SELECT f.path, s.key, s.flags FROM symbols s JOIN files f ON f.id = s.file_id
         ORDER BY f.path, s.key, s.nth",
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    for (path, key, vis) in read {
        by_file.entry(path).or_default().push(Decl { key, vis });
    }
    Ok(by_file)
}

/// Candidate bindings: the citing file, the file the ladder
/// resolved, and the TARGET-side name (the local name is this file's
/// business — the lookup asks the target what it declares).
fn candidates(conn: &rusqlite::Connection) -> Result<Vec<(String, String, String)>> {
    super::load::rows(
        conn,
        "SELECT f.path, e.dst_path, b.target
         FROM bindings b
         JOIN sites s ON s.id = b.site_id
         JOIN edges e ON e.site_id = s.id
         JOIN files f ON f.id = s.file_id
         WHERE e.granularity = 0
         ORDER BY f.path, e.dst_path, b.target",
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
}

#[cfg(test)]
mod tests {
    use super::declared_name;
    use crate::fourclass::units::segments;
    use crate::scan::lang::Lang;
    use std::collections::BTreeSet;

    /// The inverse is pinned against the REAL key producer, not a
    /// string table: units.rs builds these keys, and a change to its
    /// format that this function did not learn about would silently
    /// cost edges rather than fail anything.
    #[test]
    fn importable_names_come_back_out_of_real_keys() {
        let cases: &[(Lang, &str, &[&str])] = &[
            (
                Lang::Rust,
                "pub struct Config {\n    a: u8,\n}\n\npub fn open_door(x: u8) {\n    let _ = x;\n}\n\nimpl Config {\n    fn hidden(&self) {}\n}\n",
                // the struct by its bare key and the fn by name/arity;
                // the impl block never lends out the word "Config"
                &["Config", "hidden", "open_door"],
            ),
            (
                Lang::Go,
                "package m\n\nfunc Open(a int) {}\n\nfunc (t T) Shut() {}\n",
                // the receiver-qualified method drops out: `(T) Shut`
                // is not a name any import can bind
                &["Open"],
            ),
        ];
        for (lang, src, want) in cases {
            let units = segments(src, *lang);
            let got: Vec<&str> = units
                .iter()
                .filter_map(|u| declared_name(&u.key))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            assert_eq!(got, want.to_vec(), "{lang:?}");
        }
    }

    /// A Markdown heading is a key too, and a heading that happens to
    /// hold a slash must not read as a function of arity "Two".
    #[test]
    fn a_heading_is_never_read_as_an_arity() {
        assert_eq!(declared_name("One/Two"), Some("One/Two"));
        assert_eq!(declared_name("Getting Started"), None);
        assert_eq!(declared_name("open_door/0"), Some("open_door"));
    }
}
