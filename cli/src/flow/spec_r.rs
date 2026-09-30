//! The R flow table (plan v2.31 step 4), beside the contract in spec.rs;
//! kinds read off the round-3 probe transcript of flow.R. R has no
//! return statement — `return(x)` is a call, read by name (§5.1 rule 3)
//! — and every assignment is one binary_operator told apart by its
//! operator token, `->` and `->>` writing to their right (rules 8, 9).

use super::spec::Table;

pub static R: Table = Table::new(
    "r",
    &[r#"
block_kinds = ["braced_expression"]
if = {cond = "condition", then = "consequence", else = "alternative"}
cond_wrappers = [["parenthesized_expression", "body", ""]]
loops = [
  {kind = "for_statement", target = "variable", iter = "sequence", body = "body"},
  {kind = "while_statement", cond = "condition", body = "body"},
  {kind = "repeat_statement", body = "body"},
]
break_kinds = ["break"]
continue_kinds = ["next"]
noreturn = ["stop", "quit", "q", "abort", "cli_abort", "rlang::abort", "cli::cli_abort"]
return_calls = ["return"]
dispatch_calls = ["UseMethod", "NextMethod", "standardGeneric", "callNextMethod"]
const_true = [["true", ""]]
int_kinds = ["integer", "float"]
dynamic_names = [
  "eval", "evalq", "eval.parent", "parse", "assign", "get", "get0", "mget", "exists", "rm",
  "environment", "sys.function", "parent.frame", "local", "with", "within", "attach", "source",
]
scoping = "function"
first_write_declares = true
params = [["parameter", "name"]]
ident_kinds = ["identifier"]
name_positions = [["extract_operator", "rhs"], ["namespace_operator", "."], ["argument", "name"]]
interpolated_strings = ["string"]
member_write_bases = ["extract_operator", "subset", "subset2", "call"]
assigns = [
  {kind = "binary_operator", op = "<-", left = "lhs", right = "rhs"},
  {kind = "binary_operator", op = "=", left = "lhs", right = "rhs"},
  {kind = "binary_operator", op = "->", left = "rhs", right = "lhs"},
  {kind = "binary_operator", op = "<<-", left = "lhs", right = "rhs", mode = "outer"},
  {kind = "binary_operator", op = "->>", left = "rhs", right = "lhs", mode = "outer"},
]
conditional_ctx = [
  ["binary_operator", "rhs", ["&&", "||"]],
  ["if_statement", "consequence", []], ["if_statement", "alternative", []],
]
default_arg_fields = [["parameter", "default"]]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_r.rs"]
mod lang_r;
