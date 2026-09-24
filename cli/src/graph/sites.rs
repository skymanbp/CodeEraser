//! Resolution-free reference-site detection — the frozen sample
//! universe of the M5-2 precision instrument (design brief §5: the
//! universe is SITES, not edges, so the resolver can never choose
//! its own precision denominator). Detection needs only the
//! spec-table node kinds; no path is ever consulted.
//!
//! Multi-line specifiers (a rustfmt-folded `use x::{a, b, …}`) keep
//! ONE site whose spec is the first line's fragment: the spec string
//! is a location anchor and an anti-invention check (the self-corpus
//! drift gate in eval_graph.rs re-detects and substring-checks it).
//! Resolution consumes only the pre-`{` prefix, which formatting
//! folds never cut; a fragment cut mid-path is refused, never
//! guessed shallow (ladder/rs.rs module header).

mod java;

use super::md;
use super::spec::{SiteKind, Specifier, sites as site_table};
use crate::fourclass::units;
use crate::scan::ast::{self, children};
use crate::scan::lang::Lang;

/// One detected site, before path attachment. `nth` is the site's
/// 0-based ordinal among same-line sites (document order), making
/// (path, line, nth) a unique identity — (line, kind, spec) alone is
/// not, and the 2c sampling rank key needs uniqueness (Opus review).
pub struct RawSite {
    pub kind: &'static str,
    pub line: usize,
    pub nth: usize,
    pub spec: String,
    pub owner: Option<String>,
}

impl RawSite {
    pub fn md(kind: &'static str, line: usize, spec: String) -> Self {
        RawSite {
            kind,
            line,
            nth: 0,
            spec,
            owner: None,
        }
    }
}

/// Detect every reference site in one document.
pub fn detect(text: &str, lang: Lang) -> Vec<RawSite> {
    detect_with_units(text, lang).0
}

/// Detection plus the unit segmentation it computes anyway for
/// ownership: the graph store (schema v4 phase 1) persists both
/// without a second parse. detect()'s output is byte-identical —
/// the frozen universe and the eval drift gates stand on it (RG3).
pub fn detect_with_units(text: &str, lang: Lang) -> (Vec<RawSite>, Vec<units::Unit>) {
    // The scan-only arm (plan v2.5) carries no graph semantics and
    // must not fall through to the markdown detector below — a .css
    // file "detected" as markdown invented link sites on the
    // standalone --sites face (review 2026-08-20 #4). Markdown stays
    // the ONLY grammarless judged language.
    if lang.scan_only() {
        return (Vec::new(), Vec::new());
    }
    let mut found = if lang.grammar().is_some() {
        code_sites(text, lang)
    } else {
        md::detect(text)
    };
    let owners = units::segments(text, lang);
    let mut prev = (0usize, 0usize);
    for site in &mut found {
        // A unit spanning exactly the site's single line IS the site
        // (a `mod foo;` declaration is its own one-line unit) — self
        // ownership is noise, not containment (Opus review).
        site.owner = units::owner(&owners, site.line)
            .filter(|u| !(u.start_line == site.line && u.end_line == site.line))
            .map(|u| u.key.clone());
        prev = if prev.0 == site.line {
            (site.line, prev.1 + 1)
        } else {
            (site.line, 0)
        };
        site.nth = prev.1;
    }
    (found, owners)
}

fn code_sites(text: &str, lang: Lang) -> Vec<RawSite> {
    ast::with_tree(text, lang, |tree| {
        let (root, src) = (tree.root_node(), text.as_bytes());
        let mut found = walk_sites(root, src, site_table(lang));
        if lang == Lang::Java {
            // imports precede every type in a compilation unit (JLS 7.3),
            // so the pass's sites follow the table's in document order on
            // any line the two share; the stable sort keeps that order
            found.extend(java::type_refs(root, src));
            found.sort_by_key(|s| s.line);
        }
        found
    })
}

fn walk_sites(root: tree_sitter::Node, src: &[u8], table: &[SiteKind]) -> Vec<RawSite> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        // one statement, one site: the first table entry that emits
        // wins, so a TS `export *` lands under its star label and
        // never also under the plain export_from one behind it
        for kind in table.iter().filter(|k| k.node == node.kind()) {
            if emit(node, src, kind, &mut out) {
                break;
            }
        }
        // deterministic order: children pushed reversed => visited
        // in document order after the stack pop
        for child in children(node).into_iter().rev() {
            stack.push(child);
        }
    }
    out.sort_by_key(|s| s.line);
    out
}

