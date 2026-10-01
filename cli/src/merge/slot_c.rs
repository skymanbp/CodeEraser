//! Slot tables for C and C++ (plan v2.31 step 7), beside the contract in
//! slot.rs: C++ reads the shared piece and its own, C reads the shared
//! piece and the one kind only its grammar has — so the shared piece
//! names no kind either grammar lacks.

pub const C: &str = r#"
expr_kinds = [
  "binary_expression", "call_expression", "comma_expression", "conditional_expression",
  "field_expression", "identifier", "number_literal", "char_literal", "parenthesized_expression",
  "pointer_expression", "string_literal", "string_content", "escape_sequence",
  "concatenated_string", "subscript_expression", "true", "false", "null", "sizeof_expression",
  "cast_expression", "unary_expression",
]
type_kinds = [
  "primitive_type", "type_identifier", "sized_type_specifier", "struct_specifier",
  "type_descriptor",
]
name_fields = [
  ["function_declarator", "declarator"], ["init_declarator", "declarator"],
  ["parameter_declaration", "declarator"], ["pointer_declarator", "declarator"],
  ["array_declarator", "declarator"], ["field_declaration", "declarator"],
]
other_kinds = [
  "translation_unit", "function_definition", "function_declarator", "parameter_list",
  "parameter_declaration", "init_declarator", "pointer_declarator",
  "array_declarator", "parenthesized_declarator", "initializer_list", "field_declaration",
  "field_declaration_list", "field_identifier", "compound_statement", "argument_list",
  "assignment_expression", "update_expression", "storage_class_specifier", "type_qualifier",
  "attribute", "attribute_declaration", "attribute_specifier", "gnu_asm_expression",
  "gnu_asm_qualifier", "statement_identifier", "preproc_include", "preproc_defined",
  "preproc_elif", "preproc_else", "system_lib_string", "comment",
]
"#;

/// The one C kind C++ spells otherwise: C's `...` parameter.
pub const C_ONLY: &str = r#"
other_kinds = ["variadic_parameter"]
"#;

pub const CPP: &str = r#"
expr_kinds = ["lambda_expression", "this", "qualified_identifier", "compound_literal_expression"]
type_kinds = ["auto", "placeholder_type_specifier", "template_type", "template_argument_list"]
name_fields = [["optional_parameter_declaration", "declarator"]]
other_kinds = [
  "abstract_function_declarator", "namespace_identifier", "condition_clause",
  "lambda_capture_specifier", "lambda_default_capture", "optional_parameter_declaration",
  "reference_declarator", "structured_binding_declarator", "subscript_argument_list",
  "field_initializer_list", "field_initializer",
]
"#;
