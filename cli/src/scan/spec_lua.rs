//! The Lua LangSpec table (plan v2.30 step 4; design booklet §4 the Lua
//! column). Lives beside spec_java.rs for the reason every table does:
//! spec.rs is the contract alone (RM16).
//!
//! The external oracle is lizard (CCN; its reader list includes Lua) —
//! no Lua cognitive oracle exists — and every place the table reads
//! differently from it is a numbered stance in
//! contracts/fixtures/crosscheck/DIVERGENCES.md (the Lua section),
//! never a hidden choice.
//!
//! Key probe facts the table stands on (tree-sitter-lua 0.5.0, the
//! scripts/tsprobe transcripts):
//! - `function f()`, `local function f()`, `function M.f()` and
//!   `function M:f()` are one kind, `function_declaration`, whose
//!   `name` is an identifier, a dot_index_expression or a
//!   method_index_expression; `local` is an anonymous child token
//! - every other function is an anonymous `function_definition` — a
//!   table field's value, an assignment's value (`local f = function`,
//!   `M.f = function`), an argument. Its name, when it has one, is the
//!   binding around it (scan/binding.rs), and an unbound one is a unit
//!   of its own named `(anonymous)` (register D3: Lua's function values
//!   are its declaration form, so they never fold into their host)
//! - `if_statement` hangs its `elseif_statement`s and its
//!   `else_statement` on one repeated `alternative` field; an elseif
//!   carries its own condition, which sits at the chain's level (the
//!   Python elif reading)
//! - `goto` carries its label as an unfielded identifier child; break
//!   never carries one (register D5)
//! - comments are one compound `comment` kind (start / content / end
//!   children), skipped whole; a string lexes as a start token, a
//!   string_content and an end token, and the long-bracket tokens are
//!   `[[` / `]]` at every level (`[==[` included)
//! - a call is `function_call{name, arguments}`: the callee is its
//!   `name` field, and a member callee (`M.f()`, `obj:m()`) is a
//!   dot_index_expression or method_index_expression holding the object
//!   and the member; `self` is the method's own table

use super::spec::{LangSpec, NameStyle};

pub static LUA: LangSpec = LangSpec {
    fn_kinds: &["function_declaration", "function_definition"],
    fn_required_fields: &[],
    param_list_kinds: &["parameters"],
    // an elseif is a decision of its own (lizard counts the keyword)
    cc_kinds: &[
        "if_statement",
        "elseif_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
    ],
    cc_operators: &["and", "or"],
    chain_kinds: &[],
    // a repeat's `until` condition is its header (register D31)
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "while_statement body",
        "repeat_statement body",
    ],
    // the if's alternatives are elseif / else nodes, scored flat
    if_kinds: &["if_statement"],
    coc_flat_kinds: &["elseif_statement consequence", "else_statement"],
    // a function value is a unit of its own (register D3)
    coc_nest_only_kinds: &[],
    coc_operators: &["and", "or"],
    coc_jump_kinds: &["goto_statement"],
    label_kinds: &["identifier"],
    comment_kinds: &["comment"],
    // no convention holds across Lua code bases (register D22)
    name_style: NameStyle::Any,
    literal_delims: &["\"", "'", "[[", "]]"],
    call_kinds: &["function_call"],
    call_fields: ("name", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["dot_index_expression", "method_index_expression"],
    call_self_words: &["self"],
    // a table constructor's fields answer to the table, never to a
    // bare name (`{ f = function() end }` is reached as `t.f`)
    call_member_scopes: &["table_constructor"],
    // a member's owner is the table its name spells (`M` of `M.f`,
    // scan/binding.rs), not a type declaration around it
    owner_kinds: &[],
    overloads: None,
    // a local binding shadowing a callable is not modelled (register
    // D21): the recursion increment may overcount on that shape
    call_import_kinds: &[],
    opaque_fields: &[],
};
