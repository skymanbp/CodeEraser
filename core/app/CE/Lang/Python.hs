-- | The python definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_launch.rs,
-- cli/src/merge/slot_launch.rs, cli/src/graph/spec.rs,
-- cli/src/fourclass/kinds.rs at e877f389; from this commit on the core is
-- the authority.
module CE.Lang.Python where

top :: String
top =
  "name = 'python'\n\
  \extra = [\n\
  \  'class_definition',\n\
  \]\n\
  \docstring_hosts = [\n\
  \  'module', 'function_definition', 'class_definition',\n\
  \]\n\
  \sites = [\n\
  \  {node = 'import_statement', label = 'import', via = {form = 'each_import_target'}},\n\
  \  {node = 'import_from_statement', label = 'import_from', via = {form = 'field', arg = 'module_name'}},\n\
  \  # `from __future__ import` has no module field: the site names it\n\
  \  {node = 'future_import_statement', label = 'import_from', via = {form = 'literal', arg = '__future__'}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \fn_kinds = ['function_definition']\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['parameters']\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'elif_clause',\n\
  \  'conditional_expression',\n\
  \  'for_statement',\n\
  \  'while_statement',\n\
  \  'except_clause',\n\
  \  'case_clause',\n\
  \  'assert_statement',\n\
  \  # comprehension clauses are real branch paths: lizard and radon\n\
  \  # both count them (M1 cross-check finding). CoC deliberately does\n\
  \  # NOT count them (declarative expression, no nesting cost).\n\
  \  'for_in_clause',\n\
  \  'if_clause',\n\
  \]\n\
  \cc_operators = ['and', 'or']\n\
  \chain_kinds = []\n\
  \# the except's block is an unnamed child, named by its kind\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body alternative',\n\
  \  'while_statement body alternative',\n\
  \  'except_clause block',\n\
  \  'conditional_expression',\n\
  \  'match_statement body',\n\
  \]\n\
  \if_kinds = ['if_statement']\n\
  \# an elif's condition sits at the chain's level, like an else-if\n\
  \coc_flat_kinds = ['elif_clause consequence', 'else_clause']\n\
  \coc_nest_only_kinds = ['lambda']\n\
  \coc_operators = ['and', 'or']\n\
  \# Python has no labeled jumps.\n\
  \coc_jump_kinds = []\n\
  \label_kinds = []\n\
  \comment_kinds = ['comment']\n\
  \name_style = 'snake'\n"

scan2 :: String
scan2 =
  "# Python quotes surface as named string_start/string_end kinds.\n\
  \literal_delims = []\n\
  \call_kinds = ['call']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['attribute']\n\
  \call_self_words = ['self', 'cls']\n\
  \call_member_scopes = ['class_definition']\n\
  \owner_kinds = []\n\
  \overloads = null\n\
  \call_import_kinds = ['import_from_statement', 'import_statement']\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# Slot tables for Python, Rust and Go (plan v2.31 step 7), beside the\n\
  \# contract in slot.rs. Every kind and field here is read off the flow\n\
  \# probes (scripts/tsprobe/snippets/flow.*) and checked against the\n\
  \# pinned grammar by the unit legs; TypeScript and TSX are slot_ts.rs.\n\
  \expr_kinds = [\n\
  \  'attribute', 'binary_operator', 'boolean_operator', 'call', 'comparison_operator',\n\
  \  'conditional_expression', 'dictionary_comprehension', 'expression_list', 'generator_expression',\n\
  \  'identifier', 'integer', 'float', 'lambda', 'list_comprehension', 'named_expression', 'none',\n\
  \  'true', 'false', 'parenthesized_expression', 'set_comprehension', 'string',\n\
  \  'interpolation', 'concatenated_string', 'subscript', 'slice', 'tuple', 'list', 'dictionary', 'set',\n\
  \  'unary_operator', 'await',\n\
  \]\n\
  \type_kinds = ['type']\n\
  \name_fields = [\n\
  \  ['function_definition', 'name'], ['class_definition', 'name'], ['default_parameter', 'name'],\n\
  \  ['typed_default_parameter', 'name'],\n\
  \]\n\
  \target_fields = [['assignment', 'left'], ['augmented_assignment', 'left']]\n\
  \target_lists = ['pattern_list', 'tuple_pattern', 'list_pattern']\n\
  \part_fields = [['attribute', 'attribute']]\n\
  \part_kinds = ['string_content']\n\
  \helper_lines = 1\n\
  \other_kinds = [\n\
  \  'module', 'function_definition', 'class_definition', 'decorated_definition', 'decorator', 'block',\n\
  \  'parameters', 'lambda_parameters', 'default_parameter', 'typed_parameter', 'typed_default_parameter',\n\
  \  'list_splat_pattern', 'dictionary_splat_pattern', 'keyword_separator', 'positional_separator',\n\
  \  'argument_list', 'keyword_argument', 'pair', 'assignment', 'augmented_assignment', 'pattern_list',\n\
  \  'tuple_pattern', 'for_in_clause', 'if_clause', 'with_clause', 'with_item', 'as_pattern',\n\
  \  'as_pattern_target', 'case_pattern', 'class_pattern', 'dict_pattern', 'list_pattern',\n\
  \  'splat_pattern', 'keyword_pattern', 'union_pattern', 'dotted_name', 'import_statement',\n\
  \  'import_from_statement', 'string_start', 'string_end', 'comment',\n\
  \]\n"
