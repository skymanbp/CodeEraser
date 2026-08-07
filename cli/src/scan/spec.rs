//! Per-language node-kind tables driving function extraction and both
//! complexity metrics. One data table per language instead of one code
//! path per language (the anti-fuck-u-code design: their regex control
//! flow signatures broke on every non-C-like syntax).
//!
//! Alignment targets (plan §6 M1 acceptance): lizard (TS/Py),
//! rust-code-analysis (Rust), gocyclo/gocognit (Go). Known divergences
//! are recorded in contracts/ during the M1 cross-check, not hidden.

use super::lang::Lang;

pub struct LangSpec {
    /// Node kinds counted as standalone function units. Anything not
    /// listed here (Go func_literal, Python lambda) is absorbed into
    /// its host function by construction — no separate flag needed.
    pub fn_kinds: &'static [&'static str],
    /// Kind of the parameter-list child (direct child of a fn node).
    pub param_list_kinds: &'static [&'static str],
    /// +1 cyclomatic per node of these kinds.
    pub cc_kinds: &'static [&'static str],
    /// Binary-operator tokens adding +1 cyclomatic (short-circuit ops).
    pub cc_operators: &'static [&'static str],
    /// Cognitive: structures that increment AND raise nesting.
    pub coc_nesting_kinds: &'static [&'static str],
    /// Cognitive: flat +1 (no nesting penalty), e.g. `else`.
    pub coc_flat_kinds: &'static [&'static str],
    /// Cognitive: raise nesting only (lambdas / inline fns).
    pub coc_nest_only_kinds: &'static [&'static str],
    pub comment_kinds: &'static [&'static str],
}

pub fn spec(lang: Lang) -> &'static LangSpec {
    match lang {
        Lang::Python => &PYTHON,
        Lang::TypeScript | Lang::Tsx => &TYPESCRIPT,
        Lang::Rust => &RUST,
        Lang::Go => &GO,
        Lang::Markdown => &MARKDOWN,
    }
}

static PYTHON: LangSpec = LangSpec {
    fn_kinds: &["function_definition"],
    param_list_kinds: &["parameters", "lambda_parameters"],
    cc_kinds: &[
        "if_statement",
        "elif_clause",
        "conditional_expression",
        "for_statement",
        "while_statement",
        "except_clause",
        "case_clause",
        "assert_statement",
    ],
    cc_operators: &["and", "or"],
    coc_nesting_kinds: &[
        "if_statement",
        "for_statement",
        "while_statement",
        "except_clause",
        "conditional_expression",
        "match_statement",
    ],
    coc_flat_kinds: &["elif_clause", "else_clause"],
    coc_nest_only_kinds: &["lambda"],
    comment_kinds: &["comment"],
};

static TYPESCRIPT: LangSpec = LangSpec {
    fn_kinds: &[
        "function_declaration",
        "generator_function_declaration",
        "method_definition",
        "arrow_function",
        "function_expression",
    ],
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
    coc_nesting_kinds: &[
        "if_statement",
        "for_statement",
        "for_in_statement",
        "while_statement",
        "do_statement",
        "switch_statement",
        "catch_clause",
        "ternary_expression",
    ],
    coc_flat_kinds: &["else_clause"],
    coc_nest_only_kinds: &["arrow_function", "function_expression"],
    comment_kinds: &["comment"],
};

static RUST: LangSpec = LangSpec {
    fn_kinds: &["function_item", "closure_expression"],
    param_list_kinds: &["parameters", "closure_parameters"],
    cc_kinds: &[
        "if_expression",
        "match_arm",
        "for_expression",
        "while_expression",
        "loop_expression",
    ],
    cc_operators: &["&&", "||"],
    coc_nesting_kinds: &[
        "if_expression",
        "for_expression",
        "while_expression",
        "loop_expression",
        "match_expression",
    ],
    coc_flat_kinds: &["else_clause"],
    coc_nest_only_kinds: &["closure_expression"],
    comment_kinds: &["line_comment", "block_comment"],
};

static GO: LangSpec = LangSpec {
    // gocyclo attributes func-literal branches to the enclosing decl,
    // so func_literal is absorbed, not standalone.
    fn_kinds: &["function_declaration", "method_declaration"],
    param_list_kinds: &["parameter_list"],
    cc_kinds: &[
        "if_statement",
        "for_statement",
        "expression_case",
        "type_case",
        "default_case",
        "communication_case",
    ],
    cc_operators: &["&&", "||"],
    coc_nesting_kinds: &[
        "if_statement",
        "for_statement",
        "expression_switch_statement",
        "type_switch_statement",
        "select_statement",
    ],
    // Go has no else node kind: else lives in the if's `alternative`
    // field and is scored by the field-aware logic in cognitive.rs.
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &["func_literal"],
    comment_kinds: &["comment"],
};

static MARKDOWN: LangSpec = LangSpec {
    fn_kinds: &[],
    param_list_kinds: &[],
    cc_kinds: &[],
    cc_operators: &[],
    coc_nesting_kinds: &[],
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &[],
    comment_kinds: &[],
};
