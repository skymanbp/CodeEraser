//! The Haskell slot table (plan v2.31 step 7, step-7 ruling 8), beside
//! the contract in slot.rs. Haskell has no flow table, so this table
//! names its own statement forms and containers: the do block's
//! statements and the declarations (a declaration held by `declarations`,
//! `local_binds` or a class or instance body stands in a statement
//! position through its container). The grammar spells an expression,
//! a pattern and a type with the same kinds (`apply`, `variable`,
//! `tuple`…), so the type class holds only the kinds that are types
//! alone — a type constructor's `name`, the arrow `function` (a function
//! declaration reaches 0 through its container first), `forall`,
//! `context` and the like — and a shared kind reads as an expression.
//! Every kind and field here is read off the probe
//! (scripts/tsprobe/snippets/flow.hs) and tree-sitter-haskell 0.23.1's
//! node-types.json.

pub const HASKELL: &str = r#"
stmt_kinds = [
  "bind", "exp", "let", "rec", "signature", "data_type", "newtype", "class", "instance", "import",
  "type_synomym", "fixity", "deriving_instance", "foreign_import", "foreign_export", "type_family",
  "data_family", "type_instance", "data_instance", "kind_signature", "pattern_synonym",
  "top_splice", "default_types", "role_annotation", "default_signature",
]
container_kinds = [
  "do", "declarations", "local_binds", "class_declarations", "instance_declarations", "imports",
]
expr_kinds = [
  "apply", "infix", "variable", "literal", "integer", "float", "char", "string", "parens",
  "lambda", "lambda_case", "lambda_cases", "case", "do", "conditional", "let_in", "list", "tuple",
  "unit", "constructor", "operator", "qualified", "left_section", "right_section", "negation",
  "record", "projection", "list_comprehension", "arithmetic_sequence", "multi_way_if", "prefix_id",
  "infix_id", "wildcard", "label", "quasiquote", "implicit_variable", "empty_list",
]
type_kinds = [
  "name", "function", "star", "forall", "forall_required", "context", "promoted", "prefix_list",
  "linear_function", "kind_application", "type_application",
]
name_fields = [["function", "name"], ["bind", "name"], ["signature", "name"]]
other_kinds = [
  "haskell", "header", "module", "module_id", "exports", "export", "children", "all_names",
  "imports", "import_list", "import_name", "declarations", "class_declarations",
  "instance_declarations", "local_binds", "match", "guards", "boolean", "pattern_guard",
  "alternatives", "alternative",
  "patterns", "data_constructors", "data_constructor", "prefix", "deriving", "fields", "field",
  "field_name", "field_update", "newtype_constructor", "type_params", "type_patterns", "generator",
  "qualifiers", "comment", "haddock", "pragma", "cpp",
]
"#;
