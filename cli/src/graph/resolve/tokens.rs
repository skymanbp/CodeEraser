//! A site's specifier as resolve/1 reads it (plan v2.33 wave W2a): the
//! text split by its language's separator rule — lexing, so it stays
//! here; what the pieces mean is the core's. Each row is `[lang, kind,
//! from, form, token…]`:
//!   Python — form = the leading dots, tokens = the dotted segments
//!     joined by -1, each segment the pieces its text holds between
//!     slashes (none for a bare `from . import x`);
//!   Lua `require` — the pieces between dots and slashes, -1 for a dot
//!     and -2 for a slash between each two; any other Lua kind — the
//!     slash pieces, form 1 when the path names no file of the tree
//!     (rooted at `/` or `~`, or holding a drive or a colon);
//!   Go — the slash pieces, form 1 when the first holds a dot;
//!   C / C++ — the name's slash pieces, form 1 for the `<…>` form.

use super::intern::Intern;
use crate::graph::ladder::Site;
use crate::graph::store::kind_code;
use crate::scan::lang::Lang;

/// One site's row.
pub fn row(lang: Lang, site: &Site, from: i64, seg: &mut Intern) -> Result<Vec<i64>, String> {
    let kind = kind_code(site.kind).map_err(|e| e.to_string())?;
    let spec = site.spec;
    let (form, tokens) = match lang {
        Lang::Python => python(spec, seg),
        Lang::Lua if site.kind == "require" => (0, lua_name(spec, seg)),
        Lang::Lua => (i64::from(rooted(spec)), seg.pieces(spec, '/')),
        Lang::Go => {
            let dotted = spec.split('/').next().is_some_and(|h| h.contains('.'));
            (i64::from(dotted), seg.pieces(spec, '/'))
        }
        _ => {
            let (name, system) = c_form(spec);
            (i64::from(system), seg.pieces(name, '/'))
        }
    };
    let mut row = vec![lang as i64, kind, from, form];
    row.extend(tokens);
    Ok(row)
}

/// The name and form of an include spec: `<x>` is the system form.
pub fn c_form(spec: &str) -> (&str, bool) {
    match spec.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        Some(inner) => (inner, true),
        None => (spec, false),
    }
}

fn python(spec: &str, seg: &mut Intern) -> (i64, Vec<i64>) {
    let dots = spec.chars().take_while(|c| *c == '.').count();
    let rest = &spec[dots..];
    if rest.is_empty() {
        return (dots as i64, Vec::new());
    }
    let mut out = Vec::new();
    for (i, part) in rest.split('.').enumerate() {
        if i > 0 {
            out.push(-1);
        }
        out.extend(seg.pieces(part, '/'));
    }
    (dots as i64, out)
}

fn lua_name(spec: &str, seg: &mut Intern) -> Vec<i64> {
    let mut out = Vec::new();
    let mut piece = String::new();
    for c in spec.chars() {
        match c {
            '.' | '/' => {
                out.push(seg.id(&piece));
                out.push(if c == '.' { -1 } else { -2 });
                piece.clear();
            }
            _ => piece.push(c),
        }
    }
    out.push(seg.id(&piece));
    out
}

/// A `dofile` path that names no file of the tree.
fn rooted(spec: &str) -> bool {
    spec.starts_with(['/', '~']) || spec.contains(':')
}
