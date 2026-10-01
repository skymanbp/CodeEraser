-- | The go definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_launch.rs,
-- cli/src/merge/slot_launch.rs, cli/src/graph/spec.rs,
-- cli/src/fourclass/kinds.rs at e877f389; from this commit on the core is
-- the authority.
module CE.Lang.Go where

top :: String
top =
  "name = 'go'\n\
  \extra = [\n\
  \  'type_spec', 'type_alias', 'const_spec', 'var_spec',\n\
  \]\n\
  \sites = [\n\
  \  {node = 'import_spec', label = 'import', via = {form = 'field', arg = 'path'}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \# gocyclo attributes func-literal branches to the enclosing decl,\n\
  \# so func_literal is absorbed, not standalone.\n\
  \fn_kinds = ['function_declaration', 'method_declaration']\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['parameter_list']\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'for_statement',\n\
  \  # default_case is NOT counted: gocyclo v0.6.0 complexity.go\n\
  \  # skips CaseClause/CommClause with a nil list (\"ignore default\n\
  \  # case\"), and the whitepaper p.5 margin (getWords CC=4) agrees.\n\
  \  # Found by the M1 attack review — no default: in the fixtures,\n\
  \  # so 52/52 was silent on this axis.\n\
  \  'expression_case',\n\
  \  'type_case',\n\
  \  'communication_case',\n\
  \]\n\
  \cc_operators = ['&&', '||']\n\
  \chain_kinds = []\n\
  \# a switch's cases are unnamed children, named by their kinds;\n\
  \# select has no header and nests whole\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'expression_switch_statement expression_case default_case',\n\
  \  'type_switch_statement type_case default_case',\n\
  \  'select_statement',\n\
  \]\n\
  \if_kinds = ['if_statement']\n\
  \# Go has no else node kind: else lives in the if's `alternative`\n\
  \# field, which the emitter states as the if's alternative class\n\
  \# and the core scores (CE.Scan.Complexity).\n\
  \coc_flat_kinds = []\n\
  \coc_nest_only_kinds = ['func_literal']\n\
  \coc_operators = ['&&', '||']\n\
  \# goto always carries a label_name, so it always counts.\n\
  \coc_jump_kinds = ['continue_statement', 'break_statement', 'goto_statement']\n\
  \label_kinds = ['label_name']\n\
  \comment_kinds = ['comment']\n\
  \name_style = 'mixed_caps'\n"

scan2 :: String
scan2 =
  "# `'` is Go's rune delimiter but rune_literal is a single token.\n\
  \literal_delims = ['\"', '`']\n\
  \call_kinds = ['call_expression']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['selector_expression']\n\
  \call_self_words = []\n\
  \# a Go method is declared at the top level and its name carries\n\
  \# the receiver type, so a bare name cannot reach one at all\n\
  \call_member_scopes = []\n\
  \owner_kinds = []\n\
  \overloads = null\n\
  \call_import_kinds = []\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# Slot tables for Python, Rust and Go (plan v2.31 step 7), beside the\n\
  \# contract in slot.rs. Every kind and field here is read off the flow\n\
  \# probes (scripts/tsprobe/snippets/flow.*) and checked against the\n\
  \# pinned grammar by the unit legs; TypeScript and TSX are slot_ts.rs.\n\
  \expr_kinds = [\n\
  \  'binary_expression', 'call_expression', 'composite_literal', 'func_literal', 'identifier',\n\
  \  'index_expression', 'int_literal', 'float_literal', 'rune_literal', 'interpreted_string_literal',\n\
  \  'raw_string_literal', 'selector_expression', 'true',\n\
  \  'false', 'nil', 'iota', 'unary_expression', 'expression_list', 'parenthesized_expression',\n\
  \  'slice_expression', 'type_assertion_expression',\n\
  \]\n\
  \type_kinds = [\n\
  \  'channel_type', 'function_type', 'interface_type', 'map_type', 'pointer_type', 'slice_type',\n\
  \  'struct_type', 'type_identifier', 'array_type', 'qualified_type', 'generic_type',\n\
  \]\n\
  \name_fields = [\n\
  \  ['function_declaration', 'name'], ['method_declaration', 'name'],\n\
  \  ['parameter_declaration', 'name'], ['variadic_parameter_declaration', 'name'],\n\
  \  ['var_spec', 'name'], ['const_spec', 'name'], ['type_spec', 'name'],\n\
  \  ['field_declaration', 'name'],\n\
  \]\n\
  \target_fields = [['assignment_statement', 'left'], ['inc_statement', ''], ['dec_statement', '']]\n\
  \target_lists = ['expression_list']\n\
  \part_fields = [['selector_expression', 'field']]\n\
  \part_kinds = ['interpreted_string_literal_content', 'raw_string_literal_content']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'source_file', 'package_clause', 'package_identifier', 'import_declaration', 'import_spec',\n\
  \  'import_spec_list', 'function_declaration', 'method_declaration', 'type_declaration', 'type_spec',\n\
  \  'var_spec', 'var_spec_list', 'const_spec', 'parameter_list', 'parameter_declaration',\n\
  \  'variadic_parameter_declaration', 'field_declaration', 'field_declaration_list',\n\
  \  'field_identifier', 'block', 'statement_list', 'argument_list', 'literal_value',\n\
  \  'literal_element', 'keyed_element', 'for_clause', 'range_clause', 'inc_statement',\n\
  \  'dec_statement', 'send_statement', 'label_name', 'comment',\n\
  \]\n"
