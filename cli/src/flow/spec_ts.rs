//! The TypeScript flow table (plan v2.31 step 4), beside the contract in
//! spec.rs. Every kind, field and token here is read off the round-3
//! probe transcripts of scripts/tsprobe/snippets/flow.ts and flow.tsx
//! and checked against the pinned grammar by the unit legs; TypeScript
//! and TSX share it (the TSX grammar's kinds are a superset, less the
//! `<T>x` type_assertion no row names).

use super::spec::Table;

pub static TYPESCRIPT: Table = Table::new(
    "typescript",
    &[r#"
block_kinds = ["statement_block"]
wrapper_kinds = ["expression_statement"]
empty_kinds = ["empty_statement"]
if = {cond = "condition", then = "consequence", else = "alternative"}
else_kinds = [["else_clause", "*"]]
cond_wrappers = [["parenthesized_expression", "*", ""]]
loops = [
  {kind = "for_statement", init = "initializer", cond = "condition", update = "increment", body = "body"},
  {kind = "for_in_statement", target = "left", iter = "right", body = "body", marker = [["const", "block"], ["let", "block"], ["var", "function"]]},
  {kind = "while_statement", cond = "condition", body = "body"},
  {kind = "do_statement", cond = "condition", body = "body", body_first = true},
]
switches = [{kind = "switch_statement", subject = "value", arms = "body"}]
cases = [
  {kind = "switch_case", fallthrough = "always", value = "value", body = "body"},
  {kind = "switch_default", default = "kind", fallthrough = "always", body = "body"},
]
tries = [{kind = "try_statement", body = "body"}]
catches = [{kind = "catch_clause", param = "parameter", body = "body"}]
finally_kinds = [["finally_clause", "body"]]
return_kinds = ["return_statement"]
throw_kinds = ["throw_statement"]
break_kinds = ["break_statement"]
continue_kinds = ["continue_statement"]
labels = [["labeled_statement", "label", "body"]]
noreturn = ["process.exit"]
call_forms = [["new_expression", "constructor"]]
const_true = [["true", ""]]
int_kinds = ["number"]
dynamic_names = ["eval", "Function"]
dynamic_kinds = ["with_statement"]
params = [["required_parameter", "pattern"], ["optional_parameter", "pattern"]]
decls = [
  {kind = "lexical_declaration", items = "@variable_declarator", binder = "name", init = "value"},
  {kind = "variable_declaration", scope = "function", items = "@variable_declarator", binder = "name", init = "value"},
]
pattern_kinds = ["object_pattern", "array_pattern", "rest_pattern", "pair_pattern"]
pattern_idents = ["shorthand_property_identifier_pattern"]
ident_kinds = ["identifier"]
name_positions = [["type_annotation", "."]]
shorthand_kinds = ["shorthand_property_identifier"]
member_write_bases = ["member_expression", "subscript_expression"]
assigns = [
  {kind = "assignment_expression", left = "left", right = "right"},
  {kind = "augmented_assignment_expression", left = "left", right = "right", mode = "readwrite"},
]
update_kinds = [["update_expression", "argument"]]
conditional_ctx = [
  ["binary_expression", "right", ["&&", "||", "??"]],
  ["ternary_expression", "consequence", []], ["ternary_expression", "alternative", []],
]
default_arg_fields = [["required_parameter", "value"]]
capture_kinds = ["class_declaration"]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_ts.rs"]
mod lang_ts;
