//! The Markdown facts of a resolve/1 request (plan v2.33 W2-text stage
//! G): what this side reads of the documents the Markdown rungs ask
//! about, sent with the first request of a batch that holds a Markdown
//! site — the walked assets (a link's second candidate set), every
//! walked Markdown file's anchor set as md_slug.rs slugs it (the rendered
//! headings and raw-HTML anchor ids), and per file of a reference site
//! its definitions outside code and comments in text order and the
//! labels its reference links write (graph/md.rs's masking and
//! detector). The masking, the heading walk and the slugging stay here:
//! their other readers (the tombstone leg on the hook path, docdup's
//! segments, fourclass's section units, the walk's key) have no core
//! replacement yet. Which target a site names, the fold of a label, the
//! first definition that wins and whether a definition is used are the
//! core's (CE.Resolve.Md).

use crate::graph::ladder::md::slug::slug_set;
use crate::graph::ladder::{Scope, Site};
use crate::graph::md::{content_lines, detect, is_md_path, ref_definition};
use crate::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

/// The `assets` and `md` keys of a request whose batch holds Markdown
/// sites; each file read once per sweep (the memo).
pub fn add(body: &mut Value, scope: &Scope, sites: &[(Lang, &Site)]) {
    let md: Vec<&Site> = sites
        .iter()
        .filter(|(lang, _)| *lang == Lang::Markdown)
        .map(|(_, s)| *s)
        .collect();
    if md.is_empty() {
        return;
    }
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
