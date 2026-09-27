//! The inheritance rung of a simple type name — java.rs's child (plan
//! v2.30 step 5b): a member type the class enclosing the site inherits
//! from a supertype another walked file declares. JLS 6.4.1 puts the
//! members a class declares or inherits in scope over its whole body,
//! innermost class first and ahead of the compilation unit's imports,
//! so the site's enclosing types (java_types.rs, by line) are asked
//! innermost first; each one's supertypes, as its header writes them,
//! are resolved to the declaring file in THAT type's own compilation
//! unit — a type of the same unit, else what the unit's imports,
//! package and star imports name (java.rs `declared`), else a fully
//! qualified name over the package index — and the nearest supertype
//! level declaring a member type of the name answers, the levels
//! beyond it hidden (JLS 8.5); two supertypes each declaring one are
//! ambiguous_paths. A supertype no walked file declares (the JDK's, a
//! third party's) contributes nothing — its members are unreadable,
//! never guessed.

use super::{At, Outcome, Reason, declared_one, settle};
use crate::graph::ladder::java_header::{Header, TypeDecl};
use std::collections::BTreeSet;

pub(super) fn inherited(at: &At, name: &str, header: &Header, line: usize) -> Option<Outcome> {
    for decl in enclosing(&header.types, line) {
        let hits = in_supers(at, at.from, decl, name, &mut BTreeSet::new());
        if !hits.is_empty() {
            return settle(
                hits.into_iter().collect(),
                3,
                Reason::AmbiguousPaths,
                at.scope,
            );
        }
    }
    None
}

/// The types enclosing `line`, innermost first.
fn enclosing(types: &[TypeDecl], line: usize) -> Vec<&TypeDecl> {
    let mut out = Vec::new();
    for t in types
        .iter()
        .filter(|t| t.lines.0 <= line && line <= t.lines.1)
    {
        out.extend(enclosing(&t.members, line));
        out.push(t);
    }
    out
}

/// The files whose declaration of a supertype of `decl` — its name
/// read in `file`'s own unit — holds a member type `name`; each
/// supertype chain answers at its nearest such level, a chain met
/// twice (a diamond, a cycle) is read once.
fn in_supers(
    at: &At,
    file: &str,
    decl: &TypeDecl,
    name: &str,
    seen: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let mut hits = BTreeSet::new();
    for written in &decl.supers {
        let Some((sfile, sdecl)) = declared_type(at, file, written) else {
            continue;
        };
        if !seen.insert(format!("{sfile}\0{}\0{}", sdecl.name, sdecl.lines.0)) {
            continue;
        }
        if sdecl.members.iter().any(|m| m.name == name) {
            hits.insert(sfile);
        } else {
            hits.extend(in_supers(at, &sfile, sdecl, name, seen));
        }
    }
    hits
}

/// The walked file and declaration a supertype name written in `file`
/// names. Its head segment is a type of `file`'s own unit, else what
/// the unit names around it (`declared`), else — a fully qualified
/// name — the longest prefix the package index holds; the segments
/// after the head descend member types. None = no walked file
/// declares it.
fn declared_type<'a>(at: &At<'a>, file: &str, written: &str) -> Option<(String, &'a TypeDecl)> {
    let segs: Vec<&str> = written.split('.').collect();
    let unit = at.scope.java.get(file)?;
    let here = At {
        from: file,
        index: at.index,
        scope: at.scope,
    };
    let (path, head) = if one_declared(&unit.types, segs[0]).is_some() {
        (file.to_string(), 0)
    } else if let Some(Outcome::Resolved { path, .. }) = here.declared(segs[0], unit) {
        (path, 0)
    } else {
        qualified_file(&here, &segs)?
    };
    let decl = one_declared(&at.scope.java.get(&path)?.types, segs[head])?;
    let found = segs[head + 1..]
        .iter()
        .try_fold(decl, |d, seg| d.members.iter().find(|m| m.name == *seg))?;
    Some((path, found))
}

/// The unit's one declaration of a simple name: its top-level type of
/// the name, else the one member type of the name at any depth.
fn one_declared<'h>(types: &'h [TypeDecl], name: &str) -> Option<&'h TypeDecl> {
    if let Some(top) = types.iter().find(|t| t.name == name) {
        return Some(top);
    }
    let mut nested = Vec::new();
    collect(types, name, &mut nested);
    match nested.as_slice() {
        [one] => Some(one),
        _ => None,
    }
}

fn collect<'h>(types: &'h [TypeDecl], name: &str, out: &mut Vec<&'h TypeDecl>) {
    for t in types {
        if t.name == name {
            out.push(t);
        }
        collect(&t.members, name, out);
    }
}

/// A fully qualified name over the package index: the longest prefix
/// `p.C` whose package holds a file declaring `C` — that file (the
/// declared roots' one among several) and the index of `C` in `segs`.
fn qualified_file(at: &At, segs: &[&str]) -> Option<(String, usize)> {
    for k in (2..=segs.len()).rev() {
        let files = at.class_files(&segs[..k - 1].join("."), segs[k - 1]);
        if !files.is_empty() {
            return declared_one(files.into_iter().collect(), at.scope)
                .ok()
                .map(|file| (file, k - 1));
        }
    }
    None
}
