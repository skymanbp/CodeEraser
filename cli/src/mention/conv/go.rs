//! Go: `Ffi` from the directive comments cgo and the wasm target read
//! — `//export Name` must name this very function (cgo's own lexing:
//! the prefix `//export `, then the name with surrounding blanks
//! trimmed), `//go:wasmexport <symbol>` exports the function it
//! precedes under any symbol. Comments are siblings in tree-sitter-go,
//! so the directive is found in the comment run above the
//! declaration; the first non-comment sibling ends the run. Blank
//! lines do not end it, so the run is wider than cgo's doc group —
//! an over-exemption only, never a missed export.

use super::{Conv, text};
use tree_sitter::Node;

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let Some(name) = node.child_by_field_name("name") else {
        return 0;
    };
    let name = text(name, src);
    let mut prev = node.prev_sibling();
    while let Some(c) = prev.filter(|c| c.kind() == "comment") {
        let line = text(c, src);
        let exported = line
            .strip_prefix("//export ")
            .is_some_and(|rest| rest.trim() == name);
        if exported || line.starts_with("//go:wasmexport ") {
            return Conv::Ffi.bit();
        }
        prev = c.prev_sibling();
    }
    0
}
