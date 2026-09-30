//! The Go flow table (plan v2.31 step 4), beside the contract in
//! spec.rs. Every kind, field and token here is read off the round-3
//! probe transcript of scripts/tsprobe/snippets/flow.go and checked
//! against the pinned grammar by the unit legs.

use super::spec::Table;

pub static GO: Table = Table::new(
    "go",
    &[r#"
block_kinds = ["block"]
splice_kinds = ["statement_list"]
wrapper_kinds = ["expression_statement"]
if = {cond = "condition", then = "consequence", else = "alternative", init = "initializer"}
loops = [
  {kind = "for_statement", header = "@for_clause", init = "initializer", cond = "condition", update = "update", body = "body"},
  {kind = "for_statement", header = "@range_clause", target = "left", iter = "right", body = "body", marker = [[":=", "block"]]},
  {kind = "for_statement", header = "*", cond = ".", body = "body"},
  {kind = "for_statement", body = "body"},
]
switches = [
  {kind = "expression_switch_statement", subject = "value", arms = ".", init = "initializer"},
  {kind = "type_switch_statement", subject = "value", arms = ".", init = "initializer", binder = "alias"},
  {kind = "select_statement", arms = ".", always_default = true, empty_noreturn = true},
]
cases = [
  {kind = "expression_case", fallthrough = "statement", value = "value", body = "@statement_list"},
  {kind = "type_case", body = "@statement_list"},
  {kind = "communication_case", value = "communication", body = "@statement_list"},
  {kind = "default_case", default = "kind", fallthrough = "statement", body = "@statement_list"},
]
return_kinds = ["return_statement"]
break_kinds = ["break_statement"]
continue_kinds = ["continue_statement"]
gotos = [["goto_statement", "@label_name"]]
labels = [["labeled_statement", "label", "*"]]
fallthrough_kinds = ["fallthrough_statement"]
noreturn = [
  "panic", "os.Exit", "log.Fatal", "log.Fatalf", "log.Fatalln",
  "log.Panic", "log.Panicf", "log.Panicln", "runtime.Goexit",
]
const_true = [["true", ""]]
int_kinds = ["int_literal"]
params = [["parameter_declaration", "name"], ["variadic_parameter_declaration", "name"]]
results = "result"
decls = [
  {kind = "short_var_declaration", binder = "left", init = "right", redeclare = true},
  {kind = "var_declaration", items = "@var_spec|@var_spec_list/@var_spec", binder = "name", init = "value"},
  {kind = "receive_statement", token = ":=", binder = "left", init = "right"},
]
pattern_kinds = ["expression_list"]
ident_kinds = ["identifier"]
name_positions = [["const_spec", "name"]]
member_write_bases = ["selector_expression", "index_expression", "unary_expression"]
assigns = [
  {kind = "assignment_statement", op = "=", left = "left", right = "right"},
  {kind = "assignment_statement", left = "left", right = "right", mode = "readwrite"},
  {kind = "receive_statement", op = "=", left = "left", right = "right"},
]
update_kinds = [["inc_statement", "*"], ["dec_statement", "*"]]
conditional_ctx = [["binary_expression", "right", ["&&", "||"]]]
capture_kinds = ["func_literal"]
address_ops = [["unary_expression", "&"]]
discard_names = ["_"]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_go.rs"]
mod lang_go;
