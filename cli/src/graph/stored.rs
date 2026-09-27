//! What of a reference spec the index keeps (plan v2.30 step 5b item
//! 24). The site table used to store every spec verbatim, so a `url`
//! or an `href` written `https://<user>:<secret>@example.com/x?token=<t>`
//! landed in `.ce/index.db` with its userinfo and its query — the two
//! RFC 3986 components (§3.2.1, §3.4) that carry credentials in
//! practice — while every rung discards both before it joins: html.rs
//! cuts the path at `?`, html_head::split_origin drops the userinfo,
//! and md::is_scheme sends a URL out of the corpus unread. The stored
//! form keeps what a rung reads and nothing a rung ignores: the path,
//! a bare `?` where a query stood (its presence is a fact the HTML
//! rungs branch on — a pure query is the page — and its content is
//! nobody's) and the fragment whole, since a cross-page fragment is
//! checked against the target's ids. Only the reference-family kinds
//! are touched: a code import's spec is a module path, never a URL,
//! and stays as spelled. The detector's output is untouched (RG3: the
//! frozen universe stands on it); the trimming happens once, at the
//! store's write, so no rung's answer moves — and GRAPH_REV 19 has
//! every index written before it re-derive its site rows once.

/// The site kinds whose spec is a URL or a URL reference (RFC 3986):
/// the Markdown targets, the HTML attributes, the bare `url` autolink.
const REFERENCE_KINDS: &str = "link image ref_def url href src srcset action link_asset";

/// The spec as the index stores it (module doc).
pub fn spec(kind: &str, raw: &str) -> String {
    if !REFERENCE_KINDS.split_ascii_whitespace().any(|k| k == kind) {
        return raw.to_string();
    }
    let (head, fragment) = match raw.split_once('#') {
        Some((h, f)) => (h, Some(f)),
        None => (raw, None),
    };
    let (path, query) = match head.split_once('?') {
        Some((p, _)) => (p, "?"),
        None => (head, ""),
    };
    let mut out = without_userinfo(path);
    out.push_str(query);
    if let Some(f) = fragment {
        out.push('#');
        out.push_str(f);
    }
    out
}

/// The path with its authority's userinfo removed (example shapes):
/// `scheme://<userinfo>@host/x` and `//<userinfo>@host/x` become
/// `scheme://host/x` and `//host/x`. A spec with no `//` authority — a
/// relative reference, a `mailto:` — is as spelled: the address after
/// `mailto:` is the reference itself, not a credential in front of a
/// host.
fn without_userinfo(path: &str) -> String {
    let Some(at) = path.find("//") else {
        return path.to_string();
    };
    let scheme = &path[..at];
    let scheme_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.');
    let rooted = at == 0
        || (scheme.ends_with(':')
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme[..at - 1].chars().all(scheme_char));
    if !rooted {
        return path.to_string();
    }
    let start = at + 2;
    let end = path[start..].find('/').map_or(path.len(), |i| start + i);
    match path[start..end].rfind('@') {
        Some(i) => format!(
            "{}{}{}",
            &path[..start],
            &path[start + i + 1..end],
            &path[end..]
        ),
        None => path.to_string(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/stored.rs"]
mod tests;
