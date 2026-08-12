//! Resolution-free reference-site detection — the frozen sample
//! universe of the M5-2 precision instrument (design brief §5: the
//! universe is SITES, not edges, so the resolver can never choose
//! its own precision denominator). Detection needs only the
//! spec-table node kinds; no path is ever consulted.
//!
//! Multi-line specifiers (a rustfmt-folded `use x::{a, b, …}`) keep
//! ONE site whose spec is the first line's fragment: the spec string
//! is a location anchor and an anti-invention check (the slice gate
//! asserts it is a substring of its source line) — resolution at 2f
//! re-reads the AST and does not parse spec strings.

use super::md;
use super::spec::{SiteKind, Specifier, sites as site_table};
use crate::fourclass::units;
use crate::scan::ast::{self, children};
use crate::scan::lang::Lang;

/// One detected site, before path attachment.
pub struct RawSite {
    pub kind: &'static str,
    pub line: usize,
    pub spec: String,
    pub owner: Option<String>,
}

impl RawSite {
    pub fn md(kind: &'static str, line: usize, spec: String) -> Self {
        RawSite {
            kind,
            line,
            spec,
            owner: None,
        }
    }
}

/// Detect every reference site in one document.
pub fn detect(text: &str, lang: Lang) -> Vec<RawSite> {
    let mut found = match lang.grammar() {
        Some(grammar) => code_sites(text, lang, grammar),
        None => md::detect(text),
    };
    let owners = units::segments(text, lang);
    for site in &mut found {
        site.owner = units::owner(&owners, site.line).map(|u| u.key.clone());
    }
    found
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
        if let Some(kind) = table.iter().find(|k| k.node == node.kind()) {
            emit(node, src, kind, &mut out);
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

fn emit(node: tree_sitter::Node, src: &[u8], kind: &SiteKind, out: &mut Vec<RawSite>) {
    match &kind.via {
        Specifier::Field(field) | Specifier::FieldIfPresent(field) => {
            if let Some(spec) = field_text(node, src, field) {
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
        spec,
        owner: None,
    }
}

fn field_text(node: tree_sitter::Node, src: &[u8], field: &str) -> Option<String> {
    node_text(node.child_by_field_name(field)?, src)
}

/// Node text as a spec string: quotes trimmed (string-literal
/// specifiers), truncated at the first newline (see module header),
/// whitespace-trimmed. None when empty.
fn node_text(node: tree_sitter::Node, src: &[u8]) -> Option<String> {
    let raw = node.utf8_text(src).ok()?;
    let first = raw.split('\n').next().unwrap_or("");
    let spec = first.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    (!spec.is_empty()).then(|| spec.to_string())
}

#[cfg(test)]
mod tests {
    use super::detect;
    use crate::scan::lang::Lang;

    /// One table drives both checks per language: the expected
    /// (kind, spec) sequence, and the anti-invention rule that every
    /// spec is a substring of its source line (2b exit criterion).
    /// Pinned shapes: `mod foo { … }` is not a site, a plain export
    /// is not a site, one site per Python import target, and a
    /// multi-line use keeps ONE site whose spec is the first-line
    /// fragment (module header).
    /// (language, source, expected (kind, spec) sequence).
    type Case = (Lang, &'static str, &'static [(&'static str, &'static str)]);

    #[test]
    fn per_language_kinds_specs_and_line_substrings() {
        let cases: [Case; 4] = [
            (
                Lang::Python,
                "import a.b, c as d\nfrom .pkg import thing\n",
                &[("import", "a.b"), ("import", "c"), ("import_from", ".pkg")],
            ),
            (
                Lang::TypeScript,
                "import { x } from \"./util\";\nexport { y } from './other';\nexport const z = 1;\n",
                &[("import", "./util"), ("export_from", "./other")],
            ),
            (
                Lang::Rust,
                "mod alpha;\nmod beta { fn x() {} }\nuse crate::a::{b, c};\nuse crate::{\n    d,\n    e,\n};\n",
                &[
                    ("mod_decl", "alpha"),
                    ("use", "crate::a::{b, c}"),
                    ("use", "crate::{"),
                ],
            ),
            (
                Lang::Go,
                "package main\n\nimport (\n\t\"fmt\"\n\t\"github.com/x/y\"\n)\n",
                &[("import", "fmt"), ("import", "github.com/x/y")],
            ),
        ];
        for (lang, text, want) in cases {
            let found = detect(text, lang);
            let got: Vec<(&str, &str)> = found.iter().map(|s| (s.kind, s.spec.as_str())).collect();
            assert_eq!(got, *want, "{lang:?}");
            let lines: Vec<&str> = text.lines().collect();
            for s in &found {
                assert!(
                    lines[s.line - 1].contains(&s.spec),
                    "{lang:?}: spec {:?} not in line {:?}",
                    s.spec,
                    lines[s.line - 1]
                );
            }
        }
    }
}
