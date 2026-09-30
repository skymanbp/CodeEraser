//! The Java flow table (plan v2.31 step 4), beside the contract in
//! spec.rs; kinds read off the round-3 probe transcript of flow.java.
//! A `switch` is one kind in both positions (switch_expression), its arms
//! either colon groups that fall through or arrow rules that do not
//! (§5.1 rule 6); `yield` is a break to it (rule 4).

use super::spec::Table;

pub static JAVA: Table = Table::new(
    "java",
    &[r#"
block_kinds = ["block", "constructor_body"]
wrapper_kinds = ["expression_statement"]
if = {cond = "condition", then = "consequence", else = "alternative"}
cond_wrappers = [["parenthesized_expression", "*", ""]]
loops = [
  {kind = "for_statement", init = "init", cond = "condition", update = "update", body = "body"},
  {kind = "enhanced_for_statement", target = "name", iter = "value", body = "body"},
  {kind = "while_statement", cond = "condition", body = "body"},
  {kind = "do_statement", cond = "condition", body = "body", body_first = true},
]
switches = [{kind = "switch_expression", subject = "condition", arms = "body"}]
cases = [
  {kind = "switch_block_statement_group", default = {holds = ["@switch_label", "default"]}, fallthrough = "always", pattern = "@switch_label/@pattern", value = "@switch_label", guard = "@switch_label/@guard", body = "*"},
  {kind = "switch_rule", default = {holds = ["@switch_label", "default"]}, pattern = "@switch_label/@pattern", value = "@switch_label", guard = "@switch_label/@guard", body = "*"},
]
tries = [
  {kind = "try_statement", body = "body"},
  {kind = "try_with_resources_statement", body = "body", resources = "resources/@resource"},
]
catches = [{kind = "catch_clause", param = "@catch_formal_parameter/name", body = "body"}]
finally_kinds = [["finally_clause", "@block"]]
return_kinds = ["return_statement"]
throw_kinds = ["throw_statement"]
break_kinds = ["break_statement"]
continue_kinds = ["continue_statement"]
labels = [["labeled_statement", "@identifier", "*"]]
yield_kinds = ["yield_statement"]
noreturn = ["System.exit"]
const_true = [["true", ""]]
int_kinds = ["decimal_integer_literal"]
params = [["formal_parameter", "name"], ["spread_parameter", "@variable_declarator/name"]]
decls = [
  {kind = "local_variable_declaration", items = "declarator", binder = "name", init = "value"},
  {kind = "resource", binder = "name", init = "value"},
]
pattern_kinds = ["pattern", "record_pattern", "record_pattern_body", "record_pattern_component", "type_pattern"]
pattern_binders = [["instanceof_expression", "name"]]
ident_kinds = ["identifier"]
name_positions = [
  ["field_access", "field"], ["method_invocation", "name"], ["record_pattern", "@identifier"],
  ["scoped_identifier", "."], ["labeled_statement", "@identifier"], ["break_statement", "@identifier"],
  ["continue_statement", "@identifier"], ["marker_annotation", "name"],
  ["class_declaration", "name"], ["record_declaration", "name"],
]
member_write_bases = ["field_access", "array_access"]
assigns = [
  {kind = "assignment_expression", op = "=", left = "left", right = "right"},
  {kind = "assignment_expression", left = "left", right = "right", mode = "readwrite"},
]
update_kinds = [["update_expression", "*"]]
conditional_ctx = [
  ["binary_expression", "right", ["&&", "||"]],
  ["ternary_expression", "consequence", []], ["ternary_expression", "alternative", []],
  ["switch_expression", "body", []],
]
capture_kinds = ["lambda_expression", "class_body"]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_java.rs"]
mod lang_java;
