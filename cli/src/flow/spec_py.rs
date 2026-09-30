//! The Python flow table (plan v2.31 step 4), beside the contract in
//! spec.rs. Every kind, field and token here is read off the round-3
//! probe transcript of scripts/tsprobe/snippets/flow.py and checked
//! against the pinned grammar by the unit legs.

use super::spec::Table;

pub static PYTHON: Table = Table::new(
    "python",
    &[r#"
block_kinds = ["block"]
wrapper_kinds = ["expression_statement"]
if = {cond = "condition", then = "consequence", else = "alternative"}
elif_kinds = ["elif_clause"]
else_kinds = [["else_clause", "body"]]
cond_wrappers = [["parenthesized_expression", "*", ""]]
loops = [
  {kind = "for_statement", target = "left", iter = "right", body = "body", else = "alternative"},
  {kind = "while_statement", cond = "condition", body = "body", else = "alternative"},
]
switches = [{kind = "match_statement", subject = "subject", arms = "body", passes_break = true}]
cases = [{kind = "case_clause", default = {holds = ["@case_pattern", "_"]}, pattern = "@case_pattern", guard = "guard", body = "consequence"}]
tries = [{kind = "try_statement", body = "body", else = "@else_clause"}]
catches = [{kind = "except_clause", param = "value/alias", value = "value", body = "@block"}]
finally_kinds = [["finally_clause", "@block"]]
withs = [{kind = "with_statement", item = "@with_clause/@with_item", value = "value", binder = "value/alias", body = "body"}]
return_kinds = ["return_statement"]
throw_kinds = ["raise_statement"]
break_kinds = ["break_statement"]
continue_kinds = ["continue_statement"]
noreturn = ["sys.exit", "exit", "quit", "os._exit", "os.abort"]
const_true = [["true", ""]]
int_kinds = ["integer"]
dynamic_names = ["eval", "exec", "compile", "locals", "globals", "vars", "__import__"]
scoping = "function"
first_write_declares = true
params = [
  ["default_parameter", "name"],
  ["typed_parameter", "@identifier|@list_splat_pattern|@dictionary_splat_pattern"],
  ["typed_default_parameter", "name"],
]
receiver_names = ["self", "cls"]
pattern_kinds = [
  "pattern_list", "tuple_pattern", "list_pattern", "list_splat_pattern", "dictionary_splat_pattern",
  "case_pattern", "dict_pattern", "splat_pattern", "class_pattern", "keyword_pattern",
  "union_pattern", "as_pattern", "as_pattern_target",
]
dotted_patterns = ["dotted_name"]
nonlocal_kinds = ["global_statement", "nonlocal_statement"]
local_only_scopes = ["list_comprehension", "set_comprehension", "dictionary_comprehension", "generator_expression", "lambda"]
ident_kinds = ["identifier"]
name_positions = [
  ["attribute", "attribute"], ["keyword_argument", "name"], ["keyword_pattern", "@identifier"],
  ["class_pattern", "@dotted_name"], ["type", "."],
]
member_write_bases = ["attribute", "subscript"]
assigns = [
  {kind = "assignment", left = "left", right = "right"},
  {kind = "augmented_assignment", left = "left", right = "right", mode = "readwrite"},
  {kind = "named_expression", left = "name", right = "value"},
]
conditional_ctx = [["boolean_operator", "right", []], ["conditional_expression", "*", []]]
default_arg_fields = [["default_parameter", "value"], ["typed_default_parameter", "value"]]
capture_kinds = ["lambda", "class_definition", "generator_expression"]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_py.rs"]
mod lang_py;
