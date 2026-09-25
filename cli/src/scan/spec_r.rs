//! The R LangSpec table (plan v2.30 step 4; design booklet §4 the R
//! column). Lives beside spec_lua.rs for the reason every table does:
//! spec.rs is the contract alone (RM16).
//!
//! The external oracle is lizard (CCN): its R reader,
//! `lizard_languages/r.py`, was missed when the booklet pinned the
//! oracles and found by the step-4 crosscheck (register D0 now names
//! CoC alone). No R cognitive oracle exists, so CoC's mapping is the
//! repository's own stance and the batteries its only executor. Every
//! place the table reads differently from lizard is attributed in
//! contracts/fixtures/crosscheck/DIVERGENCES.md (the R section), never
//! a hidden choice.
//!
//! Key probe facts the table stands on (tree-sitter-r 1.3.0, the
//! scripts/tsprobe transcripts):
//! - every function is an anonymous `function_definition` (`function`
//!   and `\` alike); its `name` field holds the KEYWORD token, never a
//!   name, and the name is the assignment around it
//!   (`f <- function(x)`, `(function(x) x) -> f`, scan/binding.rs) — an
//!   unbound one is a unit of its own named `(anonymous)` (register D3)
//! - `else` has no node: `if_statement.alternative` IS the else
//!   expression or the next if (the Go and Java shape), so the if_kinds
//!   rule scores a plain else, and a braced one alike
//! - `&&` / `||` short-circuit; the vectorised `&` / `|` do not and
//!   count nowhere (register D9); `repeat` is a loop; `next` / `break`
//!   carry no label, and R has no goto (register D5)
//! - `switch()`, `ifelse()` and `tryCatch()` are calls: their control
//!   flow is invisible to the syntax and scores nothing (register D8)
//! - comments are one `comment` kind (roxygen `#'` included); a string
//!   lexes as NAMED `string_open` / `string_content` / `string_close`
//!   pieces, a raw string (`r"(…)"`) too; `1L` and `3i` are composite
//!   nodes whose only leaf is the suffix (register D10)
//! - a call is `call{function, arguments}`; a member callee (`x$f()`,
//!   `pkg::f()`) is an extract_operator or a namespace_operator holding
//!   the object and the member; R has no self word

use super::spec::{LangSpec, NameStyle};

pub static R: LangSpec = LangSpec {
    fn_kinds: &["function_definition"],
    fn_required_fields: &[],
    param_list_kinds: &["parameters"],
    cc_kinds: &[
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
    ],
    cc_operators: &["&&", "||"],
    chain_kinds: &[],
    // a for's sequence and a while's condition are headers (register
    // D31); a repeat is all body
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "while_statement body",
        "repeat_statement body",
    ],
    if_kinds: &["if_statement"],
    coc_flat_kinds: &[],
    // a function value is a unit of its own (register D3)
    coc_nest_only_kinds: &[],
    coc_operators: &["&&", "||"],
    coc_jump_kinds: &[],
    label_kinds: &[],
    comment_kinds: &["comment"],
    // base R, the tidyverse and Bioconductor each write their own
    // (register D22)
    name_style: NameStyle::Any,
    // the quote pieces are named kinds here, not anonymous tokens
    literal_delims: &["string_open", "string_close"],
    call_kinds: &["call"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["extract_operator", "namespace_operator"],
    call_self_words: &[],
    call_member_scopes: &[],
    // a member's owner is the object its name spells (`x` of `x$f`,
    // scan/binding.rs)
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &[],
    opaque_fields: &[],
};
