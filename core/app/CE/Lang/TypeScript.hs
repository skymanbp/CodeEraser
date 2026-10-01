-- | The typescript definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_launch.rs, cli/src/merge/slot_ts.rs,
-- cli/src/graph/spec.rs, cli/src/fourclass/kinds.rs at e877f389; from this
-- commit on the core is the authority.
module CE.Lang.TypeScript where

top :: String
top =
  "extra = [\n\
  \  'class_declaration', 'interface_declaration', 'enum_declaration', 'lexical_declaration',\n\
  \  'variable_declaration',\n\
  \]\n\
  \sites = [\n\
  \  {node = 'import_statement', label = 'import', via = {form = 'field', arg = 'source'}},\n\
  \  # `import fs = require(\"./b\")` hangs `source` off this clause\n\
  \  {node = 'import_require_clause', label = 'import', via = {form = 'field', arg = 'source'}},\n\
  \  # the star form first: a re-export target is the mounts table's bit 0\n\
  \  {node = 'export_statement', label = 'export_star', via = {form = 'field_if_star', arg = 'source'}},\n\
  \  {node = 'export_statement', label = 'export_from', via = {form = 'field', arg = 'source'}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \fn_kinds = [\n\
  \  'function_declaration',\n\
  \  'generator_function_declaration',\n\
  \  'method_definition',\n\
  \  'arrow_function',\n\
  \  'function_expression',\n\
  \]\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['formal_parameters']\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'ternary_expression',\n\
  \  'for_statement',\n\
  \  'for_in_statement',\n\
  \  'while_statement',\n\
  \  'do_statement',\n\
  \  'switch_case',\n\
  \  'catch_clause',\n\
  \]\n\
  \cc_operators = ['&&', '||', '??']\n\
  \chain_kinds = []\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'for_in_statement body',\n\
  \  'while_statement body',\n\
  \  'do_statement body',\n\
  \  'switch_statement body',\n\
  \  'catch_clause body',\n\
  \  'ternary_expression',\n\
  \]\n\
  \if_kinds = ['if_statement']\n\
  \coc_flat_kinds = ['else_clause']\n\
  \# Empty on purpose: arrow/function_expression are STANDALONE units\n\
  \# (fn_kinds) here, so a nest-only entry could never fire (the M1\n\
  \# attack review caught the dead entries). Only absorbed inline fns\n\
  \# (Go func_literal, Python lambda) belong in nest-only.\n\
  \coc_nest_only_kinds = []\n"

scan2 :: String
scan2 =
  "# `??` counts in CC (a real branch) but NOT here: whitepaper p.6\n\
  \# ignores null-coalescing in CoC. `?.` counts in neither (M1 stance,\n\
  \# pinned in tests/sonar_whitepaper.rs).\n\
  \coc_operators = ['&&', '||']\n\
  \coc_jump_kinds = ['continue_statement', 'break_statement']\n\
  \label_kinds = ['statement_identifier']\n\
  \comment_kinds = ['comment']\n\
  \name_style = 'mixed_caps'\n\
  \literal_delims = ['\"', \"'\", '`']\n\
  \call_kinds = ['call_expression']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['member_expression']\n\
  \call_self_words = ['this']\n\
  \call_member_scopes = ['class_body', 'object']\n\
  \owner_kinds = []\n\
  \overloads = null\n\
  \call_import_kinds = []\n\
  \opaque_fields = []\n"

shared :: String
shared =
  "[[slot]]\n\
  \# Slot tables for TypeScript and TSX (plan v2.31 step 7), beside the\n\
  \# contract in slot.rs: the two share one piece, each adding its own —\n\
  \# TypeScript's `<T>x`, TSX's JSX kinds — so the shared piece names no\n\
  \# kind either grammar lacks. Read off the flow probes and checked\n\
  \# against the pinned grammars by the unit legs.\n\
  \expr_kinds = [\n\
  \  'array', 'arrow_function', 'binary_expression', 'call_expression', 'function_expression',\n\
  \  'identifier', 'member_expression', 'new_expression', 'number', 'object', 'parenthesized_expression',\n\
  \  'string', 'template_string', 'subscript_expression', 'ternary_expression',\n\
  \  'this', 'true', 'false', 'null', 'undefined', 'unary_expression', 'yield_expression',\n\
  \  'await_expression', 'as_expression', 'non_null_expression', 'shorthand_property_identifier',\n\
  \]\n\
  \type_kinds = [\n\
  \  'type_annotation', 'predefined_type', 'type_identifier', 'nested_type_identifier', 'generic_type',\n\
  \  'array_type', 'function_type', 'object_type', 'union_type', 'literal_type', 'tuple_type',\n\
  \  'type_arguments', 'type_query',\n\
  \]\n\
  \name_fields = [\n\
  \  ['function_declaration', 'name'], ['generator_function_declaration', 'name'],\n\
  \  ['class_declaration', 'name'], ['method_definition', 'name'], ['variable_declarator', 'name'],\n\
  \  ['required_parameter', 'pattern'], ['optional_parameter', 'pattern'],\n\
  \  ['interface_declaration', 'name'], ['type_alias_declaration', 'name'],\n\
  \  ['public_field_definition', 'name'], ['method_signature', 'name'], ['property_signature', 'name'],\n\
  \]\n\
  \target_fields = [\n\
  \  ['assignment_expression', 'left'], ['augmented_assignment_expression', 'left'],\n\
  \  ['update_expression', 'argument'],\n\
  \]\n\
  \part_fields = [['member_expression', 'property']]\n\
  \part_kinds = ['string_fragment']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'program', 'function_declaration', 'generator_function_declaration', 'class_declaration',\n\
  \  'class_body', 'method_definition', 'method_signature', 'public_field_definition',\n\
  \  'accessibility_modifier', 'interface_declaration', 'interface_body', 'property_signature',\n\
  \  'type_alias_declaration', 'ambient_declaration', 'statement_block',\n\
  \  'switch_body', 'statement_identifier', 'formal_parameters', 'required_parameter',\n\
  \  'optional_parameter', 'rest_pattern', 'object_pattern', 'array_pattern', 'pair_pattern',\n\
  \  'shorthand_property_identifier_pattern', 'variable_declarator', 'arguments', 'pair',\n\
  \  'spread_element', 'property_identifier', 'assignment_expression',\n\
  \  'augmented_assignment_expression', 'update_expression', 'export_statement', 'import_statement',\n\
  \  'import_clause', 'namespace_import', 'named_imports', 'import_specifier', 'comment',\n\
  \]\n"

only :: String
only =
  "[[slot]]\n\
  \expr_kinds = ['type_assertion']\n"

name :: String
name =
  "name = 'typescript'\n"
