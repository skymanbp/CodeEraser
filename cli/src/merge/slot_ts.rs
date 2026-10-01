//! Slot tables for TypeScript and TSX (plan v2.31 step 7), beside the
//! contract in slot.rs: the two share one piece, each adding its own —
//! TypeScript's `<T>x`, TSX's JSX kinds — so the shared piece names no
//! kind either grammar lacks. Read off the flow probes and checked
//! against the pinned grammars by the unit legs.

pub const TYPESCRIPT: &str = r#"
expr_kinds = [
  "array", "arrow_function", "binary_expression", "call_expression", "function_expression",
  "identifier", "member_expression", "new_expression", "number", "object", "parenthesized_expression",
  "string", "template_string", "subscript_expression", "ternary_expression",
  "this", "true", "false", "null", "undefined", "unary_expression", "yield_expression",
  "await_expression", "as_expression", "non_null_expression", "shorthand_property_identifier",
]
type_kinds = [
  "type_annotation", "predefined_type", "type_identifier", "nested_type_identifier", "generic_type",
  "array_type", "function_type", "object_type", "union_type", "literal_type", "tuple_type",
  "type_arguments", "type_query",
]
name_fields = [
  ["function_declaration", "name"], ["generator_function_declaration", "name"],
  ["class_declaration", "name"], ["method_definition", "name"], ["variable_declarator", "name"],
  ["required_parameter", "pattern"], ["optional_parameter", "pattern"],
  ["interface_declaration", "name"], ["type_alias_declaration", "name"],
  ["public_field_definition", "name"], ["method_signature", "name"], ["property_signature", "name"],
]
target_fields = [
  ["assignment_expression", "left"], ["augmented_assignment_expression", "left"],
  ["update_expression", "argument"],
]
part_fields = [["member_expression", "property"]]
part_kinds = ["string_fragment"]
helper_lines = 2
other_kinds = [
  "program", "function_declaration", "generator_function_declaration", "class_declaration",
  "class_body", "method_definition", "method_signature", "public_field_definition",
  "accessibility_modifier", "interface_declaration", "interface_body", "property_signature",
  "type_alias_declaration", "ambient_declaration", "statement_block",
  "switch_body", "statement_identifier", "formal_parameters", "required_parameter",
  "optional_parameter", "rest_pattern", "object_pattern", "array_pattern", "pair_pattern",
  "shorthand_property_identifier_pattern", "variable_declarator", "arguments", "pair",
  "spread_element", "property_identifier", "assignment_expression",
  "augmented_assignment_expression", "update_expression", "export_statement", "import_statement",
  "import_clause", "namespace_import", "named_imports", "import_specifier", "comment",
]
"#;

pub const TS_ONLY: &str = r#"
expr_kinds = ["type_assertion"]
"#;

pub const JSX: &str = r#"
expr_kinds = ["jsx_element", "jsx_self_closing_element", "jsx_text"]
other_kinds = ["jsx_opening_element", "jsx_closing_element", "jsx_attribute", "jsx_expression"]
"#;
