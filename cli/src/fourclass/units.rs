//! Function-boundary segmentation for L1 (plan §4.3: tree-sitter
//! symbol table). Code languages reuse the scan module's extractor;
//! Markdown segments on ATX headings; lines outside any unit belong
//! to the file's top level.

use crate::scan::ast;
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
    let Some(tree) = ast::parse(text, &grammar) else {
        return Vec::new(); // no segmentation: everything is toplevel
    };
    let src = text.as_bytes();
    let mut units: Vec<Unit> = functions::extract(tree.root_node(), src, spec::spec(lang))
        .into_iter()
        .map(|f| Unit {
            key: format!("{}/{}", f.name, f.params),
            start_line: f.start_line,
            end_line: f.end_line,
        })
        .collect();
    units.extend(extra_units(tree.root_node(), src, lang));
    if lang == Lang::Go {
        qualify_go_methods(tree.root_node(), src, &mut units);
    }
    units
}

/// Prefix Go method keys with their receiver type: `(T) add/1` and
/// `(U) add/1` are different identities. Go has no containing unit
/// node, so without this two same-name receivers collide into one
/// top-level key — false stacking evidence (attack review F7).
fn qualify_go_methods(root: tree_sitter::Node, src: &[u8], units: &mut [Unit]) {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "method_declaration"
            && let Some(recv) = receiver_type(node, src)
        {
            let line = node.start_position().row + 1;
            for u in units.iter_mut().filter(|u| u.start_line == line) {
                u.key = format!("({recv}) {}", u.key);
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                stack.push(child);
            }
        }
    }
}

/// The receiver's type text (`T`, `*U`) — the identity part; the
/// binding name is deliberately excluded so renaming `(t T)` to
/// `(x T)` keeps the unit's cross-version key stable.
fn receiver_type(method: tree_sitter::Node, src: &[u8]) -> Option<String> {
    let recv = method.child_by_field_name("receiver")?;
    for i in 0..recv.child_count() {
        if let Some(param) = recv.child(i as u32)
            && param.kind() == "parameter_declaration"
            && let Some(ty) = param.child_by_field_name("type")
        {
            return Some(String::from_utf8_lossy(&src[ty.byte_range()]).into_owned());
        }
    }
    None
}

/// Named non-function units (consts, types, …) from the kinds
/// register — relocation reporting needs their names too; the M1
/// function metrics do not, which is why this list lives in
/// fourclass::kinds and not scan/spec.
fn extra_units(root: tree_sitter::Node, src: &[u8], lang: Lang) -> Vec<Unit> {
    let kinds = super::kinds::extra(lang);
    let typed = super::kinds::typed(lang);
    let mut out = Vec::new();
    if kinds.is_empty() && typed.is_empty() {
        return out;
    }
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(key) = named_key(node, src, kinds) {
            out.push(unit_at(node, key));
        }
        if let Some(key) = typed_key(node, src, typed) {
            out.push(unit_at(node, key));
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                stack.push(child);
            }
        }
    }
    out
}

fn unit_at(node: tree_sitter::Node, key: String) -> Unit {
    Unit {
        key,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
    }
}

fn named_key(node: tree_sitter::Node, src: &[u8], kinds: &[&str]) -> Option<String> {
    if !kinds.contains(&node.kind()) {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    Some(String::from_utf8_lossy(&src[name.byte_range()]).into_owned())
}

/// Key for a typed-kind unit: `impl Foo`, or `impl Advisor for Foo`
/// when the qualifier field is present (kinds::typed's why).
fn typed_key(
    node: tree_sitter::Node,
    src: &[u8],
    typed: &[(&str, &str, &str, &str)],
) -> Option<String> {
    let (_, field, qual_field, prefix) = typed.iter().find(|(k, ..)| *k == node.kind())?;
    let name = node.child_by_field_name(field)?;
    let text = String::from_utf8_lossy(&src[name.byte_range()]);
    Some(match node.child_by_field_name(qual_field) {
        Some(q) => format!(
            "{prefix} {} for {text}",
            String::from_utf8_lossy(&src[q.byte_range()])
        ),
        None => format!("{prefix} {text}"),
    })
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

    /// The owning unit key of `line` in Rust `src` ("" = toplevel).
    fn rust_owner(src: &str, line: usize) -> String {
        let units = segments(src, Lang::Rust);
        owner(&units, line)
            .map(|u| u.key.clone())
            .unwrap_or_default()
    }

    #[test]
    fn nested_functions_resolve_to_innermost() {
        let src = "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}\n";
        assert_eq!(rust_owner(src, 3), "inner/0");
        assert_eq!(rust_owner(src, 5), "outer/0");
    }

    /// Attack review F7: impl blocks are units (methods are span-
    /// contained, not top-level), and Go methods carry their receiver
    /// type in the key.
    #[test]
    fn impl_blocks_contain_their_methods() {
        let src = "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n\
                   impl Show for A {\n    fn show(&self) {}\n}\n";
        assert_eq!(rust_owner(src, 2), "add/1");
        let units = segments(src, Lang::Rust);
        let mut impls: Vec<&str> = units
            .iter()
            .filter(|u| u.key.starts_with("impl "))
            .map(|u| u.key.as_str())
            .collect();
        impls.sort_unstable(); // extraction order is not part of the contract
        // the trait qualifier keeps a type's inherent and trait impls
        // distinct (the FPR replay caught them colliding)
        assert_eq!(impls, ["impl A", "impl B", "impl Show for A"]);
    }

    #[test]
    fn go_method_keys_carry_the_receiver_type() {
        let src = "func (t T) add(x int) {}\nfunc (u *U) add(x int) {}\nfunc free(x int) {}\n";
        let (_, keys) = keyed(src, Lang::Go);
        assert!(keys.contains(&"(T) add/1".to_string()), "keys: {keys:?}");
        assert!(keys.contains(&"(*U) add/1".to_string()), "keys: {keys:?}");
        assert!(keys.contains(&"free/1".to_string()), "keys: {keys:?}");
    }

    #[test]
    fn named_non_function_units_are_registered() {
        // the register's CLASSES case: a relocated pub const must be
        // attributable by name (R-L2-6's 1-in-35 structural hole)
        let src =
            "pub const CLASSES: [&str; 4] = [\n    \"a\",\n];\n\nstruct Row {\n    id: u64,\n}\n";
        assert_eq!(rust_owner(src, 2), "CLASSES");
        assert_eq!(rust_owner(src, 6), "Row");
    }
}
