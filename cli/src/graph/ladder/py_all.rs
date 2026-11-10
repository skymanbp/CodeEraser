//! A Python package's `__all__` as the walk reads it (lane
//! v235_graphtrust, item T3): CPython's `from pkg import *` imports each
//! name `pkg/__init__.py`'s `__all__` lists — a submodule among them is
//! executed — so the core's star-import rule reads the list beside the
//! package (CE.Resolve.PyAlso; the request's `py.all`). Read by the one
//! `__all__` reader the visibility slice uses (fourclass/visibility/
//! py.rs `module_all_parts`), off the walked file's own root, never resolved.

use crate::scan::ast;
use crate::scan::lang::Lang;

/// Whether a walked path is a package's `__init__.py`.
pub fn is_init(rel: &str) -> bool {
    rel == "__init__.py" || rel.ends_with("/__init__.py")
}

/// One `__init__.py`'s `__all__`: the names its literal bindings and
/// edits list (in name order) and whether any binding or edit is one no
/// literal reads (`opaque`).
pub type All = (Vec<String>, bool);

/// One `__init__.py`'s `__all__` ([`All`]), None when the file never
/// spells the name — and so, before any parse, when its text does not
/// hold the bytes an identifier spelling it needs.
pub fn read(text: &str) -> Option<All> {
    if !text.contains("__all__") {
        return None;
    }
    let tree = ast::parse_lang(text, Lang::Python)?;
    crate::fourclass::visibility::module_all_parts(tree.root_node(), text.as_bytes())
        .map(|(names, opaque)| (names.into_iter().collect(), opaque))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/py_all.rs"]
mod tests;
