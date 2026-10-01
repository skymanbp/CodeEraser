-- | The rust definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_launch.rs,
-- cli/src/merge/slot_launch.rs, cli/src/graph/spec.rs,
-- cli/src/fourclass/kinds.rs at e877f389; from this commit on the core is
-- the authority.
module CE.Lang.Rust where

top :: String
top =
  "name = 'rust'\n\
  \extra = [\n\
  \  'const_item', 'static_item', 'struct_item', 'enum_item', 'trait_item', 'mod_item',\n\
  \]\n\
  \sites = [\n\
  \  {node = 'use_declaration', label = 'use', via = {form = 'use_targets'}},\n\
  \  {node = 'mod_item', label = 'mod_decl', via = {form = 'name_if_no_body'}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \fn_kinds = ['function_item', 'closure_expression']\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['parameters', 'closure_parameters']\n\
  \cc_kinds = [\n\
  \  'if_expression',\n\
  \  'match_arm',\n\
  \  'for_expression',\n\
  \  'while_expression',\n\
  \  'loop_expression',\n\
  \  # `expr?` is an implicit early-return branch (== match Ok/Err);\n\
  \  # rust-code-analysis counts it (M1 cross-check, ban.rs 21 vs 17).\n\
  \  'try_expression',\n\
  \]\n\
  \cc_operators = ['&&', '||']\n\
  \# let_chain joins let_conditions/exprs with anonymous `&&` tokens\n\
  \# (no operator field — AST-probed; M1 attack review finding).\n\
  \chain_kinds = ['let_chain']\n\
  \coc_nesting_kinds = [\n\
  \  'if_expression consequence alternative',\n\
  \  'for_expression body',\n\
  \  'while_expression body',\n\
  \  'loop_expression body',\n\
  \  'match_expression body',\n\
  \]\n\
  \if_kinds = ['if_expression']\n\
  \coc_flat_kinds = ['else_clause']\n\
  \# Empty on purpose: closure_expression is a standalone unit\n\
  \# (fn_kinds), so nest-only could never fire (dead-entry review).\n\
  \coc_nest_only_kinds = []\n\
  \coc_operators = ['&&', '||']\n\
  \# `break 'l` / `continue 'l`: the label child kind is `label`;\n\
  \# a plain `break value` has an expression child, so it won't count.\n\
  \coc_jump_kinds = ['continue_expression', 'break_expression']\n\
  \label_kinds = ['label']\n\
  \comment_kinds = ['line_comment', 'block_comment']\n\
  \name_style = 'snake'\n\
  \# NOT `'`: that is the lifetime/label tick (char_literal is one\n\
  \# token and needs no delimiter piece).\n\
  \literal_delims = ['\"']\n\
  \call_kinds = ['call_expression']\n"

scan2 :: String
scan2 =
  "call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['field_expression', 'scoped_identifier']\n\
  \call_self_words = ['self', 'Self']\n\
  \call_member_scopes = ['impl_item', 'trait_item']\n\
  \owner_kinds = []\n\
  \overloads = null\n\
  \call_import_kinds = ['use_declaration']\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# Slot tables for Python, Rust and Go (plan v2.31 step 7), beside the\n\
  \# contract in slot.rs. Every kind and field here is read off the flow\n\
  \# probes (scripts/tsprobe/snippets/flow.*) and checked against the\n\
  \# pinned grammar by the unit legs; TypeScript and TSX are slot_ts.rs.\n\
  \expr_kinds = [\n\
  \  'array_expression', 'binary_expression', 'boolean_literal', 'call_expression',\n\
  \  'closure_expression', 'field_expression', 'identifier', 'index_expression', 'integer_literal',\n\
  \  'float_literal', 'negative_literal', 'char_literal', 'range_expression', 'reference_expression',\n\
  \  'string_literal', 'struct_expression', 'try_expression', 'tuple_expression',\n\
  \  'type_cast_expression', 'unary_expression', 'macro_invocation', 'scoped_identifier', 'self',\n\
  \  'await_expression', 'unit_expression', 'parenthesized_expression', 'shorthand_field_initializer',\n\
  \  'raw_string_literal',\n\
  \]\n\
  \type_kinds = [\n\
  \  'abstract_type', 'array_type', 'function_type', 'generic_type', 'never_type', 'pointer_type',\n\
  \  'primitive_type', 'reference_type', 'tuple_type', 'type_arguments', 'type_identifier',\n\
  \  'scoped_type_identifier', 'dynamic_type',\n\
  \]\n\
  \name_fields = [\n\
  \  ['function_item', 'name'], ['struct_item', 'name'], ['enum_item', 'name'], ['trait_item', 'name'],\n\
  \  ['const_item', 'name'], ['static_item', 'name'], ['type_item', 'name'], ['parameter', 'pattern'],\n\
  \  ['let_declaration', 'pattern'], ['field_declaration', 'name'],\n\
  \]\n\
  \target_fields = [['assignment_expression', 'left'], ['compound_assignment_expr', 'left']]\n\
  \part_fields = [['field_expression', 'field']]\n\
  \part_kinds = ['string_content']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'source_file', 'function_item', 'struct_item', 'impl_item', 'declaration_list',\n\
  \  'field_declaration', 'field_declaration_list', 'use_declaration', 'visibility_modifier',\n\
  \  'attribute_item', 'attribute', 'parameters', 'parameter', 'self_parameter', 'closure_parameters',\n\
  \  'block', 'async_block', 'arguments', 'assignment_expression', 'compound_assignment_expr',\n\
  \  'field_identifier', 'field_initializer', 'field_initializer_list', 'shorthand_field_identifier',\n\
  \  'let_condition', 'match_block', 'match_pattern', 'captured_pattern', 'field_pattern',\n\
  \  'range_pattern', 'reference_pattern', 'struct_pattern', 'tuple_pattern', 'tuple_struct_pattern',\n\
  \  'mutable_specifier', 'label', 'token_tree', 'line_comment', 'block_comment', 'doc_comment',\n\
  \  'inner_doc_comment_marker', 'outer_doc_comment_marker',\n\
  \]\n"
