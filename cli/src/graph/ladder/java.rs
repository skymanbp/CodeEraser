//! Java rungs (plan v2.30 step 3; design booklet §8 row Java). Java
//! names a class by its package, and a package is what a compilation
//! unit DECLARES (JLS 7.4), not where the file sits — so the candidate
//! index is the walk's reading of every file's header (Scope::java,
//! java_header.rs): a class `a.b.C` is a file named `C.java` that
//! declares `package a.b`, which is javac's own reading. The sites
//! (graph/sites.rs, graph/sites/java.rs) walk:
//!   R1 `import a.b.C`: that file;
//!   R2 a shorter prefix naming such a file — `import a.b.C.D` names
//!      the nested D inside C.java, and `import static a.b.C.m` a
//!      member of it; `import a.b.*` the package's one directory
//!      (ResolvedPackage), or — `a.b.C.*`, a type's member types —
//!      C.java; `import static a.b.C.*` C.java;
//!   R3 `type_ref`, in the JLS 6.4.1 order: the file's own
//!      single-type import of the name (its answer, whatever it is),
//!      the name's file in the file's own package (its own directory
//!      first), then in the packages it star-imports; a qualified
//!      `A.B` is whatever `A` names, else a fully qualified name — its
//!      type annotations dropped (`a.b.@Tag C` names a.b.C);
//!   R4 External: a name the JDK answers (java_jdk.rs) — an import or
//!      qualified name under a package it exports, a `java.lang` type,
//!      or a simple name no rung holds in a file whose out-of-scope
//!      star imports are all JDK packages.
//! Two files answering one rung is one class in two places —
//! ambiguous_root, or ambiguous_paths across star-imported packages —
//! unless the declared `[graph.search_roots] java` directories hold
//! exactly one of them. Anything else is out_of_scope: a third-party
//! import, a nested type reached by inheritance, a class declared in a
//! file of another name (a second top-level class) — misses, never a
//! guessed edge.

use super::java_header::{self, Header};
use super::java_jdk::{LANG, PACKAGES};
use super::{Outcome, Reason, Rung, Scope};
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};

/// Every walked Java file by the package its header declares.
type Index = BTreeMap<String, Vec<String>>;

pub fn resolve(kind: &str, from: &str, spec: &str, scope: &Scope) -> Outcome {
    let (is_static, name) = match spec.strip_prefix("static") {
        Some(rest) if rest.starts_with(char::is_whitespace) => (true, rest),
        _ => (false, spec),
    };
    let name: String = unannotated(name).split_whitespace().collect();
    let segs: Vec<&str> = name.split('.').collect();
    // a static import names a member after its class; a name cut short
    // (a multi-line import, `a.b.`) names nothing
    if segs.iter().any(|s| s.is_empty()) || (is_static && segs.len() < 2) {
        return Outcome::Unresolved(Reason::OutOfScope);
    }
    let index = scope.memo.cached("java-packages", "", || index_of(scope));
    let found = match kind {
        "import" if is_static => class_of(&segs[..segs.len() - 1], 2, 2, &index, scope),
        "import" => class_of(&segs, 1, 2, &index, scope),
        "import_star" if is_static => class_of(&segs, 2, 2, &index, scope),
        "import_star" => {
            package_dir(&name, &index, scope).or_else(|| class_of(&segs, 2, 2, &index, scope))
        }
        "type_ref" => return type_ref(from, &segs, &index, scope),
        _ => return Outcome::Unresolved(Reason::Unsupported),
    };
    found.unwrap_or_else(|| jdk(&segs))
}

/// A name without its type annotations: `a.b.@Tag C` names `a.b.C`
/// (JLS 9.7.4) — each `@` with its name and argument list, read by the
/// header lexer.
fn unannotated(name: &str) -> String {
    let mut out = String::new();
    let mut rest = name;
    while let Some((before, after)) = rest.split_once('@') {
        out.push_str(before);
        rest = java_header::past_annotation(after);
    }
    out + rest
}

fn index_of(scope: &Scope) -> Index {
    let mut index = Index::new();
    for (file, header) in scope.java {
        if scope.files.contains(file) {
            index
                .entry(header.package.clone())
                .or_default()
                .push(file.clone());
        }
    }
    index
}

/// The file declaring the class a fully qualified name names: the
/// longest prefix `p.C` whose package `p` holds a `C.java` — the whole
/// name at `exact`, a shorter prefix (an enclosing class) at `nested`.
/// A package segment is required: the unnamed package imports nothing.
fn class_of(
    segs: &[&str],
    exact: Rung,
    nested: Rung,
    index: &Index,
    scope: &Scope,
) -> Option<Outcome> {
    (2..=segs.len()).rev().find_map(|k| {
        let rung = if k == segs.len() { exact } else { nested };
        let hits = class_files(index, &segs[..k - 1].join("."), segs[k - 1]);
        settle(hits, rung, Reason::AmbiguousRoot, scope)
    })
}

fn class_files(index: &Index, package: &str, class: &str) -> Vec<String> {
    let file = format!("{class}.java");
    index
        .get(package)
        .into_iter()
        .flatten()
        .filter(|f| f.rsplit('/').next() == Some(file.as_str()))
        .cloned()
        .collect()
}

