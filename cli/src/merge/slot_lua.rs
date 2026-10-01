//! The Lua slot table (plan v2.31 step 7), beside the contract in
//! slot.rs: Lua has no type position.

pub const LUA: &str = r#"
expr_kinds = [
  "binary_expression", "bracket_index_expression", "dot_index_expression",
  "method_index_expression", "function_call", "function_definition", "identifier", "number",
  "string", "true", "false", "nil", "vararg_expression", "table_constructor",
  "parenthesized_expression", "unary_expression",
]
name_fields = [["function_declaration", "name"]]
target_fields = [["variable_list", "name"]]
part_fields = [["dot_index_expression", "field"], ["method_index_expression", "method"]]
part_kinds = ["string_content"]
helper_lines = 2
other_kinds = [
  "chunk", "function_declaration", "parameters", "block", "arguments", "assignment_statement",
  "variable_list", "expression_list", "attribute", "field", "for_generic_clause",
  "for_numeric_clause", "comment", "comment_content",
]
"#;
