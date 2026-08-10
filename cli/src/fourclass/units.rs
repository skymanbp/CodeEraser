//! Function-boundary segmentation for L1 (plan §4.3: tree-sitter
//! symbol table). Code languages reuse the scan module's extractor;
//! Markdown segments on ATX headings; lines outside any unit belong
//! to the file's top level.

use crate::scan::functions;
use crate::scan::lang::Lang;
use crate::scan::spec;

/// One alignment unit, owned (no tree lifetime): `key` is the
/// cross-version identity (name + arity for code, heading text for
/// Markdown), spans are 1-based inclusive line ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub key: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn segments(text: &str, lang: Lang) -> Vec<Unit> {
    match lang.grammar() {
        Some(grammar) => code_segments(text, lang, grammar),
        None => markdown_segments(text),
    }
}

fn code_segments(text: &str, lang: Lang, grammar: tree_sitter::Language) -> Vec<Unit> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&grammar).is_err() {
        return Vec::new(); // no segmentation: everything is toplevel
    }
    let Some(tree) = parser.parse(text, None) else {
        return Vec::new();
    };
    functions::extract(tree.root_node(), text.as_bytes(), spec::spec(lang))
        .into_iter()
        .map(|f| Unit {
            key: format!("{}/{}", f.name, f.params),
            start_line: f.start_line,
            end_line: f.end_line,
        })
        .collect()
}

/// ATX headings open a section that runs to the next heading of any
/// level (nesting by level is a reporting nicety L1 does not need).
fn markdown_segments(text: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    let mut total = 0;
    for (i, line) in text.lines().enumerate() {
        total = i + 1;
        let t = line.trim_start();
        if t.starts_with('#') && t.trim_start_matches('#').starts_with(' ') {
            if let Some(prev) = out.last_mut() {
                prev.end_line = i; // previous section ends above this heading
            }
            out.push(Unit {
                key: t.trim_matches('#').trim().to_string(),
                start_line: i + 1,
                end_line: i + 1,
            });
        }
    }
    if let Some(last) = out.last_mut() {
        last.end_line = total;
    }
    out
}

/// The innermost unit containing `line` (1-based), or None = toplevel.
pub fn owner(units: &[Unit], line: usize) -> Option<&Unit> {
    units
        .iter()
        .filter(|u| u.start_line <= line && line <= u.end_line)
        .min_by_key(|u| u.end_line - u.start_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyed(src: &str, lang: Lang) -> (Vec<Unit>, Vec<String>) {
        let units = segments(src, lang);
        let keys = units.iter().map(|u| u.key.clone()).collect();
        (units, keys)
    }

    #[test]
    fn python_functions_become_units() {
        let (units, keys) = keyed(
            "def alpha(a, b):\n    return a\n\ndef beta():\n    pass\n",
            Lang::Python,
        );
        assert_eq!(keys, ["alpha/2", "beta/0"]);
        assert_eq!(owner(&units, 2).unwrap().key, "alpha/2");
        assert_eq!(owner(&units, 3), None); // blank line between defs
    }

    #[test]
    fn markdown_sections_split_on_headings() {
        let (units, keys) = keyed("intro\n# One\nbody\n## Two\nmore\n", Lang::Markdown);
        assert_eq!(keys, ["One", "Two"]);
        assert_eq!(owner(&units, 1), None); // preamble is toplevel
        assert_eq!(owner(&units, 3).unwrap().key, "One");
        assert_eq!(owner(&units, 5).unwrap().key, "Two");
    }

    #[test]
    fn nested_functions_resolve_to_innermost() {
        let src = "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}\n";
        let units = segments(src, Lang::Rust);
        assert_eq!(owner(&units, 3).unwrap().key, "inner/0");
        assert_eq!(owner(&units, 5).unwrap().key, "outer/0");
    }
}
