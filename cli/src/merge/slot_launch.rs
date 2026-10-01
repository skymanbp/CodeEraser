//! Slot tables for Python, Rust and Go (plan v2.31 step 7), beside the
//! contract in slot.rs. Every kind and field here is read off the flow
//! probes (scripts/tsprobe/snippets/flow.*) and checked against the
//! pinned grammar by the unit legs; TypeScript and TSX are slot_ts.rs.

pub const PYTHON: &str = r#"
expr_kinds = [
  "attribute", "binary_operator", "boolean_operator", "call", "comparison_operator",
  "conditional_expression", "dictionary_comprehension", "expression_list", "generator_expression",
  "identifier", "integer", "float", "lambda", "list_comprehension", "named_expression", "none",
  "true", "false", "parenthesized_expression", "set_comprehension", "string", "string_content",
  "interpolation", "concatenated_string", "subscript", "slice", "tuple", "list", "dictionary", "set",
  "unary_operator", "await",
]
type_kinds = ["type"]
name_fields = [
  ["function_definition", "name"], ["class_definition", "name"], ["default_parameter", "name"],
  ["typed_default_parameter", "name"],
]
other_kinds = [
  "module", "function_definition", "class_definition", "decorated_definition", "decorator", "block",
  "parameters", "lambda_parameters", "default_parameter", "typed_parameter", "typed_default_parameter",
  "list_splat_pattern", "dictionary_splat_pattern", "keyword_separator", "positional_separator",
  "argument_list", "keyword_argument", "pair", "assignment", "augmented_assignment", "pattern_list",
  "tuple_pattern", "for_in_clause", "if_clause", "with_clause", "with_item", "as_pattern",
  "as_pattern_target", "case_pattern", "class_pattern", "dict_pattern", "list_pattern",
  "splat_pattern", "keyword_pattern", "union_pattern", "dotted_name", "import_statement",
  "import_from_statement", "string_start", "string_end", "comment",
]
"#;

pub const RUST: &str = r#"
expr_kinds = [
  "array_expression", "binary_expression", "boolean_literal", "call_expression",
  "closure_expression", "field_expression", "identifier", "index_expression", "integer_literal",
  "float_literal", "negative_literal", "char_literal", "range_expression", "reference_expression",
  "string_literal", "string_content", "struct_expression", "try_expression", "tuple_expression",
  "type_cast_expression", "unary_expression", "macro_invocation", "scoped_identifier", "self",
  "await_expression", "unit_expression", "parenthesized_expression", "shorthand_field_initializer",
  "raw_string_literal",
]
type_kinds = [
  "abstract_type", "array_type", "function_type", "generic_type", "never_type", "pointer_type",
  "primitive_type", "reference_type", "tuple_type", "type_arguments", "type_identifier",
  "scoped_type_identifier", "dynamic_type",
]
name_fields = [
  ["function_item", "name"], ["struct_item", "name"], ["enum_item", "name"], ["trait_item", "name"],
  ["const_item", "name"], ["static_item", "name"], ["type_item", "name"], ["parameter", "pattern"],
  ["let_declaration", "pattern"], ["field_declaration", "name"],
]
other_kinds = [
  "source_file", "function_item", "struct_item", "impl_item", "declaration_list",
  "field_declaration", "field_declaration_list", "use_declaration", "visibility_modifier",
  "attribute_item", "attribute", "parameters", "parameter", "self_parameter", "closure_parameters",
  "block", "async_block", "arguments", "assignment_expression", "compound_assignment_expr",
  "field_identifier", "field_initializer", "field_initializer_list", "shorthand_field_identifier",
  "let_condition", "match_block", "match_pattern", "captured_pattern", "field_pattern",
  "range_pattern", "reference_pattern", "struct_pattern", "tuple_pattern", "tuple_struct_pattern",
  "mutable_specifier", "label", "token_tree", "line_comment", "block_comment", "doc_comment",
  "inner_doc_comment_marker", "outer_doc_comment_marker",
]
"#;

pub const GO: &str = r#"
expr_kinds = [
  "binary_expression", "call_expression", "composite_literal", "func_literal", "identifier",
  "index_expression", "int_literal", "float_literal", "rune_literal", "interpreted_string_literal",
  "interpreted_string_literal_content", "raw_string_literal", "selector_expression", "true",
  "false", "nil", "iota", "unary_expression", "expression_list", "parenthesized_expression",
  "slice_expression", "type_assertion_expression",
]
type_kinds = [
  "channel_type", "function_type", "interface_type", "map_type", "pointer_type", "slice_type",
  "struct_type", "type_identifier", "array_type", "qualified_type", "generic_type",
]
name_fields = [
  ["function_declaration", "name"], ["method_declaration", "name"],
  ["parameter_declaration", "name"], ["variadic_parameter_declaration", "name"],
  ["var_spec", "name"], ["const_spec", "name"], ["type_spec", "name"],
  ["field_declaration", "name"],
]
other_kinds = [
  "source_file", "package_clause", "package_identifier", "import_declaration", "import_spec",
  "import_spec_list", "function_declaration", "method_declaration", "type_declaration", "type_spec",
  "var_spec", "var_spec_list", "const_spec", "parameter_list", "parameter_declaration",
  "variadic_parameter_declaration", "field_declaration", "field_declaration_list",
  "field_identifier", "block", "statement_list", "argument_list", "literal_value",
  "literal_element", "keyed_element", "for_clause", "range_clause", "inc_statement",
  "dec_statement", "send_statement", "label_name", "comment",
]
"#;
