//! The R slot table (plan v2.31 step 7), beside the contract in
//! slot.rs: R has no type position.

pub const R: &str = r#"
expr_kinds = [
  "binary_operator", "unary_operator", "call", "extract_operator", "namespace_operator", "subset",
  "subset2", "float", "integer", "string", "string_content", "true", "false", "null", "identifier",
  "function_definition", "parenthesized_expression",
]
name_fields = [["parameter", "name"]]
other_kinds = [
  "program", "braced_expression", "parameters", "parameter", "dots", "arguments", "argument",
  "comma", "string_open", "string_close", "comment",
]
"#;
