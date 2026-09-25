//! The C / C++ LangSpec tables (plan v2.30 step 2). ONE set of kinds
//! for both grammars: tree-sitter-cpp is a superset of tree-sitter-c
//! and spells every construct the two share with the same node kind
//! (probe transcripts 2026-09-24, four rounds against the pinned
//! 0.24.2 / 0.23.4 — scripts/tsprobe), so the C++-only kinds below
//! simply never occur in a C parse. The two tables differ in one
//! LANGUAGE fact no kind can carry (plan v2.30 step 3): C++ overloads a
//! name by its parameters and C does not — two same-named C functions
//! of one scope are one function under two `#if` arms, two C++ ones
//! may be two functions (LangSpec::overloads). Lives beside
//! spec_hs.rs for the reason that file does: spec.rs is the contract
//! alone (RM16).
//!
//! The external oracle is lizard (CCN only — no C/C++ cognitive
//! oracle exists), and every place the table reads differently from it
//! is a numbered stance in contracts/fixtures/crosscheck/DIVERGENCES.md
//! (the C / C++ section), never a hidden choice.
//!
//! Key probe facts the table stands on:
//! - a function's NAME sits at the leaf of a declarator chain
//!   (`function_definition.declarator` → function_declarator →
//!   pointer_declarator / parenthesized_declarator / … → identifier |
//!   field_identifier | qualified_identifier | destructor_name |
//!   operator_name | template_function), and its parameter list hangs
//!   off the innermost function_declarator — functions.rs walks that
//!   chain, which is why param_list_kinds stays empty here
//! - `else` is an `else_clause` holding either a compound_statement or
//!   the next if_statement (the TypeScript shape, scored by the same
//!   flat-hybrid rule); `default:` is a case_statement like any case
//! - `switch_statement` nests; its case_statement rows do not
//! - C++ `and` / `or` arrive as the `operator` field's text exactly
//!   like `&&` / `||`; `not` is unary and counts nowhere
//! - `goto` carries `label: statement_identifier`; C's break /
//!   continue never carry a label, so they are not listed (an entry
//!   that can never fire is a dead entry — M1 attack review)
//! - a lambda_expression absorbs into its host (no fn_kinds entry)
//!   and raises nesting only — the Go func_literal precedent
//! - string_literal lexes as `"` + string_content + `"`, char_literal
//!   as `'` + character + `'`, a raw string as `R"` + `(` +
//!   raw_string_content + `)` + `"`: the anonymous quote tokens are
//!   the delimiter pieces under literal_delims; the raw string's
//!   parentheses stay text tokens, so a raw literal is five tokens
//!   where a plain one is one (a dedup undercount, register D23)
//! - the callee is the `function` field of call_expression; a member
//!   callee is a field_expression (`this->m`, `obj.m`) or a
//!   qualified_identifier (`K::m`); a class body is a
//!   field_declaration_list; `using ns::name;` binds a name locally
//! - the preprocessor is invisible to both metrics: `#ifdef` /
//!   `#elif` are neither branches nor nesting (register D1 — lizard
//!   counts them), and a macro body is one opaque `preproc_arg` token

use super::spec::{LangSpec, NameStyle, Overloads};

/// C: same-named definitions of one scope are one function written
/// under two preprocessor arms — the language forbids a second one.
pub static C: LangSpec = FAMILY;

/// C++: a name is overloaded by its parameters. A default argument
/// counts toward the upper bound only; a parameter pack and the
/// C-style `...` (an anonymous token in a parameter_list, probed)
/// remove it; a pack expansion in a call (`f(a...)`, probed as
/// parameter_pack_expansion) passes a count no reader can tell. A
/// constructor stays reachable — `K(x)` constructs a K.
pub static CPP: LangSpec = LangSpec {
    overloads: Some(&CPP_OVERLOADS),
    ..FAMILY
};

static CPP_OVERLOADS: Overloads = Overloads {
    optional: &["optional_parameter_declaration"],
    variadic: &["variadic_parameter_declaration", "..."],
    ignored: &[],
    spread: &["parameter_pack_expansion"],
    unreachable: &[],
};

const FAMILY: LangSpec = LangSpec {
    fn_kinds: &["function_definition"],
    // every definition shape rides declarator::defined instead
    fn_required_fields: &[],
    // the list hangs off the declarator chain — functions.rs reads it
    // through the chain, nothing here to scan for
    param_list_kinds: &[],
    cc_kinds: &[
        "if_statement",
        "for_statement",
        "for_range_loop",
        "while_statement",
        "do_statement",
        // every case incl. `default:` (register D2: lizard counts the
        // `case` keyword only; the Rust match_arm / Haskell alternative
        // precedent — a total switch's default is a real path)
        "case_statement",
        "conditional_expression",
        "catch_clause",
    ],
    // `and` / `or` are the C++ alternative tokens (register D19)
    cc_operators: &["&&", "||", "and", "or"],
    chain_kinds: &[],
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "for_range_loop body",
        "while_statement body",
        "do_statement body",
        "switch_statement body",
        // the ternary nests like TypeScript's (register D4)
        "conditional_expression",
        "catch_clause body",
    ],
    // the if's `alternative` is an else_clause, scored by the flat rule
    if_kinds: &["if_statement"],
    coc_flat_kinds: &["else_clause"],
    coc_nest_only_kinds: &["lambda_expression"],
    coc_operators: &["&&", "||", "and", "or"],
    coc_jump_kinds: &["goto_statement"],
    label_kinds: &["statement_identifier"],
    comment_kinds: &["comment"],
    // no single convention holds across C code bases (register D22)
    name_style: NameStyle::Any,
    literal_delims: &["\"", "'", "character", "R\""],
    call_kinds: &["call_expression"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["field_expression", "qualified_identifier"],
    call_self_words: &["this"],
    // a class / struct / union body says so itself (the TypeScript
    // class_body precedent); a namespace body is a declaration_list
    // and is NOT a member scope — bare names resolve inside it
    call_member_scopes: &["field_declaration_list"],
    // the declarator spells a member's owner (scan/declarator.rs)
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &["using_declaration"],
    // `#if defined(A) && B` / `#elif`: the condition is an expression
    // to the parser and compile-time text to every metric (register
    // D1 — fmt's is_big_endian read 2 for the `&&` in its `#elif`
    // before this line existed); `#ifdef` names an identifier, not an
    // expression, and needs no row
    opaque_fields: &[("preproc_if", "condition"), ("preproc_elif", "condition")],
};
