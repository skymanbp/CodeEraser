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
mod tests {
    use super::detect;
    use crate::scan::lang::Lang;

    /// One table drives both checks per language: the expected
    /// (kind, spec) sequence, and the anti-invention rule that every
    /// spec is a substring of its STATEMENT WINDOW — the site line is
    /// the statement head, and a multi-line TS import carries its
    /// full specifier on a later line of the same statement (2c/2d
    /// review F1: 14 frozen zod sites; rust only holds the per-line
    /// form by accident of first-line truncation).
    /// Pinned shapes: `mod foo { … }` is not a site, a plain export
    /// is not a site, one site per Python import target, a
    /// multi-line use keeps ONE site whose spec is the first-line
    /// fragment (module header), a qualified/aliased Haskell import
    /// keeps the bare module name, and `foreign import` is NOT a
    /// site — its anon `import` token shares the kind name (the 3k
    /// D11 collision class) but carries no module field.
    /// (language, source, expected (kind, spec) sequence).
    type Case = (Lang, &'static str, &'static [(&'static str, &'static str)]);

    /// The per-language table, split from its assertion loop at the
    /// E01 fn-length line.
    fn cases() -> [Case; 5] {
        [
            (
                Lang::Python,
                "import a.b, c as d\nfrom .pkg import thing\n",
                &[("import", "a.b"), ("import", "c"), ("import_from", ".pkg")],
            ),
            (
                Lang::TypeScript,
                "import { x } from \"./util\";\nimport {\n  a,\n  b,\n} from \"./multi\";\nexport { y } from './other';\nexport const z = 1;\n",
                &[
                    ("import", "./util"),
                    ("import", "./multi"),
                    ("export_from", "./other"),
                ],
            ),
            (
                Lang::Rust,
                // the #[path] attribute emits NO site of its own — the
                // remap is ladder-side (rs.rs path_attr), which is what
                // keeps the frozen site universe standing across REV 5
                "mod alpha;\n#[path = \"x.rs\"]\nmod beta { fn x() {} }\nuse crate::a::{b, c};\nuse crate::{\n    d,\n    e,\n};\n",
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
            (
                Lang::Haskell,
                "module Main where\n\nimport CE.Alpha\nimport qualified Data.Map as M\nimport Data.List (sort)\n\nforeign import ccall \"math.h sin\" c_sin :: Double -> Double\n",
                &[
                    ("import", "CE.Alpha"),
                    ("import", "Data.Map"),
                    ("import", "Data.List"),
                ],
            ),
        ]
    }

    #[test]
    fn per_language_kinds_specs_and_line_substrings() {
        for (lang, text, want) in cases() {
            let found = detect(text, lang);
            let got: Vec<(&str, &str)> = found.iter().map(|s| (s.kind, s.spec.as_str())).collect();
            assert_eq!(got, *want, "{lang:?}");
            let lines: Vec<&str> = text.lines().collect();
            let stray: Vec<&str> = found
                .iter()
                .filter(|s| {
                    let end = (s.line + 15).min(lines.len());
                    !lines[s.line - 1..end].iter().any(|l| l.contains(&s.spec))
                })
                .map(|s| s.spec.as_str())
                .collect();
            assert!(
                stray.is_empty(),
                "{lang:?}: specs not within their statement window: {stray:?}"
            );
        }
    }

    /// Self-ownership is filtered (Opus review): a `mod foo;` site's
    /// one-line unit is the site itself, so its owner is None, while
    /// a use inside a real function keeps its owner. Same-line sites
    /// get distinct nth ordinals (unique identity for 2c sampling).
    #[test]
    fn owner_not_self_and_nth_ordinals() {
        let text = "mod alpha;\nfn holder() {\n    use crate::x;\n}\n";
        let found = detect(text, Lang::Rust);
        assert_eq!(found[0].kind, "mod_decl");
        assert_eq!(found[0].owner, None, "self-ownership must filter");
        assert_eq!(found[1].kind, "use");
        assert_eq!(found[1].owner.as_deref(), Some("holder/0"));
        let two = detect("[a](./x.md) [b](./y.md)\n", Lang::Markdown);
        assert_eq!(
            two.iter().map(|s| s.nth).collect::<Vec<_>>(),
            vec![0, 1],
            "same-line sites need distinct ordinals"
        );
    }
}
