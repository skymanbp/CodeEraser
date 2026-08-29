//! Function-boundary segmentation for L1 (plan §4.3: tree-sitter
//! symbol table). Code languages reuse the scan module's extractor;
//! Markdown segments on ATX headings; lines outside any unit belong
//! to the file's top level.

use super::visibility;
use crate::mention::conv;
use crate::scan::ast;
use crate::scan::functions;
use crate::scan::lang::Lang;
use crate::scan::spec;

/// One alignment unit, owned (no tree lifetime): `key` is the
/// cross-version identity (name + arity for code, heading text for
/// Markdown), spans are 1-based inclusive line ranges, and the two
/// words carry the declaration's OWN facts — `vis` its visibility
/// bits (visibility/), `conv` the AST half of its convention-category
/// word (mention/conv) — read here because this is where the
/// declaration node is still in hand, and persisted as `symbols.flags`
/// and `symbols.conv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub key: String,
    pub start_line: usize,
    pub end_line: usize,
    pub vis: i64,
    pub conv: i64,
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
    let facts = conv::file_facts(tree.root_node(), src, lang);
    // Go receiver qualification ("(T) add") rides f.name from the
    // extraction root (scan::functions::name_of) — one throat for
    // metric names, these keys and the baseline entities (M5-close
    // review D4 retired the post-pass that lived here).
    let mut units: Vec<Unit> = functions::extract(tree.root_node(), src, spec::spec(lang))
        .into_iter()
        .map(|f| Unit {
            key: format!("{}/{}", f.name, f.params),
            start_line: f.start_line,
            end_line: f.end_line,
            vis: visibility::bits(f.node, src, lang),
            conv: conv::ast_bits(f.node, src, lang, &facts),
        })
        .collect();
    units.extend(extra_units(tree.root_node(), src, lang, &facts));
    units
}

/// Named non-function units (consts, types, …) from the kinds
/// register — relocation reporting needs their names too; the M1
/// function metrics do not, which is why this list lives in
/// fourclass::kinds and not scan/spec. The named register and the
/// Rust impl form are disjoint, so a node keys at most once.
fn extra_units(
    root: tree_sitter::Node,
    src: &[u8],
    lang: Lang,
    facts: &conv::FileFacts,
) -> Vec<Unit> {
    let kinds = super::kinds::extra(lang);
    let mut out = Vec::new();
    if kinds.is_empty() {
        return out;
    }
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(key) = named_key(node, src, kinds).or_else(|| impl_key(node, src, lang)) {
            out.push(Unit {
                key,
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                vis: visibility::bits(node, src, lang),
                conv: conv::ast_bits(node, src, lang, facts),
            });
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                stack.push(child);
            }
        }
    }
    out
}

fn named_key(node: tree_sitter::Node, src: &[u8], kinds: &[&str]) -> Option<String> {
    if !kinds.contains(&node.kind()) {
        return None;
    }
    // an instance of a family names the family, never a new type
    if node
        .parent()
        .is_some_and(|p| super::kinds::REDECLARING.contains(&p.kind()))
    {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    Some(String::from_utf8_lossy(&src[name.byte_range()]).into_owned())
}

/// Key for a Rust impl block: `impl Foo`, or `impl Advisor for Foo`
/// when a trait is named. Impl blocks carry no `name` field — without
/// a unit for them, methods of two different impls look top-level and
/// their shared name/arity key becomes false stacking evidence (attack
/// review 2026-08-11 F7); the trait keeps a type's inherent and trait
/// impls distinct (the FPR replay caught the unqualified key colliding
/// on exactly that shape). The one-row dispatch table this replaced
/// never grew a second row (v2.18 subtraction batch).
fn impl_key(node: tree_sitter::Node, src: &[u8], lang: Lang) -> Option<String> {
    if lang != Lang::Rust || node.kind() != "impl_item" {
        return None;
    }
    let field = |name| {
        node.child_by_field_name(name)
            .map(|n| String::from_utf8_lossy(&src[n.byte_range()]).into_owned())
    };
    let ty = field("type")?;
    Some(match field("trait") {
        Some(tr) => format!("impl {tr} for {ty}"),
        None => format!("impl {ty}"),
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
                vis: visibility::MARKDOWN_VIS,
                conv: 0, // a heading is outside the mention domain (RG9)
            });
        }
    }
    if let Some(last) = out.last_mut() {
        last.end_line = total;
    }
    out
}

/// THE nth assignment (schema v5, F2): occurrence order by
/// start_line within each key group — `(path, key)` alone is not an
/// identity (same-key Rust methods across impl blocks collide in one
/// file). The graph `symbols` rows and the dedup `unitsig` cache
/// both persist (key, nth) and MUST agree, so both call this one
/// throat instead of re-deriving the order.
pub fn with_nth(units: &[Unit]) -> Vec<(&Unit, i64)> {
    let mut ordered: Vec<&Unit> = units.iter().collect();
    ordered.sort_by_key(|u| (u.key.as_str(), u.start_line, u.end_line));
    let mut out = Vec::with_capacity(ordered.len());
    let mut nth = 0i64;
    let mut prev: Option<&str> = None;
    for u in ordered {
        nth = if prev == Some(u.key.as_str()) {
            nth + 1
        } else {
            0
        };
        prev = Some(u.key.as_str());
        out.push((u, nth));
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
#[path = "../../tests/unit/fourclass/units.rs"]
mod tests;
