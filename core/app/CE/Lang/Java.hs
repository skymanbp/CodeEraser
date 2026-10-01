-- | The java definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_java.rs, cli/src/merge/slot_java.rs,
-- cli/src/graph/spec.rs, cli/src/fourclass/kinds.rs at e877f389; from this
-- commit on the core is the authority.
module CE.Lang.Java where

top :: String
top =
  "name = 'java'\n\
  \extra = [\n\
  \  'class_declaration', 'interface_declaration', 'enum_declaration', 'record_declaration',\n\
  \  'annotation_type_declaration', 'field_declaration', 'constant_declaration',\n\
  \]\n\
  \sites = [\n\
  \  # the star form under its own label, listed first\n\
  \  {node = 'import_declaration', label = 'import_star', via = {form = 'first_named', arg = {star = true}}},\n\
  \  {node = 'import_declaration', label = 'import', via = {form = 'first_named', arg = {star = false}}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \# The Java LangSpec table (plan v2.30 step 3; design booklet §4 the\n\
  \# Java column). Lives beside spec_launch.rs for the reason every table\n\
  \# does: spec.rs is the contract alone (RM16).\n\
  \#\n\
  \# The external oracles are lizard (CCN) and PMD's CognitiveComplexity\n\
  \# rule — Java is the first new language whose cognitive complexity an\n\
  \# independent implementation can check — and every place the table\n\
  \# reads differently from either is a numbered stance in\n\
  \# contracts/fixtures/crosscheck/DIVERGENCES.md (the Java section),\n\
  \# never a hidden choice.\n\
  \#\n\
  \# Key probe facts the table stands on (tree-sitter-java 0.23.5, the\n\
  \# scripts/tsprobe transcripts of Probe.java / round2.java and a third\n\
  \# sample of enum, interface, annotation and qualified-name shapes):\n\
  \# - a method, a constructor and a record's compact constructor carry\n\
  \#   their name in `name`; an abstract, interface or native method is\n\
  \#   the same kind with no `body` (the fn_required_fields gate)\n\
  \# - `else` has no node: `if_statement.alternative` IS the else body or\n\
  \#   the next if (the Go shape), and a single-statement else is any\n\
  \#   statement there — the if_kinds rule scores all of them\n\
  \# - one `switch_expression` spells both the statement and the arrow\n\
  \#   switch; each `case` / `default` label is a `switch_label` (default\n\
  \#   included, register D2)\n\
  \# - the ternary is `ternary_expression` (it nests, register D4); a\n\
  \#   lambda_expression absorbs into its host and raises nesting only\n\
  \#   (register D3), while an anonymous or local class's methods are\n\
  \#   units of their own\n\
  \# - `break L` / `continue L` carry the label as a bare `identifier`\n\
  \#   child (no field): a plain `break;` has none (register D5)\n\
  \# - comments are `line_comment` and `block_comment` (Javadoc is a\n\
  \#   block comment); strings lex as `\"` + string_fragment + `\"`, a text\n\
  \#   block as `\"\"\"` + multiline_string_fragment + `\"\"\"`, and a char\n\
  \#   literal as one `character_literal` leaf\n\
  \# - a call is `method_invocation{object?, name, arguments}`: the\n\
  \#   receiver is the call's own field, not a member node inside the\n\
  \#   callee (call_fields). `this.m()` is the caller's own object;\n\
  \#   `super.m()` is NOT — it names the superclass's `m`, and an\n\
  \#   override calling `super.m()` is Java's commonest delegation, never\n\
  \#   a recursion. `K.m()` reaches the caller's own class by its name\n\
  \#   (the owner tail, scan/calls.rs)\n\
  \# - a method is always a member: of a class, interface, enum, record\n"

scan2 :: String
scan2 =
  "#   (a `class_body`) or annotation body, so no Java callable is ever\n\
  \#   bare-reachable across types — the owner road is the only one\n\
  \fn_kinds = [\n\
  \  'method_declaration',\n\
  \  'constructor_declaration',\n\
  \  'compact_constructor_declaration',\n\
  \]\n\
  \fn_required_fields = [['method_declaration', 'body']]\n\
  \param_list_kinds = ['formal_parameters']\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'for_statement',\n\
  \  'enhanced_for_statement',\n\
  \  'while_statement',\n\
  \  'do_statement',\n\
  \  # `case` and `default` alike (register D2: lizard counts the\n\
  \  # `case` keyword only)\n\
  \  'switch_label',\n\
  \  'ternary_expression',\n\
  \  'catch_clause',\n\
  \]\n\
  \cc_operators = ['&&', '||']\n\
  \chain_kinds = []\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'enhanced_for_statement body',\n\
  \  'while_statement body',\n\
  \  'do_statement body',\n\
  \  'switch_expression body',\n\
  \  'ternary_expression',\n\
  \  'catch_clause body',\n\
  \]\n\
  \if_kinds = ['if_statement']\n\
  \coc_flat_kinds = []\n\
  \coc_nest_only_kinds = ['lambda_expression']\n\
  \coc_operators = ['&&', '||']\n\
  \coc_jump_kinds = ['break_statement', 'continue_statement']\n\
  \label_kinds = ['identifier']\n\
  \comment_kinds = ['line_comment', 'block_comment']\n"

