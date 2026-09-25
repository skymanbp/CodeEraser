//! The M1 launch tables — Python, TypeScript / TSX, Rust and Go —
//! moved out of spec.rs (the contract) when plan v2.30 step 3 gave the
//! struct the mechanisms Java needs; the tables are unchanged but for
//! the new fields, each spelled as its language already behaved (booklet
//! §4: every extension is backward compatible, and the existing
//! batteries prove it value by value).

use super::spec::{LangSpec, NameStyle};

pub static PYTHON: LangSpec = LangSpec {
    fn_kinds: &["function_definition"],
    fn_required_fields: &[],
    param_list_kinds: &["parameters"],
    cc_kinds: &[
        "if_statement",
        "elif_clause",
        "conditional_expression",
        "for_statement",
        "while_statement",
        "except_clause",
        "case_clause",
        "assert_statement",
        // comprehension clauses are real branch paths: lizard and radon
        // both count them (M1 cross-check finding). CoC deliberately does
        // NOT count them (declarative expression, no nesting cost).
        "for_in_clause",
        "if_clause",
    ],
    cc_operators: &["and", "or"],
    chain_kinds: &[],
    // the except's block is an unnamed child, named by its kind
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body alternative",
        "while_statement body alternative",
        "except_clause block",
        "conditional_expression",
        "match_statement body",
    ],
    if_kinds: &["if_statement"],
    // an elif's condition sits at the chain's level, like an else-if
    coc_flat_kinds: &["elif_clause consequence", "else_clause"],
    coc_nest_only_kinds: &["lambda"],
    coc_operators: &["and", "or"],
    // Python has no labeled jumps.
    coc_jump_kinds: &[],
    label_kinds: &[],
    comment_kinds: &["comment"],
    name_style: NameStyle::Snake,
    // Python quotes surface as named string_start/string_end kinds.
    literal_delims: &[],
    call_kinds: &["call"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["attribute"],
    call_self_words: &["self", "cls"],
    call_member_scopes: &["class_definition"],
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &["import_from_statement", "import_statement"],
    opaque_fields: &[],
};

pub static TYPESCRIPT: LangSpec = LangSpec {
    fn_kinds: &[
        "function_declaration",
        "generator_function_declaration",
        "method_definition",
        "arrow_function",
        "function_expression",
    ],
    fn_required_fields: &[],
    param_list_kinds: &["formal_parameters"],
    cc_kinds: &[
        "if_statement",
        "ternary_expression",
        "for_statement",
        "for_in_statement",
        "while_statement",
        "do_statement",
        "switch_case",
        "catch_clause",
    ],
    cc_operators: &["&&", "||", "??"],
    chain_kinds: &[],
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "for_in_statement body",
        "while_statement body",
        "do_statement body",
        "switch_statement body",
        "catch_clause body",
        "ternary_expression",
    ],
    if_kinds: &["if_statement"],
    coc_flat_kinds: &["else_clause"],
    // Empty on purpose: arrow/function_expression are STANDALONE units
    // (fn_kinds) here, so a nest-only entry could never fire (the M1
    // attack review caught the dead entries). Only absorbed inline fns
    // (Go func_literal, Python lambda) belong in nest-only.
    coc_nest_only_kinds: &[],
    // `??` counts in CC (a real branch) but NOT here: whitepaper p.6
    // ignores null-coalescing in CoC. `?.` counts in neither (M1 stance,
    // pinned in tests/sonar_whitepaper.rs).
    coc_operators: &["&&", "||"],
    coc_jump_kinds: &["continue_statement", "break_statement"],
    label_kinds: &["statement_identifier"],
    comment_kinds: &["comment"],
    name_style: NameStyle::MixedCaps,
    literal_delims: &["\"", "'", "`"],
    call_kinds: &["call_expression"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["member_expression"],
    call_self_words: &["this"],
    call_member_scopes: &["class_body", "object"],
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &[],
    opaque_fields: &[],
};

