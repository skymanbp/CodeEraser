//! The document facts of a resolve/1 request (plan v2.33 W2-text stages
//! G and H): what this side reads of the documents the Markdown and HTML
//! rungs ask about, sent with the first request of a batch that holds a
//! Markdown or an HTML site — the walked assets (a link's second
//! candidate set), every walked Markdown file's anchor set as md_slug.rs
//! slugs it (the rendered headings and raw-HTML anchor ids; a page's link
//! into a Markdown document checks its fragment there too), per file of a
//! Markdown reference site its definitions outside code and comments in
//! text order and the labels its reference links write (graph/md.rs's
//! masking and detector), and — when the batch holds an HTML site — per
//! walked HTML file and per HTML site's file the walk did not hold
//! `[path, lang, base, canonical, ogUrl, [[hreflang, href]], [id]]`, each
//! value as the page wrote it (ladder/html_head.rs). The masking, the
//! heading walk, the slugging and the page's syntax-tree walk stay here:
//! their other readers (the tombstone leg on the hook path, docdup's
//! segments, fourclass's section units, the walk's key) have no core
//! replacement yet. Which target a site names, the fold of a label, the
//! first definition that wins, whether a definition is used, the
//! character references, the origin split and the host and tree root of
//! a page's own URL are the core's (CE.Resolve.Md, CE.Resolve.Html,
//! CE.Resolve.HtmlHead).

use crate::graph::ladder::html_head;
use crate::graph::ladder::md::slug::slug_set;
use crate::graph::ladder::{Scope, Site};
use crate::graph::md::{content_lines, detect, is_md_path, ref_definition};
use crate::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

/// The `assets`, `md` and `html` keys of a request whose batch holds
/// Markdown or HTML sites; each file read once per sweep (the memo).
pub fn add(body: &mut Value, scope: &Scope, sites: &[(Lang, &Site)]) {
    if !sites
        .iter()
        .any(|(lang, _)| matches!(lang, Lang::Markdown | Lang::Html))
    {
        return;
    }
    let md: Vec<&Site> = sites
        .iter()
        .filter(|(lang, _)| *lang == Lang::Markdown)
        .map(|(_, s)| *s)
        .collect();
    let tabled: BTreeSet<&str> = md
        .iter()
        .filter(|s| matches!(s.kind, "ref_link" | "ref_def"))
        .map(|s| s.from)
        .collect();
    let slugs = scope.memo.cached("resolve:md-slugs", "", || {
        let slugs: Vec<Value> = scope
            .files
            .iter()
            .filter(|f| is_md_path(f))
            .map(|f| json!([f, slug_set(&read(scope.root, f))]))
            .collect();
        Value::Array(slugs)
    });
    let refs: Vec<Value> = tabled
        .into_iter()
        .map(|from| {
            let table = scope
                .memo
                .cached("resolve:md-refs", from, || refs(scope.root, from));
            (*table).clone()
        })
        .collect();
    body["assets"] = json!(scope.assets);
    body["md"] = json!({ "slugs": *slugs, "refs": refs });
    if let Some(docs) = html_docs(scope, sites) {
        body["html"] = json!({ "docs": docs });
    }
}

/// The pages' rows when the batch holds HTML sites: every walked HTML
/// file (read once per sweep) and each HTML site's file the walk did not
/// hold, by path.
fn html_docs(scope: &Scope, sites: &[(Lang, &Site)]) -> Option<Vec<Value>> {
    let froms: BTreeSet<&str> = sites
        .iter()
        .filter(|(lang, _)| *lang == Lang::Html)
        .map(|(_, s)| s.from)
        .collect();
    if froms.is_empty() {
        return None;
    }
    let walked = scope.memo.cached("resolve:html-docs", "", || {
        let docs: Vec<Value> = scope
            .files
            .iter()
            .filter(|f| Lang::from_path(Path::new(f)) == Some(Lang::Html))
            .map(|f| page(scope.root, f))
            .collect();
        docs
    });
    let mut docs = (*walked).clone();
    let unwalked: Vec<&str> = froms
        .into_iter()
        .filter(|f| !scope.files.contains(*f))
        .collect();
    if !unwalked.is_empty() {
        docs.extend(unwalked.into_iter().map(|f| page(scope.root, f)));
        docs.sort_by(|a, b| a[0].as_str().cmp(&b[0].as_str()));
    }
    Some(docs)
}

/// One page's row; a page that does not read is empty (no head, no id),
/// as the rungs always read it.
fn page(root: &Path, rel: &str) -> Value {
    let text = read(root, rel);
    let head = html_head::read(&text);
    json!([
        rel,
        head.lang,
        head.base,
        head.canonical,
        head.og_url,
        head.alternates,
        html_head::ids(&text)
    ])
}

/// One file's reference facts: `[path, [[label, target]], [label]]`.
fn refs(root: &Path, from: &str) -> Value {
    let text = read(root, from);
    let defs: Vec<(&str, &str)> = content_lines(&text)
        .into_iter()
        .filter(|(_, _, mask)| !mask.first().copied().unwrap_or(false))
        .filter_map(|(_, line, _)| ref_definition(line))
        .collect();
    let used: Vec<String> = detect(&text)
        .into_iter()
        .filter(|s| s.kind == "ref_link")
        .map(|s| s.spec)
        .collect();
    json!([from, defs, used])
}

/// A document's text; one that does not read is empty (no anchor, no
/// definition), as the rungs always read it.
fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_default()
}