/// `import a.b.*`: the one directory holding package a.b's files.
fn package_dir(package: &str, index: &Index, scope: &Scope) -> Option<Outcome> {
    let dirs = index
        .get(package)?
        .iter()
        .map(|f| roots::parent_dir(f))
        .collect();
    match declared_one(dirs, scope) {
        Ok(dir) => Some(Outcome::ResolvedPackage { dir, rung: 2 }),
        Err(_) => Some(Outcome::Unresolved(Reason::AmbiguousRoot)),
    }
}

/// One hit resolves; several resolve only when the declared roots
/// hold exactly one of them, else `many` refuses; none says nothing.
fn settle(hits: Vec<String>, rung: Rung, many: Reason, scope: &Scope) -> Option<Outcome> {
    if hits.is_empty() {
        return None;
    }
    Some(match declared_one(hits.into_iter().collect(), scope) {
        Ok(path) => Outcome::Resolved { path, rung },
        Err(()) => Outcome::Unresolved(many),
    })
}

/// The one member of a non-empty set, or the one under the declared
/// `[graph.search_roots] java` directories.
fn declared_one(set: BTreeSet<String>, scope: &Scope) -> Result<String, ()> {
    if set.len() == 1 {
        return set.into_iter().next().ok_or(());
    }
    let declared = scope.search_roots.get("java");
    let under = |p: &String| {
        declared.is_some_and(|dirs| {
            dirs.iter()
                .any(|d| d.is_empty() || p == d || p.starts_with(&format!("{d}/")))
        })
    };
    let mut kept = set.into_iter().filter(under);
    match (kept.next(), kept.next()) {
        (Some(one), None) => Ok(one),
        _ => Err(()),
    }
}

/// R3, then R4 for what no rung holds.
fn type_ref(from: &str, segs: &[&str], index: &Index, scope: &Scope) -> Outcome {
    let Some(header) = scope.java.get(from) else {
        return Outcome::Unresolved(Reason::OutOfScope);
    };
    if let Some(found) = simple(from, segs[0], header, index, scope) {
        return found;
    }
    if let Some(found) = class_of(segs, 3, 3, index, scope) {
        return found;
    }
    if LANG.split_whitespace().any(|t| t == segs[0]) {
        return Outcome::External { rung: 4 };
    }
    if segs.len() > 1 {
        return jdk(segs);
    }
    if star_jdk(header, index) {
        return Outcome::External { rung: 4 };
    }
    Outcome::Unresolved(Reason::OutOfScope)
}

/// A simple type name in the JLS 6.4.1 order: a single-type import of
/// it (static ones included — `import static a.b.C.N` brings a static
/// nested N), the file's own package, then its star-imported packages.
fn simple(
    from: &str,
    name: &str,
    header: &Header,
    index: &Index,
    scope: &Scope,
) -> Option<Outcome> {
    // a type and a static member may share a name (two namespaces):
    // the type import answers first
    let single = |wanted: bool| {
        header
            .imports
            .iter()
            .find(|i| !i.star && i.is_static == wanted && i.name.rsplit('.').next() == Some(name))
    };
    if let Some(import) = single(false).or_else(|| single(true)) {
        let segs: Vec<&str> = import.name.split('.').collect();
        let cut = if import.is_static {
            segs.len() - 1
        } else {
            segs.len()
        };
        let found = class_of(&segs[..cut], 3, 3, index, scope).unwrap_or_else(|| jdk(&segs));
        return Some(found);
    }
    let own = class_files(index, &header.package, name);
    let beside = roots::join_dir(&roots::parent_dir(from), &format!("{name}.java"));
    if own.contains(&beside) {
        return Some(Outcome::Resolved {
            path: beside,
            rung: 3,
        });
    }
    if let Some(found) = settle(own, 3, Reason::AmbiguousRoot, scope) {
        return Some(found);
    }
    let starred = header
        .imports
        .iter()
        .filter(|i| i.star && !i.is_static)
        .flat_map(|i| class_files(index, &i.name, name))
        .collect();
    settle(starred, 3, Reason::AmbiguousPaths, scope)
}

/// Whether a simple name no rung holds can only come from the JDK: the
/// file star-imports at least one JDK package, and every package it
/// star-imports that the tree does not hold is one.
fn star_jdk(header: &Header, index: &Index) -> bool {
    let out: Vec<&str> = header
        .imports
        .iter()
        .filter(|i| i.star && !i.is_static && !index.contains_key(&i.name))
        .map(|i| i.name.as_str())
        .collect();
    !out.is_empty() && out.iter().all(|p| exported(p))
}

/// R4 for a dotted name: External when some prefix of it is a package
/// the JDK exports.
fn jdk(segs: &[&str]) -> Outcome {
    if (1..=segs.len()).any(|k| exported(&segs[..k].join("."))) {
        Outcome::External { rung: 4 }
    } else {
        Outcome::Unresolved(Reason::OutOfScope)
    }
}

fn exported(package: &str) -> bool {
    PACKAGES.split_whitespace().any(|p| p == package)
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/java.rs"]
mod tests;
