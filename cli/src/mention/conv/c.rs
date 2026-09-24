//! C and C++: one fact, `Ffi` — the declaration is handed to a foreign
//! caller by its own syntax (plan v2.30 step 2; design booklet §9):
//!   - a `linkage_specification` ancestor: `extern "C" void f() {}`
//!     wraps the definition as its `body`, the block form wraps a
//!     `declaration_list` around several — both probed, both the same
//!     ancestor test;
//!   - an attribute the linker or the runtime reads: `__declspec(dllexport)`
//!     (an `ms_declspec_modifier` child) and, in an `attribute_specifier`
//!     child, `visibility("default")`, `used`, `constructor` or
//!     `destructor` — the Rust arm's `used` / `ctor` rows are the same
//!     categories. The grammar nests a call_expression inside the
//!     specifier, so the specifier is read as words: `visibility` with
//!     `hidden` is the opposite claim and earns nothing, and a C++11
//!     `[[nodiscard]]` (an `attribute_declaration`) names no export.
//!
//! A plain `extern` is a storage class, not a linkage specification,
//! and claims nothing: it says the definition lives elsewhere, never
//! that a foreign caller reaches it.

use super::{Conv, text};
use crate::fourclass::visibility::ancestors;
use crate::scan::ast;
use tree_sitter::Node;

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let linked = ancestors(node).any(|a| a.kind() == "linkage_specification");
    let attributed = ast::children(node)
        .into_iter()
        .filter(|c| matches!(c.kind(), "attribute_specifier" | "ms_declspec_modifier"))
        .any(|c| exports(text(c, src)));
    if linked || attributed {
        Conv::Ffi.bit()
    } else {
        0
    }
}

/// The attribute's words (identifier characters only; punctuation and
/// the quotes around `"default"` are separators).
fn exports(attr: &str) -> bool {
    let words: Vec<&str> = attr
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .collect();
    words
        .iter()
        .any(|w| matches!(*w, "dllexport" | "used" | "constructor" | "destructor"))
        || (words.contains(&"visibility") && words.contains(&"default"))
}
