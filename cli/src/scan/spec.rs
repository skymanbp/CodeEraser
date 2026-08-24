//! Per-language node-kind tables driving function extraction and both
//! complexity metrics. One data table per language instead of one code
//! path per language (the anti-fuck-u-code design: their regex control
//! flow signatures broke on every non-C-like syntax).
//!
//! Alignment targets (plan §6 M1 acceptance): lizard (TS/Py),
//! rust-code-analysis (Rust), gocyclo/gocognit (Go). Known divergences
//! are recorded in contracts/ during the M1 cross-check, not hidden.

use super::lang::Lang;

/// Function-name convention for the readability naming check (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameStyle {
    /// snake_case — PEP 8 (Python), RFC 430 (Rust): no uppercase.
    Snake,
    /// mixedCaps / PascalCase — Effective Go, common TS: no underscores.
    MixedCaps,
    /// No convention enforced (Markdown).
    Any,
}

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
    /// Kinds whose N children are joined by N-1 anonymous short-circuit
    /// tokens invisible to cc_operators (Rust let_chain: no `operator`
    /// field). CC adds N-1; CoC adds one operator run.
    pub chain_kinds: &'static [&'static str],
    /// Cognitive: structures that increment AND raise nesting.
    pub coc_nesting_kinds: &'static [&'static str],
    /// Cognitive: flat +1 (no nesting penalty), e.g. `else`.
    pub coc_flat_kinds: &'static [&'static str],
    /// Cognitive: raise nesting only (lambdas / inline fns).
    pub coc_nest_only_kinds: &'static [&'static str],
    /// Cognitive: operator tokens forming counted boolean runs. Split
    /// from cc_operators: the whitepaper (v1.7 p.6 "Ignore shorthand")
    /// ignores null-coalescing in CoC while CC counts it as a branch.
    pub coc_operators: &'static [&'static str],
    /// Cognitive: jump statements adding a fundamental +1 when labeled
    /// (whitepaper p.8 "Jumps to labels": goto / break L / continue L).
    pub coc_jump_kinds: &'static [&'static str],
    /// Node kind of the label child that marks a jump as labeled
    /// (AST-probed: Go `label_name`, Rust `label`, TS
    /// `statement_identifier`).
    pub label_kinds: &'static [&'static str],
    pub comment_kinds: &'static [&'static str],
    /// Readability: function-name convention (plan §4.1 naming item).
    pub name_style: NameStyle,
    /// Dedup: anonymous string-delimiter tokens that count as literal
    /// pieces. Per-language because `'` is a QUOTE only where the
    /// grammar lexes char/string content separately — in Rust `'` is
    /// the lifetime/label tick (M2 attack review: classifying it LIT
    /// made every `&'a str` signature a false clone driver).
    pub literal_delims: &'static [&'static str],
    /// fn_kinds entries that form a unit ONLY when the node carries a
    /// `name` field. Haskell's `bind` kind is also the do-statement
    /// `x <- act` and pattern-bind kind (AST-probed 3k) — without the
    /// name gate every do line would become a standalone unit and its
    /// complexity would vanish from the host.
    pub fn_named_only_kinds: &'static [&'static str],
}

pub fn spec(lang: Lang) -> &'static LangSpec {
    match lang {
        Lang::Python => &PYTHON,
        Lang::TypeScript | Lang::Tsx => &TYPESCRIPT,
        Lang::Rust => &RUST,
        Lang::Go => &GO,
        Lang::Markdown => &MARKDOWN,
        Lang::Haskell => &super::spec_hs::HASKELL,
        // The sentinel is never walked; the scan-only arm (plan
        // v2.5) is size-only like Markdown — grammar() is None for
        // all of them, so measure_file never reaches these tables:
        // the empty MARKDOWN spec is the honest degenerate.
        _ => &MARKDOWN,
    }
}

static PYTHON: LangSpec = LangSpec {
    fn_kinds: &["function_definition"],
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
    coc_operators: &["and", "or"],
    // Python has no labeled jumps.
    coc_jump_kinds: &[],
    label_kinds: &[],
    comment_kinds: &["comment"],
    name_style: NameStyle::Snake,
    // Python quotes surface as named string_start/string_end kinds.
    literal_delims: &[],
    fn_named_only_kinds: &[],
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
    chain_kinds: &[],
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
    fn_named_only_kinds: &[],
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
        // `expr?` is an implicit early-return branch (== match Ok/Err);
        // rust-code-analysis counts it (M1 cross-check, ban.rs 21 vs 17).
        "try_expression",
    ],
    cc_operators: &["&&", "||"],
    // let_chain joins let_conditions/exprs with anonymous `&&` tokens
    // (no operator field — AST-probed; M1 attack review finding).
    chain_kinds: &["let_chain"],
    coc_nesting_kinds: &[
        "if_expression",
        "for_expression",
        "while_expression",
        "loop_expression",
        "match_expression",
    ],
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
    fn_named_only_kinds: &[],
};

static GO: LangSpec = LangSpec {
    // gocyclo attributes func-literal branches to the enclosing decl,
    // so func_literal is absorbed, not standalone.
    fn_kinds: &["function_declaration", "method_declaration"],
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
    coc_operators: &["&&", "||"],
    // goto always carries a label_name, so it always counts.
    coc_jump_kinds: &["continue_statement", "break_statement", "goto_statement"],
    label_kinds: &["label_name"],
    comment_kinds: &["comment"],
    name_style: NameStyle::MixedCaps,
    // `'` is Go's rune delimiter but rune_literal is a single token.
    literal_delims: &["\"", "`"],
    fn_named_only_kinds: &[],
};

static MARKDOWN: LangSpec = LangSpec {
    fn_kinds: &[],
    param_list_kinds: &[],
    cc_kinds: &[],
    cc_operators: &[],
    chain_kinds: &[],
    coc_nesting_kinds: &[],
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &[],
    coc_operators: &[],
    coc_jump_kinds: &[],
    label_kinds: &[],
    comment_kinds: &[],
    name_style: NameStyle::Any,
    literal_delims: &[],
    fn_named_only_kinds: &[],
};
