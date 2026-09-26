//! HTML reference sites (plan v2.30 step 5; booklet §6 and §8): the
//! attributes that name another resource, read off each element's
//! tag as (element, attribute) pairs. A spec-table row names a node
//! kind and a field; it cannot say which ELEMENT an `href` sits on,
//! and a `<link>` even reads a sibling attribute (`rel`) to tell a
//! page from an asset — so, like Java's `type_ref`, the pass is its
//! own and graph/sites.rs dispatches it. A spec is the attribute
//! value as written: `&amp;` and percent escapes decode in the ladder
//! (the anti-invention rule keeps every spec a substring of its
//! line), a `srcset` opens one site per comma-separated candidate
//! (its URL, never the descriptor), and an empty or absent value is a
//! site with an empty spec, kept for the unresolved ledger (O60).

use super::RawSite;
use crate::scan::ast::{self, children};
use crate::scan::html::{attributes, tag_name};
use tree_sitter::Node;

/// `element attribute label` per row: the attributes that reference a
/// page (`href`, a form's `action`) or an asset (`src`, `srcset`, an
/// object's `data`, a video's `poster`), by the element they sit on
/// (the HTML Living Standard's URL-valued attributes; booklet §6).
/// `<link href>` is the one row that reads more than its own
/// attribute: a stylesheet, icon or preload link names an asset
/// (`link_asset`, `asset_rel`), any other relationship — alternate,
/// canonical, next — names a page (`href`).
const ROWS: &str = "\
a href href
area href href
base href href
use href href
link href href
form action action
script src src
img src src
img srcset srcset
source src src
source srcset srcset
iframe src src
embed src src
track src src
video src src
video poster src
audio src src
object data src";

/// The `rel` tokens under which a `<link>` names an asset (booklet
/// §8: stylesheet | icon | preload; `modulepreload`, `prefetch` and
/// `manifest` fetch a resource the same way, and every icon
/// relationship — `icon`, `shortcut icon`, `apple-touch-icon`,
/// `mask-icon` — ends in `icon`).
fn asset_rel(rel: &str) -> bool {
    rel.split_ascii_whitespace().any(|t| {
        let t = t.to_ascii_lowercase();
        t.ends_with("icon")
            || matches!(
                t.as_str(),
                "stylesheet" | "preload" | "modulepreload" | "prefetch" | "manifest"
            )
    })
}

/// The frozen label one attribute of one element opens a site under,
/// or None when no row names the pair.
fn label_of(element: &str, attr: &str, rel: Option<&str>) -> Option<&'static str> {
    let row = ROWS
        .lines()
        .map(|row| {
            let mut w = row.split_ascii_whitespace();
            (w.next(), w.next(), w.next())
        })
        .find(|(el, at, _)| *el == Some(element) && *at == Some(attr))?;
    if element == "link" && rel.is_some_and(asset_rel) {
        return Some("link_asset");
    }
    row.2
}

/// Every site of one document, in document order.
pub(super) fn sites(root: Node, src: &[u8]) -> Vec<RawSite> {
    let mut out = Vec::new();
    for node in ast::preorder(root, |_| true, children) {
        if matches!(node.kind(), "start_tag" | "self_closing_tag") {
            tag_sites(node, src, &mut out);
        }
    }
    out
}

/// The sites one tag opens: each of its attributes a row names for the
/// element, the `<link>` relationship read first.
fn tag_sites(tag: Node, src: &[u8], out: &mut Vec<RawSite>) {
    let Some(element) = tag_name(tag, src) else {
        return;
    };
    let attrs = attributes(tag, src);
    let rel = attrs.iter().find(|a| a.name == "rel").map(|a| a.text(src));
    for attr in &attrs {
        let Some(label) = label_of(&element, &attr.name, rel.as_deref()) else {
            continue;
        };
        match (label, attr.value) {
            ("srcset", Some(value)) => srcset_sites(value, src, out),
            _ => out.push(RawSite::at(
                label,
                attr.row(src).unwrap_or(tag.start_position().row) + 1,
                attr.text(src),
            )),
        }
    }
}

/// One site per `srcset` candidate: the URL before the candidate's
/// descriptor, on the line it is written on (the list may wrap), an
/// empty candidate (a trailing comma) opening none.
fn srcset_sites(value: Node, src: &[u8], out: &mut Vec<RawSite>) {
    let Ok(text) = value.utf8_text(src) else {
        return;
    };
    let first_row = value.start_position().row;
    let mut at = 0;
    for candidate in text.split(',') {
        let lead = candidate.len() - candidate.trim_start().len();
        let row = first_row + text[..at + lead].matches('\n').count();
        at += candidate.len() + 1;
        if let Some(url) = candidate.split_whitespace().next() {
            out.push(RawSite::at("srcset", row + 1, url.to_string()));
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/sites/html.rs"]
mod tests;
