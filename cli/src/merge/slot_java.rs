//! The Java slot table (plan v2.31 step 7), beside the contract in
//! slot.rs.

pub const JAVA: &str = r#"
expr_kinds = [
  "array_access", "array_creation_expression", "array_initializer", "binary_expression",
  "field_access", "identifier", "instanceof_expression", "lambda_expression", "method_invocation",
  "null_literal", "object_creation_expression", "parenthesized_expression", "string_literal",
  "string_fragment", "ternary_expression", "this", "true", "false", "unary_expression",
  "decimal_integer_literal", "decimal_floating_point_literal", "hex_integer_literal",
  "character_literal", "cast_expression", "method_reference", "class_literal",
]
type_kinds = [
  "array_type", "boolean_type", "generic_type", "integral_type", "floating_point_type",
  "type_arguments", "type_identifier", "scoped_type_identifier", "void_type", "catch_type",
]
name_fields = [
  ["method_declaration", "name"], ["class_declaration", "name"], ["constructor_declaration", "name"],
  ["record_declaration", "name"], ["interface_declaration", "name"], ["enum_declaration", "name"],
  ["formal_parameter", "name"], ["catch_formal_parameter", "name"], ["variable_declarator", "name"],
]
other_kinds = [
  "program", "package_declaration", "import_declaration", "scoped_identifier", "class_declaration",
  "class_body", "record_declaration", "method_declaration", "constructor_declaration",
  "compact_constructor_declaration", "constructor_body", "field_declaration", "modifiers",
  "marker_annotation", "annotation", "formal_parameters", "formal_parameter",
  "receiver_parameter", "spread_parameter", "catch_formal_parameter", "variable_declarator",
  "dimensions", "dimensions_expr", "block", "argument_list", "assignment_expression",
  "update_expression", "resource_specification", "switch_block", "switch_label", "guard",
  "pattern", "type_pattern", "record_pattern", "record_pattern_body", "record_pattern_component",
  "line_comment", "block_comment",
]
"#;
