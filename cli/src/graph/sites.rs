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
    let mut found = match lang.grammar() {
        Some(grammar) => code_sites(text, lang, grammar),
        None => md::detect(text),
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

fn code_sites(text: &str, lang: Lang, grammar: tree_sitter::Language) -> Vec<RawSite> {
    match ast::parse(text, &grammar) {
        None => Vec::new(),
        Some(tree) => walk_sites(tree.root_node(), text.as_bytes(), site_table(lang)),
    }
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
    let before = out.len();
    match &kind.via {
        Specifier::Field(field) => {
            if let Some(spec) = field_text(node, src, field) {
                out.push(site(kind.label, node, spec));
            }
        }
        Specifier::FieldIfStar(field) => {
            if star_export(node)
                && let Some(spec) = field_text(node, src, field)
            {
                out.push(site(kind.label, node, spec));
            }
        }
        Specifier::NameIfNoBody => {
            if node.child_by_field_name("body").is_none()
                && let Some(spec) = field_text(node, src, "name")
            {
                out.push(site(kind.label, node, spec));
            }
        }
        Specifier::EachImportTarget => {
            for child in children(node) {
                if let Some(spec) = import_target(child, src) {
                    out.push(site(kind.label, child, spec));
                }
            }
        }
    }
    out.len() > before
}

/// `export * from …` carries a bare `*` token, `export * as ns from …`
/// a `namespace_export` child; the `export_clause` forms carry neither.
fn star_export(node: tree_sitter::Node) -> bool {
    children(node)
        .into_iter()
        .any(|c| matches!(c.kind(), "*" | "namespace_export"))
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
/// whitespace-trimmed. None when empty — a degenerate specifier
/// (`import ""`) drops the site; acceptable pre-resolution because
/// nothing resolvable was referenced, and the 2f unresolved ledger
/// is the honest home for such rows once resolution exists.
fn node_text(node: tree_sitter::Node, src: &[u8]) -> Option<String> {
    let raw = node.utf8_text(src).ok()?;
    let first = raw.split('\n').next().unwrap_or("");
    let spec = first.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    (!spec.is_empty()).then(|| spec.to_string())
}

#[cfg(test)]
#[path = "sites_tests.rs"]
mod tests;
