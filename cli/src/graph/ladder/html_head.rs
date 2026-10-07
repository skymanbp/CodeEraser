//! What one HTML document's syntax tree says about the page, read for
//! the HTML rungs (plan v2.30 step 5, booklet §8). Since plan v2.33
//! W2-text stage H the rungs are the core's (CE.Resolve.Html) and so is
//! every reading of these facts — the character references decoded, the
//! origin split off a URL, the host and tree root of the URL that names
//! the page (CE.Resolve.HtmlHead); this side walks the tree and sends what
//! it wrote, as written (graph/resolve/markdown.rs): the `<html lang>`, the
//! first `<base href>`, the first canonical link, the first `og:url`, the
//! hreflang alternates, and the `id` set a page links into — the last also
//! hashed into the resolve_key as a Markdown slug set is
//! (dedup/walkidx.rs), so it stays here with its hash.

use crate::fourclass::units;
use crate::scan::ast::{self, children};
use crate::scan::html::{attribute, tag_name, tag_of};
use crate::scan::lang::Lang;

/// One document's head facts as written: the first non-empty value of
/// each (an empty attribute is no value), and every alternate link that
/// carries both an `hreflang` and an `href`, in document order.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Head {
    pub lang: Option<String>,
    pub base: Option<String>,
    pub canonical: Option<String>,
    pub og_url: Option<String>,
    pub alternates: Vec<(String, String)>,
}

/// The head facts of a document's text.
pub fn read(text: &str) -> Head {
    let mut head = Head::default();
    let Some(tree) = ast::parse_lang(text, Lang::Html) else {
        return head;
    };
    let src = text.as_bytes();
    for node in ast::preorder(tree.root_node(), |_| true, children) {
        let Some(tag) = tag_of(node) else { continue };
        let Some(name) = tag_name(tag, src) else {
            continue;
        };
        let at = |attr: &str| attribute(tag, src, attr).filter(|v| !v.is_empty());
        let rel = |token: &str| {
            at("rel").is_some_and(|r| {
                r.split_ascii_whitespace()
                    .any(|t| t.eq_ignore_ascii_case(token))
            })
        };
        match name.as_str() {
            "html" if head.lang.is_none() => head.lang = at("lang"),
            "base" if head.base.is_none() => head.base = at("href"),
            "meta" if head.og_url.is_none() && at("property").as_deref() == Some("og:url") => {
                head.og_url = at("content");
            }
            "link" if head.canonical.is_none() && rel("canonical") => head.canonical = at("href"),
            "link" if rel("alternate") => {
                if let (Some(hreflang), Some(href)) = (at("hreflang"), at("href")) {
                    head.alternates.push((hreflang, href));
                }
            }
            _ => {}
        }
    }
    head
}

/// The `id` values of a document, in document order — the anchor set
/// a cross-page fragment is validated against (the section units'
/// own reading, fourclass/units.rs, `#` dropped).
pub fn ids(text: &str) -> Vec<String> {
    units::segments(text, Lang::Html)
        .into_iter()
        .filter_map(|u| u.key.strip_prefix('#').map(str::to_string))
        .collect()
}

/// The hash of the consulted projection, `ids` — the resolve_key
/// input for a page (dedup/walkidx.rs): an id edit anywhere re-fires
/// the sweep, a body edit does not (the md slug_hash discipline).
pub fn id_hash(text: &str) -> u64 {
    let joined: Vec<u8> = ids(text)
        .iter()
        .flat_map(|id| id.bytes().chain(std::iter::once(0)))
        .collect();
    crate::dedup::tokens::fnv1a(&joined)
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/html_ids.rs"]
mod tests;
