//! Flow tables for the C family (plan v2.31 step 4), beside the contract
//! in spec.rs. C and C++ read one text, FAMILY, and part only on the
//! noreturn names (§5.1 rule 3: C++ adds `std::exit`, `std::abort` and
//! `std::terminate`) — the text form of scan::spec_c's `..FAMILY`.
//! FAMILY names C++-only constructs (try, lambdas, range-for, condition
//! clauses, references, default arguments, `and` / `or`): a C tree never
//! holds them, and the unit legs pin that exact list as absent from the
//! C grammar and present in the C++ one. Kinds are read off the round-3
//! probe transcripts of flow.c and flow.cpp.

use super::spec::Table;

const FAMILY: &str = r#"
block_kinds = ["compound_statement"]
wrapper_kinds = ["expression_statement", "attributed_statement", "init_statement"]
if = {cond = "condition", then = "consequence", else = "alternative"}
else_kinds = [["else_clause", "*"]]
cond_wrappers = [["parenthesized_expression", "*", ""], ["condition_clause", "value", "initializer"]]
loops = [
  {kind = "for_statement", init = "initializer", cond = "condition", update = "update", body = "body"},
  {kind = "for_range_loop", target = "declarator", iter = "right", body = "body"},
  {kind = "while_statement", cond = "condition", body = "body"},
  {kind = "do_statement", cond = "condition", body = "body", body_first = true},
]
switches = [{kind = "switch_statement", subject = "condition", arms = "body"}]
cases = [{kind = "case_statement", default = {missing = "value"}, fallthrough = "always", value = "value", body = "*"}]
tries = [{kind = "try_statement", body = "body"}]
catches = [{kind = "catch_clause", param = "parameters/@parameter_declaration/declarator", body = "body"}]
return_kinds = ["return_statement"]
throw_kinds = ["throw_statement"]
break_kinds = ["break_statement"]
continue_kinds = ["continue_statement"]
gotos = [["goto_statement", "label"]]
labels = [["labeled_statement", "label", "*"]]
noreturn_attrs = [["type_qualifier", "_Noreturn"], ["attribute_specifier", "noreturn"], ["attribute", "noreturn"]]
const_true = [["true", ""]]
int_kinds = ["number_literal"]
dynamic_names = ["setjmp"]
dynamic_kinds = ["preproc_if", "preproc_ifdef", "preproc_elif", "preproc_else", "gnu_asm_expression"]
params = [["parameter_declaration", "declarator"], ["optional_parameter_declaration", "declarator"]]
decls = [
  {kind = "declaration", token = "=", binder = "declarator", init = "value"},
  {kind = "declaration", items = "declarator", pair = "init_declarator", binder = "declarator", init = "value", storage = "@storage_class_specifier"},
]
lasting_storage = ["static", "extern"]
prototype_kinds = ["function_declarator"]
prototype_reads = ["type_identifier"]
pattern_kinds = ["structured_binding_declarator"]
binder_paths = [
  ["pointer_declarator", "declarator"], ["array_declarator", "declarator"],
  ["parenthesized_declarator", "*"], ["function_declarator", "declarator"],
  ["reference_declarator", "*"],
]
ident_kinds = ["identifier"]
name_positions = [["qualified_identifier", "."], ["attribute", "name"], ["attribute_specifier", "."]]
member_write_bases = ["field_expression", "subscript_expression", "pointer_expression"]
assigns = [
  {kind = "assignment_expression", op = "=", left = "left", right = "right"},
  {kind = "assignment_expression", left = "left", right = "right", mode = "readwrite"},
]
update_kinds = [["update_expression", "argument"]]
conditional_ctx = [
  ["binary_expression", "right", ["&&", "||", "and", "or"]],
  ["conditional_expression", "consequence", []], ["conditional_expression", "alternative", []],
]
default_arg_fields = [["optional_parameter_declaration", "default_value"]]
head_reads = ["field_initializer_list"]
capture_kinds = ["lambda_expression", "function_definition"]
address_ops = [["pointer_expression", "&"]]
ref_binding_kinds = ["reference_declarator"]
"#;

/// Rule 3's C names, the list left open: C++ adds its three `std::`
/// spellings (STD) before END closes it, so no name is written twice.
const NORETURN: &str = r#"
noreturn = [
  "exit", "_exit", "_Exit", "abort", "quick_exit", "longjmp", "siglongjmp",
  "__builtin_unreachable", "__builtin_trap",
"#;
const STD: &str = r#"  "std::exit", "std::abort", "std::terminate",
"#;
const END: &str = "]\n";

pub static C: Table = Table::new("c", &[FAMILY, NORETURN, END]);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_c.rs"]
mod lang_c;

pub static CPP: Table = Table::new("cpp", &[FAMILY, NORETURN, STD, END]);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_cpp.rs"]
mod lang_cpp;
