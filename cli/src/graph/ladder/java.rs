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
//! exactly one of them. A file in a standard-layout main source set
//! sees no test code, and a package split across source sets answers
//! the importing file's own part (java_sets.rs). A name the file
//! declares itself — `import static a.b.C.Nested.*` inside C.java —
//! reaches no other file: own_unit, as the detector already drops a
//! type_ref to one (graph/sites/java.rs). Anything else is
//! out_of_scope: a third-party import, a nested type reached by
//! inheritance, a class declared in a file of another name (a second
//! top-level class) — misses, never a guessed edge.

use super::java_header::{self, Header};
use super::java_jdk::{LANG, PACKAGES};
use super::java_sets::{own_root_dir, visible};
use super::{Outcome, Reason, Rung, Scope, Site};
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};

/// Every walked Java file by the package its header declares.
type Index = BTreeMap<String, Vec<String>>;

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let (kind, from, spec) = (site.kind, site.from, site.spec);
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
    let at = At {
        from,
        index: &index,
        scope,
    };
    let found = match kind {
        "import" if is_static => at.class_of(&segs[..segs.len() - 1], 2, 2),
        "import" => at.class_of(&segs, 1, 2),
        "import_star" if is_static => at.class_of(&segs, 2, 2),
        "import_star" => at.package_dir(&name).or_else(|| at.class_of(&segs, 2, 2)),
        "type_ref" => Some(at.type_ref(&segs)),
        _ => return Outcome::Unresolved(Reason::Unsupported),
    };
    own_unit(from, found.unwrap_or_else(|| jdk(&segs)))
}

/// A name the referencing file declares itself reaches no other file:
/// JLS 7.3 puts a compilation unit's own types in scope without an
/// import, so importing one names no other unit and draws no edge.
fn own_unit(from: &str, found: Outcome) -> Outcome {
    match found {
        Outcome::Resolved { path, .. } if path == from => Outcome::Unresolved(Reason::OwnUnit),
        other => other,
    }
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

/// One site's question to the index: the referencing file, the
/// index and the scope — every candidate it hands out is one the file
/// can see (java_sets.rs).
struct At<'a> {
    from: &'a str,
    index: &'a Index,
    scope: &'a Scope<'a>,
}

impl At<'_> {
    /// The file declaring the class a fully qualified name names: the
    /// longest prefix `p.C` whose package `p` holds a `C.java` — the
    /// whole name at `exact`, a shorter prefix (an enclosing class) at
    /// `nested`. A package segment is required: the unnamed package
    /// imports nothing.
    fn class_of(&self, segs: &[&str], exact: Rung, nested: Rung) -> Option<Outcome> {
        (2..=segs.len()).rev().find_map(|k| {
            let rung = if k == segs.len() { exact } else { nested };
            let hits = self.class_files(&segs[..k - 1].join("."), segs[k - 1]);
            settle(hits, rung, Reason::AmbiguousRoot, self.scope)
        })
    }

    fn class_files(&self, package: &str, class: &str) -> Vec<String> {
        let file = format!("{class}.java");
        self.index
            .get(package)
            .into_iter()
            .flatten()
            .filter(|f| f.rsplit('/').next() == Some(file.as_str()) && visible(self.from, f))
            .cloned()
            .collect()
    }

    /// `import a.b.*`: the one directory holding the package's files
    /// this file can see — its own source root's part of a split one.
    fn package_dir(&self, package: &str) -> Option<Outcome> {
        let dirs: BTreeSet<String> = self
            .index
            .get(package)?
            .iter()
            .filter(|f| visible(self.from, f))
            .map(|f| roots::parent_dir(f))
            .collect();
        if dirs.is_empty() {
            return None;
        }
        let picked = match own_root_dir(self.from, dirs.iter()) {
            Some(dir) => Ok(dir),
            None => declared_one(dirs, self.scope),
        };
        Some(match picked {
            Ok(dir) => Outcome::ResolvedPackage { dir, rung: 2 },
            Err(()) => Outcome::Unresolved(Reason::AmbiguousRoot),
        })
    }

    /// R3, then R4 for what no rung holds.
    fn type_ref(&self, segs: &[&str]) -> Outcome {
        let Some(header) = self.scope.java.get(self.from) else {
            return Outcome::Unresolved(Reason::OutOfScope);
        };
        if let Some(found) = self.simple(segs[0], header) {
            return found;
        }
        if let Some(found) = self.class_of(segs, 3, 3) {
            return found;
        }
        if LANG.split_whitespace().any(|t| t == segs[0]) {
            return Outcome::External { rung: 4 };
        }
        if segs.len() > 1 {
            return jdk(segs);
        }
        if star_jdk(header, self.index) {
            return Outcome::External { rung: 4 };
        }
        Outcome::Unresolved(Reason::OutOfScope)
    }

    /// A simple type name in the JLS 6.4.1 order: a single-type import
    /// of it (static ones included — `import static a.b.C.N` brings a
    /// static nested N), the file's own package, then its star-imported
    /// packages.
    fn simple(&self, name: &str, header: &Header) -> Option<Outcome> {
        // a type and a static member may share a name (two namespaces):
        // the type import answers first
        let single = |wanted: bool| {
            header.imports.iter().find(|i| {
                !i.star && i.is_static == wanted && i.name.rsplit('.').next() == Some(name)
            })
        };
        if let Some(import) = single(false).or_else(|| single(true)) {
            let segs: Vec<&str> = import.name.split('.').collect();
            let cut = if import.is_static {
                segs.len() - 1
            } else {
                segs.len()
            };
            return Some(
                self.class_of(&segs[..cut], 3, 3)
                    .unwrap_or_else(|| jdk(&segs)),
            );
        }
        let own = self.class_files(&header.package, name);
        let beside = roots::join_dir(&roots::parent_dir(self.from), &format!("{name}.java"));
        if own.contains(&beside) {
            return Some(Outcome::Resolved {
                path: beside,
                rung: 3,
            });
        }
        if let Some(found) = settle(own, 3, Reason::AmbiguousRoot, self.scope) {
            return Some(found);
        }
        let starred = header
            .imports
            .iter()
            .filter(|i| i.star && !i.is_static)
            .flat_map(|i| self.class_files(&i.name, name))
            .collect();
        settle(starred, 3, Reason::AmbiguousPaths, self.scope)
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
