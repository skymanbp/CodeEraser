//! Java rungs (plan v2.30 step 3; design booklet §8 row Java; the
//! inheritance, folded-import and second-top-level-class boundaries
//! closed in step 5b). Java names a class by its package, and a
//! package is what a compilation unit DECLARES (JLS 7.4), not where the
//! file sits — so the candidate index is the walk's reading of every
//! file's header (Scope::java, java_header.rs): a class `a.b.C` is a
//! file that declares `package a.b` and a top-level type `C`
//! (java_types.rs; a file the reader found no type in answers by its
//! name, `C.java`), which is javac's own reading — a second top-level
//! class declared in a file of another name included. The sites
//! (graph/sites.rs, graph/sites/java.rs) walk:
//!   R1 `import a.b.C`: that file — an import folded over lines, whose
//!      site the detector cut at its first line (`a.b.`), is read whole
//!      from the header's import on the site's line (`header_name`);
//!   R2 a shorter prefix naming such a file — `import a.b.C.D` names
//!      the nested D inside C.java, and `import static a.b.C.m` a
//!      member of it; `import a.b.*` the package's one directory
//!      (ResolvedPackage), or — `a.b.C.*`, a type's member types —
//!      C.java; `import static a.b.C.*` C.java;
//!   R3 `type_ref`, in the JLS 6.4.1 order: a member type the class
//!      enclosing the site inherits (java_inherit.rs — the supertypes
//!      of the enclosing types, innermost first, followed to the files
//!      declaring them and on through their own supertypes), the file's
//!      own single-type import of the name (its answer, whatever it
//!      is), the name's file in the file's own package (its own
//!      directory first), then in the packages it star-imports; a
//!      qualified `A.B` is whatever `A` names, else a fully qualified
//!      name — its type annotations dropped (`a.b.@Tag C` names a.b.C);
//!   R4 External: a name the JDK answers (java_jdk.rs) — an import or
//!      qualified name under a package it exports, a `java.lang` type,
//!      or a simple name no rung holds in a file whose out-of-scope
//!      star imports are all JDK packages.
//! Two files answering one rung is one class in two places —
//! ambiguous_root, or ambiguous_paths across star-imported packages or
//! supertypes — unless the declared `[graph.search_roots] java`
//! directories hold exactly one of them. A file in a standard-layout
//! main source set sees no test code, and a package split across
//! source sets answers the importing file's own part (java_sets.rs). A
//! name the file declares itself — `import static a.b.C.Nested.*`
//! inside C.java — reaches no other file: own_unit, as the detector
//! already drops a type_ref to one (graph/sites/java.rs). Anything else
//! is out_of_scope: a third-party import, a member inherited from a
//! supertype no walked file declares — misses, never a guessed edge.
//! The settling rules and the JDK readings are the child java_pick.rs.

use super::java_header::{Header, Import};
use super::java_jdk::LANG;
use super::java_sets::{own_root_dir, visible};
use super::{Outcome, Reason, Rung, Scope, Site};
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};

#[path = "java_inherit.rs"]
mod inherit;
#[path = "java_pick.rs"]
mod pick;
use pick::{declared_one, jdk, own_unit, settle, star_jdk, unannotated};

/// Every walked Java file by the package its header declares.
type Index = BTreeMap<String, Vec<String>>;

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let (kind, from) = (site.kind, site.from);
    let (is_static, spec) = match site.spec.strip_prefix("static") {
        Some(rest) if rest.starts_with(char::is_whitespace) => (true, rest),
        _ => (false, site.spec),
    };
    let mut name: String = unannotated(spec).split_whitespace().collect();
    if let Some(whole) = header_name(scope, site, &name, is_static) {
        name = whole;
    }
    let segs: Vec<&str> = name.split('.').collect();
    // a static import names a member after its class; a name cut short
    // that no header import completes (`a..b`) names nothing
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
        "type_ref" => Some(at.type_ref(&segs, site.line)),
        _ => return Outcome::Unresolved(Reason::Unsupported),
    };
    own_unit(from, found.unwrap_or_else(|| jdk(&segs)))
}

/// The import the header read on the site's line, when the site's spec
/// is a cut of it (plan v2.30 step 5b): a declaration folded over lines
/// leaves the detector — which reads the site's first line — `a.b.`,
/// while the header lexer read the whole name. Among the imports of the
/// site's kind and flags on that line, one whose name IS the spec means
/// the spec is whole (None); else the one name the cut spec begins
/// answers, two never pick.
fn header_name(scope: &Scope, site: &Site, spec: &str, is_static: bool) -> Option<String> {
    let star = match site.kind {
        "import" => false,
        "import_star" => true,
        _ => return None,
    };
    let on_line: Vec<&Import> = scope
        .java
        .get(site.from)?
        .imports
        .iter()
        .filter(|i| i.line == site.line && i.star == star && i.is_static == is_static)
        .collect();
    if on_line.iter().any(|i| i.name == spec) {
        return None;
    }
    let head = spec.trim_end_matches('.');
    let mut begun = on_line.iter().filter(|i| i.name.starts_with(head));
    match (begun.next(), begun.next()) {
        (Some(one), None) => Some(one.name.clone()),
        _ => None,
    }
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
    /// longest prefix `p.C` whose package `p` holds a file declaring
    /// `C` — the whole name at `exact`, a shorter prefix (an enclosing
    /// class) at `nested`. A package segment is required: the unnamed
    /// package imports nothing.
    fn class_of(&self, segs: &[&str], exact: Rung, nested: Rung) -> Option<Outcome> {
        (2..=segs.len()).rev().find_map(|k| {
            let rung = if k == segs.len() { exact } else { nested };
            let hits = self.class_files(&segs[..k - 1].join("."), segs[k - 1]);
            settle(hits, rung, Reason::AmbiguousRoot, self.scope)
        })
    }

    /// The package's files this file can see that declare `class`.
    fn class_files(&self, package: &str, class: &str) -> Vec<String> {
        self.index
            .get(package)
            .into_iter()
            .flatten()
            .filter(|f| visible(self.from, f) && self.declares(f, class))
            .cloned()
            .collect()
    }

    /// Whether `f` declares the top-level type `class`: by the types
    /// its header read (a second top-level class counts, step 5b), or
    /// — a file the reader found no type in — by its name.
    fn declares(&self, f: &str, class: &str) -> bool {
        match self.scope.java.get(f).map(|h| h.types.as_slice()) {
            Some(types) if !types.is_empty() => types.iter().any(|t| t.name == class),
            _ => f.rsplit('/').next() == Some(format!("{class}.java").as_str()),
        }
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
    fn type_ref(&self, segs: &[&str], line: usize) -> Outcome {
        let Some(header) = self.scope.java.get(self.from) else {
            return Outcome::Unresolved(Reason::OutOfScope);
        };
        if let Some(found) = self.simple(segs[0], header, line) {
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

    /// A simple type name in the JLS 6.4.1 order: a member type the
    /// class enclosing the site inherits (java_inherit.rs) — in scope
    /// over the class body, ahead of the unit's imports — then what the
    /// unit names around it.
    fn simple(&self, name: &str, header: &Header, line: usize) -> Option<Outcome> {
        inherit::inherited(self, name, header, line).or_else(|| self.declared(name, header))
    }

    /// The unit's own view of a simple name: a single-type import of it
    /// (static ones included — `import static a.b.C.N` brings a static
    /// nested N), the file's own package, then its star-imported
    /// packages.
    fn declared(&self, name: &str, header: &Header) -> Option<Outcome> {
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

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/java.rs"]
mod tests;