pub static RUST: LangSpec = LangSpec {
    fn_kinds: &["function_item", "closure_expression"],
    fn_required_fields: &[],
    param_list_kinds: &["parameters", "closure_parameters"],
    cc_kinds: &[
        "if_expression",
        "match_arm",
        "for_expression",
        "while_expression",
        "loop_expression",
        // `expr?` is an implicit early-return branch (== match Ok/Err);
        // rust-code-analysis counts it (M1 cross-check, ban.rs 21 vs 17).
        "try_expression",
    ],
    cc_operators: &["&&", "||"],
    // let_chain joins let_conditions/exprs with anonymous `&&` tokens
    // (no operator field — AST-probed; M1 attack review finding).
    chain_kinds: &["let_chain"],
    coc_nesting_kinds: &[
        "if_expression consequence alternative",
        "for_expression body",
        "while_expression body",
        "loop_expression body",
        "match_expression body",
    ],
    if_kinds: &["if_expression"],
    coc_flat_kinds: &["else_clause"],
    // Empty on purpose: closure_expression is a standalone unit
    // (fn_kinds), so nest-only could never fire (dead-entry review).
    coc_nest_only_kinds: &[],
    coc_operators: &["&&", "||"],
    // `break 'l` / `continue 'l`: the label child kind is `label`;
    // a plain `break value` has an expression child, so it won't count.
    coc_jump_kinds: &["continue_expression", "break_expression"],
    label_kinds: &["label"],
    comment_kinds: &["line_comment", "block_comment"],
    name_style: NameStyle::Snake,
    // NOT `'`: that is the lifetime/label tick (char_literal is one
    // token and needs no delimiter piece).
    literal_delims: &["\""],
    call_kinds: &["call_expression"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["field_expression", "scoped_identifier"],
    call_self_words: &["self", "Self"],
    call_member_scopes: &["impl_item", "trait_item"],
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &["use_declaration"],
    opaque_fields: &[],
};

pub static GO: LangSpec = LangSpec {
    // gocyclo attributes func-literal branches to the enclosing decl,
    // so func_literal is absorbed, not standalone.
    fn_kinds: &["function_declaration", "method_declaration"],
    fn_required_fields: &[],
    param_list_kinds: &["parameter_list"],
    cc_kinds: &[
        "if_statement",
        "for_statement",
        // default_case is NOT counted: gocyclo v0.6.0 complexity.go
        // skips CaseClause/CommClause with a nil list ("ignore default
        // case"), and the whitepaper p.5 margin (getWords CC=4) agrees.
        // Found by the M1 attack review — no default: in the fixtures,
        // so 52/52 was silent on this axis.
        "expression_case",
        "type_case",
        "communication_case",
    ],
    cc_operators: &["&&", "||"],
    chain_kinds: &[],
    // a switch's cases are unnamed children, named by their kinds;
    // select has no header and nests whole
    coc_nesting_kinds: &[
        "if_statement consequence alternative",
        "for_statement body",
        "expression_switch_statement expression_case default_case",
        "type_switch_statement type_case default_case",
        "select_statement",
    ],
    if_kinds: &["if_statement"],
    // Go has no else node kind: else lives in the if's `alternative`
    // field and is scored by the field-aware logic in cognitive.rs.
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &["func_literal"],
    coc_operators: &["&&", "||"],
    // goto always carries a label_name, so it always counts.
    coc_jump_kinds: &["continue_statement", "break_statement", "goto_statement"],
    label_kinds: &["label_name"],
    comment_kinds: &["comment"],
    name_style: NameStyle::MixedCaps,
    // `'` is Go's rune delimiter but rune_literal is a single token.
    literal_delims: &["\"", "`"],
    call_kinds: &["call_expression"],
    call_fields: ("function", None),
    call_name_kinds: &["identifier"],
    call_member_kinds: &["selector_expression"],
    call_self_words: &[],
    // a Go method is declared at the top level and its name carries
    // the receiver type, so a bare name cannot reach one at all
    call_member_scopes: &[],
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &[],
    opaque_fields: &[],
};
