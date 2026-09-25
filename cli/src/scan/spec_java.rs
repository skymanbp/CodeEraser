//! The Java LangSpec table (plan v2.30 step 3; design booklet §4 the
//! Java column). Lives beside spec_launch.rs for the reason every table
//! does: spec.rs is the contract alone (RM16).
//!
//! The external oracles are lizard (CCN) and PMD's CognitiveComplexity
//! rule — Java is the first new language whose cognitive complexity an
//! independent implementation can check — and every place the table
//! reads differently from either is a numbered stance in
//! contracts/fixtures/crosscheck/DIVERGENCES.md (the Java section),
//! never a hidden choice.
//!
//! Key probe facts the table stands on (tree-sitter-java 0.23.5, the
//! scripts/tsprobe transcripts of Probe.java / round2.java and a third
//! sample of enum, interface, annotation and qualified-name shapes):
//! - a method, a constructor and a record's compact constructor carry
//!   their name in `name`; an abstract, interface or native method is
//!   the same kind with no `body` (the fn_required_fields gate)
//! - `else` has no node: `if_statement.alternative` IS the else body or
//!   the next if (the Go shape), and a single-statement else is any
//!   statement there — the if_kinds rule scores all of them
//! - one `switch_expression` spells both the statement and the arrow
//!   switch; each `case` / `default` label is a `switch_label` (default
//!   included, register D2)
//! - the ternary is `ternary_expression` (it nests, register D4); a
//!   lambda_expression absorbs into its host and raises nesting only
//!   (register D3), while an anonymous or local class's methods are
//!   units of their own
//! - `break L` / `continue L` carry the label as a bare `identifier`
//!   child (no field): a plain `break;` has none (register D5)
//! - comments are `line_comment` and `block_comment` (Javadoc is a
//!   block comment); strings lex as `"` + string_fragment + `"`, a text
//!   block as `"""` + multiline_string_fragment + `"""`, and a char
//!   literal as one `character_literal` leaf
//! - a call is `method_invocation{object?, name, arguments}`: the
//!   receiver is the call's own field, not a member node inside the
//!   callee (call_fields). `this.m()` is the caller's own object;
//!   `super.m()` is NOT — it names the superclass's `m`, and an
//!   override calling `super.m()` is Java's commonest delegation, never
//!   a recursion. `K.m()` reaches the caller's own class by its name
//!   (the owner tail, scan/calls.rs)
//! - a method is always a member: of a class, interface, enum, record
//!   (a `class_body`) or annotation body, so no Java callable is ever
//!   bare-reachable across types — the owner road is the only one

use super::spec::{LangSpec, NameStyle, Overloads};

pub static JAVA: LangSpec = LangSpec {
    fn_kinds: &[
        "method_declaration",
        "constructor_declaration",
        "compact_constructor_declaration",
    ],
    fn_required_fields: &[("method_declaration", "body")],
    param_list_kinds: &["formal_parameters"],
    cc_kinds: &[
        "if_statement",
        "for_statement",
        "enhanced_for_statement",
        "while_statement",
        "do_statement",
        // `case` and `default` alike (register D2: lizard counts the
        // `case` keyword only)
        "switch_label",
        "ternary_expression",
        "catch_clause",
    ],
    cc_operators: &["&&", "||"],
    chain_kinds: &[],
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "enhanced_for_statement body",
        "while_statement body",
        "do_statement body",
        "switch_expression body",
        "ternary_expression",
        "catch_clause body",
    ],
    if_kinds: &["if_statement"],
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &["lambda_expression"],
    coc_operators: &["&&", "||"],
    coc_jump_kinds: &["break_statement", "continue_statement"],
    label_kinds: &["identifier"],
    comment_kinds: &["line_comment", "block_comment"],
    // the platform's own naming conventions (register D22)
    name_style: NameStyle::MixedCaps,
    literal_delims: &["\"", "\"\"\""],
    call_kinds: &["method_invocation"],
    call_fields: ("name", Some("object")),
    call_name_kinds: &["identifier"],
    call_member_kinds: &[],
    // never `super`: super.m() names the superclass's m (module doc)
    call_self_words: &["this"],
    // an enum's methods sit in `enum_body_declarations` inside the
    // `enum_body`, which the grandparent read reaches
    call_member_scopes: &[
        "class_body",
        "interface_body",
        "enum_body",
        "annotation_type_body",
    ],
    owner_kinds: &[
        "class_declaration",
        "interface_declaration",
        "enum_declaration",
        "record_declaration",
        "annotation_type_declaration",
    ],
    overloads: Some(&JAVA_OVERLOADS),
    // Java has no import below the file header
    call_import_kinds: &[],
    opaque_fields: &[],
};

/// A varargs parameter removes the upper bound; the receiver parameter
/// (`void m(Foo this)`) takes no argument; a call never spreads (an
/// array handed to a varargs parameter is one argument); a constructor
/// is reached by `new` and `this(…)`, neither of them a call kind.
static JAVA_OVERLOADS: Overloads = Overloads {
    optional: &[],
    variadic: &["spread_parameter"],
    ignored: &["receiver_parameter"],
    spread: &[],
    unreachable: &["constructor_declaration", "compact_constructor_declaration"],
};
