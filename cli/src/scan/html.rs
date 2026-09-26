//! Shared readers of an HTML tag (tree-sitter-html 0.23.2, probed
//! 2026-09-24 — scripts/tsprobe): the element a `start_tag` /
//! `self_closing_tag` opens and the attributes it carries. Three
//! consumers read a tag — the section units (fourclass/units.rs, an
//! element's `id`), the reference sites (graph/sites/html.rs, the
//! attributes naming another resource) and the docdup segments
//! (docdup/html.rs, which elements are prose) — and one reader keeps
//! them agreeing on what a tag name and an attribute are: both are
//! ASCII-case-insensitive in HTML, so they are read lowercased, and an
//! attribute's value sits either under a `quoted_attribute_value` or
//! bare, or is absent (`<a href>`), which the standard reads as the
//! empty string.

use tree_sitter::Node;

/// The tag node of an element (its first child), when the node is an
/// element: a plain `element`, a `script_element` or a `style_element`.
pub fn tag_of(element: Node<'_>) -> Option<Node<'_>> {
    element
        .child(0)
        .filter(|t| matches!(t.kind(), "start_tag" | "self_closing_tag"))
}

/// The element's name, lowercased, off its tag node.
pub fn tag_name(tag: Node<'_>, src: &[u8]) -> Option<String> {
    let name = crate::scan::ast::children(tag)
        .into_iter()
        .find(|c| c.kind() == "tag_name")?;
    Some(name.utf8_text(src).ok()?.to_ascii_lowercase())
}

/// Whether the element's name is one of a space-separated table.
pub fn tag_in(element: Node<'_>, src: &[u8], table: &str) -> bool {
    tag_of(element)
        .and_then(|t| tag_name(t, src))
        .is_some_and(|name| table.split_ascii_whitespace().any(|t| t == name))
}

/// One attribute of a tag: its name lowercased, and its value node —
/// the `attribute_value` under a `quoted_attribute_value` or the bare
/// one — or None for a valueless attribute.
pub struct Attr<'t> {
    pub name: String,
    pub value: Option<Node<'t>>,
}

impl Attr<'_> {
    /// The value as written — its surrounding whitespace stripped, as
    /// the standard strips a URL attribute's, then cut at a line break
    /// like every specifier (a wrapped value is anchored by its first
    /// line); a valueless attribute is the empty string.
    pub fn text(&self, src: &[u8]) -> String {
        self.value
            .and_then(|v| v.utf8_text(src).ok())
            .and_then(|t| t.trim().lines().next())
            .unwrap_or("")
            .trim_end()
            .to_string()
    }

    /// The row the value's first non-blank byte sits on — a quoted
    /// value may open on the line after its quote, and a site is
    /// anchored where its spec is written — or None when valueless.
    pub fn row(&self, src: &[u8]) -> Option<usize> {
        let v = self.value?;
        let text = v.utf8_text(src).ok()?;
        let lead = text.len() - text.trim_start().len();
        Some(v.start_position().row + text[..lead].matches('\n').count())
    }
}

/// Every attribute of a tag, in document order.
pub fn attributes<'t>(tag: Node<'t>, src: &[u8]) -> Vec<Attr<'t>> {
    crate::scan::ast::children(tag)
        .into_iter()
        .filter(|c| c.kind() == "attribute")
        .filter_map(|a| {
            let kids = crate::scan::ast::children(a);
            let name = kids.iter().find(|c| c.kind() == "attribute_name")?;
            let value = kids.iter().find_map(|c| match c.kind() {
                "attribute_value" => Some(*c),
                "quoted_attribute_value" => crate::scan::ast::children(*c)
                    .into_iter()
                    .find(|q| q.kind() == "attribute_value"),
                _ => None,
            });
            Some(Attr {
                name: name.utf8_text(src).ok()?.to_ascii_lowercase(),
                value,
            })
        })
        .collect()
}

/// The value text of one named attribute on a tag: None when the tag
/// carries no such attribute, Some("") when it carries it valueless.
pub fn attribute(tag: Node<'_>, src: &[u8], name: &str) -> Option<String> {
    attributes(tag, src)
        .into_iter()
        .find(|a| a.name == name)
        .map(|a| a.text(src))
}