/// Whether the entry opened a site (or several) on this node.
fn emit(node: tree_sitter::Node, src: &[u8], kind: &SiteKind, out: &mut Vec<RawSite>) -> bool {
    let found = specs(node, src, &kind.via);
    let opened = !found.is_empty();
    out.extend(
        found
            .into_iter()
            .map(|(at, spec)| site(kind.label, at, spec)),
    );
    opened
}

/// The specs an entry reads off a node, each with the node its site
/// sits on (a Python import target is a child of the statement).
fn specs<'t>(
    node: tree_sitter::Node<'t>,
    src: &[u8],
    via: &Specifier,
) -> Vec<(tree_sitter::Node<'t>, String)> {
    let here = |spec: Option<String>| spec.map(|s| (node, s)).into_iter().collect();
    match via {
        Specifier::Field(field) => here(field_text(node, src, field)),
        Specifier::FieldIfStar(field) => here(
            star_export(node)
                .then(|| field_text(node, src, field))
                .flatten(),
        ),
        Specifier::NameIfNoBody => here(
            node.child_by_field_name("body")
                .is_none()
                .then(|| field_text(node, src, "name"))
                .flatten(),
        ),
        Specifier::EachImportTarget => children(node)
            .into_iter()
            .filter_map(|child| import_target(child, src).map(|spec| (child, spec)))
            .collect(),
        Specifier::Literal(spec) => vec![(node, spec.to_string())],
        Specifier::FirstNamed { star } => here(
            (star_export(node) == *star)
                .then(|| import_name(node, src))
                .flatten(),
        ),
    }
}

/// `export * from …` carries a bare `*` token, `export * as ns from …`
/// a `namespace_export` child and a Java `import a.b.*` an `asterisk`
/// one; the `export_clause` forms and a single-type import carry none.
fn star_export(node: tree_sitter::Node) -> bool {
    children(node)
        .into_iter()
        .any(|c| matches!(c.kind(), "*" | "namespace_export" | "asterisk"))
}

/// A Java import's spec (Specifier::FirstNamed): the source from the
/// `static` token, when there is one, to the end of the first
/// `scoped_identifier` / `identifier` child — cut at a line break like
/// every spec, so it stays a substring of its line.
fn import_name(node: tree_sitter::Node, src: &[u8]) -> Option<String> {
    let kids = children(node);
    let name = kids
        .iter()
        .find(|c| matches!(c.kind(), "scoped_identifier" | "identifier"))?;
    let start = kids
        .iter()
        .find(|c| c.kind() == "static")
        .map_or(name.start_byte(), |s| s.start_byte());
    let raw = std::str::from_utf8(src.get(start..name.end_byte())?).ok()?;
    Some(raw.split('\n').next().unwrap_or("").trim().to_string())
}

/// Python `import a.b, c as d`: dotted_name children are targets;
/// aliased_import carries its target in the `name` field.
fn import_target(node: tree_sitter::Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "dotted_name" => node_text(node, src),
        "aliased_import" => field_text(node, src, "name"),
        _ => None,
    }
}

fn site(label: &'static str, node: tree_sitter::Node, spec: String) -> RawSite {
    RawSite {
        kind: label,
        line: node.start_position().row + 1,
        nth: 0, // assigned centrally in detect()
        spec,
        owner: None,
    }
}

fn field_text(node: tree_sitter::Node, src: &[u8], field: &str) -> Option<String> {
    node_text(node.child_by_field_name(field)?, src)
}

/// Node text as a spec string: quotes trimmed (string-literal
/// specifiers), truncated at the first newline (see module header),
/// whitespace-trimmed. A degenerate specifier (`import ""`) is KEPT
/// as an empty spec: the site referenced nothing resolvable, and the
/// unresolved ledger is its honest home — the ladder refuses it by
/// name (`Reason::Empty`) instead of detection dropping it in
/// silence (the promise this comment carried until L step #15, O60).
fn node_text(node: tree_sitter::Node, src: &[u8]) -> Option<String> {
    let raw = node.utf8_text(src).ok()?;
    let first = raw.split('\n').next().unwrap_or("");
    Some(
        first
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string(),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/graph/sites_tests.rs"]
mod tests;
