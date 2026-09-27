//! rs_use.rs's child (split on the file-length line as a `#[path]`
//! child, the rs.rs → rs_use.rs shape; plan v2.30 step 5b): the whole
//! `use` item behind a spec a hand fold cut, and the R5 binder hookup
//! (§4 R5 as amended 2026-08-18, widened in step 5b). The parent's
//! bind-free walk stays reachable, and the surface facts stay
//! rs_reexport.rs's.

use super::super::ctx_for;
use super::use_walk;
use crate::graph::ladder::rs_reexport::{self, Bind};
use crate::graph::ladder::{Outcome, Scope, Site};

/// The whole argument of the `use` item that starts on `row` and whose
/// first line, trimmed, is `spec` — the text a hand fold spread over
/// lines, its whitespace folded to single spaces so `use_path` reads it
/// as one line. Items nest (an inline `mod`, a function body), so the
/// tree is walked down through the nodes covering the row; None when
/// no such item opens there.
pub(super) fn use_text_at(
    tree: &tree_sitter::Tree,
    src: &str,
    row: usize,
    spec: &str,
) -> Option<String> {
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        if node.start_position().row > row || node.end_position().row < row {
            continue;
        }
        if node.kind() == "use_declaration"
            && node.start_position().row == row
            && let Some(arg) = node.child_by_field_name("argument")
            && let Ok(text) = arg.utf8_text(src.as_bytes())
            && text
                .lines()
                .next()
                .is_some_and(|first| first.trim() == spec)
        {
            return Some(text.split_whitespace().collect::<Vec<_>>().join(" "));
        }
        pending.extend(crate::scan::ast::children(node));
    }
    None
}

/// A single-terminal walk that left segments unconsumed consults the
/// terminal's re-export surface for ONE hop and answers the DEFINITION
/// file (the frozen GT's stance); an unbound, ambiguous or
/// self-pointing hop keeps the file-level edge — refinement only, never
/// a downgrade, never a guess. The hop walks the bound path AND the
/// site's segments after the bound name (step 5b): under `pub extern
/// crate inner;` a `facade::inner::sub::Deep` descends the crate's
/// tree, and a re-exported module's children are files.
pub(super) fn bound(scope: &Scope, walked: &[&str], out: Outcome, used: usize) -> Outcome {
    let Outcome::Resolved { path, rung } = &out else {
        return out;
    };
    if used >= walked.len() {
        return out;
    }
    let (name, tail) = (walked[used], &walked[used + 1..]);
    let target = match rs_reexport::binds_to(scope, path, name) {
        Some(Bind::Named(segs, row)) => hop(scope, path, &segs, tail, row),
        Some(Bind::Globbed(globs)) => globbed(scope, path, name, tail, globs),
        None => None,
    };
    match target {
        Some(found) if found != *path => Outcome::ResolvedVia {
            path: found,
            rung: *rung,
        },
        _ => out,
    }
}

/// One bind-free hop: the bound path, then `tail`, walked from the
/// facade at the entry's own row — flatten is top-level-only, and a
/// fixed line 1 read whatever bodied `mod` opened the file as the hop's
/// namespace (the step-8 review's shadow block). A global `pub use
/// ::foo::Bar` (and every `pub extern crate`) carries an empty first
/// segment. None = the hop resolved nothing.
fn hop(scope: &Scope, facade: &str, segs: &[String], tail: &[&str], row: usize) -> Option<String> {
    let global = segs.first().is_some_and(String::is_empty);
    let refs: Vec<&str> = segs
        .iter()
        .skip(usize::from(global))
        .map(String::as_str)
        .chain(tail.iter().copied())
        .collect();
    let hop_site = Site {
        kind: "use",
        from: facade,
        spec: "",
        line: row + 1,
    };
    let ctx = ctx_for(scope, facade);
    match use_walk(&hop_site, &refs, global, ctx.0.as_ref(), &ctx.1, scope).0 {
        Outcome::Resolved { path, .. } => Some(path),
        _ => None,
    }
}

/// The glob half of the surface (step 5b): a `pub use p::*` carries
/// `name` when the file `p` walks to exports it (rs_reexport::exports —
/// a pub item of its own, or a pub use binding the name; a glob inside
/// it is not followed). Exactly one glob carrying the name answers the
/// walk of `p::name` and the tail; several keep the file-level edge
/// (picking would invent a path), as does none — a private item behind
/// a glob is no export.
fn globbed(
    scope: &Scope,
    facade: &str,
    name: &str,
    tail: &[&str],
    globs: Vec<(Vec<String>, usize)>,
) -> Option<String> {
    let mut carrying = globs.into_iter().filter(|(segs, row)| {
        hop(scope, facade, segs, &[], *row)
            .is_some_and(|module| module != facade && rs_reexport::exports(scope, &module, name))
    });
    let (segs, row) = carrying.next()?;
    if carrying.next().is_some() {
        return None;
    }
    let full: Vec<&str> = std::iter::once(name).chain(tail.iter().copied()).collect();
    hop(scope, facade, &segs, &full, row)
}
