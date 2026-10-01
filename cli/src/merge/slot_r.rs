//! The R slot table (plan v2.31 step 7), beside the contract in
//! slot.rs: R has no type position.

pub const R: &str = r#"
expr_kinds = [
  "binary_operator", "unary_operator", "call", "extract_operator", "namespace_operator", "subset",
  "subset2", "float", "integer", "string", "true", "false", "null", "identifier",
  "function_definition", "parenthesized_expression",
]
name_fields = [["parameter", "name"]]
target_ops = [
  ["binary_operator", "lhs", "<-"], ["binary_operator", "lhs", "<<-"],
  ["binary_operator", "lhs", "="], ["binary_operator", "rhs", "->"],
  ["binary_operator", "rhs", "->>"],
]
part_fields = [["extract_operator", "rhs"]]
part_kinds = ["string_content"]
helper_lines = 2
other_kinds = [
  "program", "braced_expression", "parameters", "parameter", "dots", "arguments", "argument",
  "comma", "string_open", "string_close", "comment",
]
"#;
