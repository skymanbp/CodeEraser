//! The Rust flow table (plan v2.31 step 4), beside the contract in
//! spec.rs. Every kind, field and token here is read off the round-3
//! probe transcript of scripts/tsprobe/snippets/flow.rs and checked
//! against the pinned grammar by the unit legs.

use super::spec::Table;

pub static RUST: Table = Table::new(
    "rust",
    &[r#"
block_kinds = ["block"]
wrapper_kinds = ["expression_statement", "unsafe_block"]
if = {cond = "condition", then = "consequence", else = "alternative"}
else_kinds = [["else_clause", "*"]]
loops = [
  {kind = "loop_expression", body = "body"},
  {kind = "while_expression", cond = "condition", body = "body"},
  {kind = "for_expression", target = "pattern", iter = "value", body = "body"},
]
switches = [{kind = "match_expression", subject = "value", arms = "body", always_default = true, passes_break = true}]
cases = [{kind = "match_arm", pattern = "pattern", guard = "pattern/condition", body = "value"}]
return_kinds = ["return_expression"]
break_kinds = ["break_expression"]
continue_kinds = ["continue_expression"]
self_label = "@label"
noreturn = ["panic!", "unreachable!", "todo!", "unimplemented!", "std::process::exit", "process::exit"]
macros = [{kind = "macro_invocation", name = "macro", args = "@token_tree", strings = "string_literal"}]
const_true = [["boolean_literal", "true"]]
int_kinds = ["integer_literal"]
params = [["parameter", "pattern"]]
decls = [{kind = "let_declaration", binder = "pattern", init = "value", alternative = "alternative"}]
pattern_kinds = [
  "tuple_pattern", "tuple_struct_pattern", "struct_pattern", "field_pattern",
  "captured_pattern", "reference_pattern", "match_pattern",
]
pattern_idents = ["shorthand_field_identifier"]
upper_pattern_paths = true
pattern_binders = [["let_condition", "pattern"]]
branch_binders = ["let_condition"]
ident_kinds = ["identifier"]
name_positions = [
  ["scoped_identifier", "."], ["label", "."], ["tuple_struct_pattern", "type"],
  ["macro_invocation", "macro"], ["use_declaration", "argument"],
]
member_write_bases = ["field_expression", "index_expression", "unary_expression"]
assigns = [
  {kind = "assignment_expression", left = "left", right = "right"},
  {kind = "compound_assignment_expr", left = "left", right = "right", mode = "readwrite"},
]
conditional_ctx = [
  ["binary_expression", "right", ["&&", "||"]],
  ["if_expression", "consequence", []], ["if_expression", "alternative", []],
  ["match_expression", "body", []],
]
capture_kinds = ["async_block"]
address_ops = [["reference_expression", "@mutable_specifier"]]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_rs.rs"]
mod lang_rs;
