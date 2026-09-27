//! java.rs's child (plan v2.30 step 5b, split on the file-length line):
//! the settling rules every Java rung shares — one hit resolves, the
//! declared `[graph.search_roots] java` directories pick among several,
//! a file's own name draws no edge — and the JDK tables' readings
//! (java_jdk.rs).

use super::{Index, Outcome, Reason, Rung, Scope};
use crate::graph::ladder::java_header::Header;
use crate::graph::ladder::java_jdk::PACKAGES;
use std::collections::BTreeSet;

/// A name the referencing file declares itself reaches no other file:
/// JLS 7.3 puts a compilation unit's own types in scope without an
/// import, so importing one names no other unit and draws no edge.
pub(super) fn own_unit(from: &str, found: Outcome) -> Outcome {
    match found {
        Outcome::Resolved { path, .. } if path == from => Outcome::Unresolved(Reason::OwnUnit),
        other => other,
    }
}

/// A name without its type annotations: `a.b.@Tag C` names `a.b.C`
/// (JLS 9.7.4) — each `@` with its name and argument list, read by the
/// header lexer.
pub(super) fn unannotated(name: &str) -> String {
    let mut out = String::new();
    let mut rest = name;
    while let Some((before, after)) = rest.split_once('@') {
        out.push_str(before);
        rest = crate::graph::ladder::java_header::past_annotation(after);
    }
    out + rest
}

/// One hit resolves; several resolve only when the declared roots
/// hold exactly one of them, else `many` refuses; none says nothing.
pub(super) fn settle(
    hits: Vec<String>,
    rung: Rung,
    many: Reason,
    scope: &Scope,
) -> Option<Outcome> {
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
pub(super) fn declared_one(set: BTreeSet<String>, scope: &Scope) -> Result<String, ()> {
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
pub(super) fn star_jdk(header: &Header, index: &Index) -> bool {
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
pub(super) fn jdk(segs: &[&str]) -> Outcome {
    if (1..=segs.len()).any(|k| exported(&segs[..k].join("."))) {
        Outcome::External { rung: 4 }
    } else {
        Outcome::Unresolved(Reason::OutOfScope)
    }
}

fn exported(package: &str) -> bool {
    PACKAGES.split_whitespace().any(|p| p == package)
}