scan3 :: String
scan3 =
  "# the platform's own naming conventions (register D22)\n\
  \name_style = 'mixed_caps'\n\
  \literal_delims = ['\"', '\"\"\"']\n\
  \call_kinds = ['method_invocation']\n\
  \call_fields = ['name', 'object']\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = []\n\
  \# never `super`: super.m() names the superclass's m (module doc)\n\
  \call_self_words = ['this']\n\
  \# an enum's methods sit in `enum_body_declarations` inside the\n\
  \# `enum_body`, which the grandparent read reaches\n\
  \call_member_scopes = [\n\
  \  'class_body',\n\
  \  'interface_body',\n\
  \  'enum_body',\n\
  \  'annotation_type_body',\n\
  \]\n\
  \owner_kinds = [\n\
  \  'class_declaration',\n\
  \  'interface_declaration',\n\
  \  'enum_declaration',\n\
  \  'record_declaration',\n\
  \  'annotation_type_declaration',\n\
  \]\n\
  \overloads = {optional = [], variadic = ['spread_parameter'], ignored = ['receiver_parameter'], spread = [], unreachable = ['constructor_declaration', 'compact_constructor_declaration']}\n\
  \# Java has no import below the file header\n\
  \call_import_kinds = []\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# The Java slot table (plan v2.31 step 7), beside the contract in\n\
  \# slot.rs.\n\
  \expr_kinds = [\n\
  \  'array_access', 'array_creation_expression', 'array_initializer', 'binary_expression',\n\
  \  'field_access', 'identifier', 'instanceof_expression', 'lambda_expression', 'method_invocation',\n\
  \  'null_literal', 'object_creation_expression', 'parenthesized_expression', 'string_literal',\n\
  \  'ternary_expression', 'this', 'true', 'false', 'unary_expression',\n\
  \  'decimal_integer_literal', 'decimal_floating_point_literal', 'hex_integer_literal',\n\
  \  'character_literal', 'cast_expression', 'method_reference', 'class_literal',\n\
  \]\n\
  \type_kinds = [\n\
  \  'array_type', 'boolean_type', 'generic_type', 'integral_type', 'floating_point_type',\n\
  \  'type_arguments', 'type_identifier', 'scoped_type_identifier', 'void_type', 'catch_type',\n\
  \]\n\
  \name_fields = [\n\
  \  ['method_declaration', 'name'], ['class_declaration', 'name'], ['constructor_declaration', 'name'],\n\
  \  ['record_declaration', 'name'], ['interface_declaration', 'name'], ['enum_declaration', 'name'],\n\
  \  ['formal_parameter', 'name'], ['catch_formal_parameter', 'name'], ['variable_declarator', 'name'],\n\
  \]\n\
  \target_fields = [['assignment_expression', 'left'], ['update_expression', '']]\n\
  \part_fields = [['field_access', 'field'], ['method_invocation', 'name']]\n\
  \part_kinds = ['string_fragment']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'program', 'package_declaration', 'import_declaration', 'scoped_identifier', 'class_declaration',\n\
  \  'class_body', 'record_declaration', 'method_declaration', 'constructor_declaration',\n\
  \  'compact_constructor_declaration', 'constructor_body', 'field_declaration', 'modifiers',\n\
  \  'marker_annotation', 'annotation', 'formal_parameters', 'formal_parameter',\n\
  \  'receiver_parameter', 'spread_parameter', 'catch_formal_parameter', 'variable_declarator',\n\
  \  'dimensions', 'dimensions_expr', 'block', 'argument_list', 'assignment_expression',\n\
  \  'update_expression', 'resource_specification', 'switch_block', 'switch_label', 'guard',\n\
  \  'pattern', 'type_pattern', 'record_pattern', 'record_pattern_body', 'record_pattern_component',\n\
  \  'line_comment', 'block_comment',\n\
  \]\n"
