//! What one HTML document says about itself, read for the HTML rungs
//! (html.rs; plan v2.30 step 5, booklet §8): where it is served — its
//! own `<link rel="canonical">`, `<meta property="og:url">` or the
//! `hreflang` alternate that names itself — from which the tree
//! directory that origin serves is derived; the `<base href>` its
//! relative references resolve against; and the `id` set a page links
//! into, hashed into the resolve_key as a Markdown slug set is
//! (dedup/walkidx.rs). Plus the two URL readings the rungs share: a
//! character reference decoded (`&amp;`) and an origin split off an
//! absolute URL. Nothing here consults the tree: every fact is the
//! document's own text.

use crate::fourclass::units;
use crate::scan::ast::{self, children};
use crate::scan::html::{attribute, tag_name, tag_of};
use crate::scan::lang::Lang;

/// The facts of one document the rungs read. `base` is the first
/// `<base href>` as written. `host` is the host of the document's own
/// served URL and `root` the tree directory that host serves — derived,
/// never declared: the URL's path names the document itself (`/how/`
/// serves `how/index.html`), so the directory the page's own path is
/// cut down to is the deployment root; a URL that does not name the
/// page (a canonical pointing elsewhere) derives nothing.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub base: Option<String>,
    pub host: Option<String>,
    pub root: Option<String>,
}

/// The document at tree path `from`, read off its text.
pub fn read(from: &str, text: &str) -> Document {
    let Some(tree) = ast::parse_lang(text, Lang::Html) else {
        return Document::default();
    };
    let src = text.as_bytes();
    let (mut lang, mut base, mut canonical, mut og_url) = (None, None, None, None);
    let mut alternates: Vec<(String, String)> = Vec::new();
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
            "html" if lang.is_none() => lang = at("lang"),
            "base" if base.is_none() => base = at("href"),
            "meta" if og_url.is_none() && at("property").as_deref() == Some("og:url") => {
                og_url = at("content");
            }
            "link" if canonical.is_none() && rel("canonical") => canonical = at("href"),
            "link" if rel("alternate") => {
                if let (Some(hreflang), Some(href)) = (at("hreflang"), at("href")) {
                    alternates.push((hreflang, href));
                }
            }
            _ => {}
        }
    }
    let own = self_url(from, lang.as_deref(), canonical.or(og_url), &alternates);
    Document {
        base: base.map(|b| decode_refs(&b)),
        host: own.as_ref().map(|(host, _)| host.clone()),
        root: own.map(|(_, root)| root),
    }
}

/// The (host, root) of the URL that names the page: its canonical (or,
/// without one, its og:url) when that names it — a canonical pointing
/// elsewhere is the page saying it has no URL of its own — else the
/// hreflang alternate in the page's own language (`<html lang>`,
/// primary subtag; every alternate when none is declared) that shares
/// the longest path with it: the other language's alternate names the
/// other page, and only its tail could match this one.
fn self_url(
    from: &str,
    lang: Option<&str>,
    declared: Option<String>,
    alternates: &[(String, String)],
) -> Option<(String, String)> {
    if let Some(url) = declared {
        return served_from(from, &decode_refs(&url));
    }
    let primary = |tag: &str| tag.split('-').next().unwrap_or("").to_ascii_lowercase();
    alternates
        .iter()
        .filter(|(hreflang, _)| lang.is_none_or(|l| primary(l) == primary(hreflang)))
        .filter_map(|(_, href)| served_from(from, &decode_refs(href)))
        .min_by_key(|(_, root)| root.len())
}

/// The (host, root) a URL yields when its path names the page at
/// `from`: the tree paths the URL path could serve (a directory URL its
/// index page, an extensionless one that page or `name.html`), the one
/// that ends `from` at a directory boundary cutting the root off it.
fn served_from(from: &str, url: &str) -> Option<(String, String)> {
    let (host, path) = split_origin(url)?;
    let path = path.split(['?', '#']).next().unwrap_or("");
    let p = super::md::slug::percent_decode(path.trim_start_matches('/'));
    let candidates: Vec<String> = if p.is_empty() || p.ends_with('/') {
        vec![format!("{p}index.html"), format!("{p}index.htm")]
    } else if p.rsplit('/').next().is_some_and(|last| last.contains('.')) {
        vec![p.clone()]
    } else {
        ["/index.html", "/index.htm", ".html", ".htm"]
            .iter()
            .map(|tail| format!("{p}{tail}"))
            .collect()
    };
    let root = candidates.iter().find_map(|c| {
        if from == c {
            Some(String::new())
        } else {
            from.strip_suffix(c.as_str())
                .and_then(|head| head.strip_suffix('/'))
                .map(str::to_string)
        }
    })?;
    Some((host, root))
}

/// An absolute (`scheme://host/path`) or protocol-relative (`//host/
/// path`) URL split into its host, lowercased, and its path with the
/// query and fragment still attached — None for anything else, a
/// `mailto:` or `javascript:` scheme included (md::is_scheme reads
/// those; a host they do not carry).
pub fn split_origin(url: &str) -> Option<(String, &str)> {
    let rest = if let Some(rest) = url.strip_prefix("//") {
        rest
    } else {
        let (scheme, rest) = url.split_once("://")?;
        let alnum = |c: char| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.');
        if scheme.is_empty() || !scheme.chars().all(alnum) {
            return None;
        }
        rest
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let host = authority.rsplit('@').next().unwrap_or(authority);
    Some((host.to_ascii_lowercase(), &rest[end..]))
}

/// The character references an attribute value carries decoded
/// (`&amp;` above all — a query's `&` is written so in valid HTML —
/// with the other four named ones and the numeric forms); an unknown
/// or unterminated reference stays as written.
pub fn decode_refs(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        match rest[1..].find(';').and_then(|j| decoded(&rest[1..j + 2])) {
            Some((c, len)) => {
                out.push(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// One reference body (`amp;`, `#38;`, `#x26;`) as its character and
/// the width of the whole reference including the `&`.
fn decoded(body: &str) -> Option<(char, usize)> {
    let name = body.strip_suffix(';')?;
    let c = match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        _ => {
            let digits = name.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse().ok()?,
            };
            char::from_u32(code).filter(|c| *c != '\0')?
        }
    };
    Some((c, body.len() + 1))
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
#[path = "../../../tests/unit/graph/ladder/html_head_tests.rs"]
mod tests;
