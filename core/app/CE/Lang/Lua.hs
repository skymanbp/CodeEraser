-- | The lua definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_lua.rs, cli/src/merge/slot_lua.rs,
-- cli/src/graph/spec/calls.rs at e877f389; from this commit on the core is
-- the authority.
module CE.Lang.Lua where

top :: String
top =
  "name = 'lua'\n\
  \# The calls that open a site: a module by `require`, a file by\n\
  \# `dofile` / `loadfile`.\n\
  \calls = [\n\
  \  {label = 'require', callees = ['require'], package = '', unquoted = null},\n\
  \  {label = 'load', callees = ['dofile', 'loadfile'], package = '', unquoted = null},\n\
  \]\n\
  \# The protected-call wrappers, each with the count of arguments\n\
  \# before the protected function's own.\n\
  \protected = [['pcall', 1], ['xpcall', 2]]\n"

scan :: String
scan =
  "[scan]\n\
  \# The Lua LangSpec table (plan v2.30 step 4; design booklet §4 the Lua\n\
  \# column). Lives beside spec_java.rs for the reason every table does:\n\
  \# spec.rs is the contract alone (RM16).\n\
  \#\n\
  \# The external oracle is lizard (CCN; its reader list includes Lua) —\n\
  \# no Lua cognitive oracle exists — and every place the table reads\n\
  \# differently from it is a numbered stance in\n\
  \# contracts/fixtures/crosscheck/DIVERGENCES.md (the Lua section),\n\
  \# never a hidden choice.\n\
  \#\n\
  \# Key probe facts the table stands on (tree-sitter-lua 0.5.0, the\n\
  \# scripts/tsprobe transcripts):\n\
  \# - `function f()`, `local function f()`, `function M.f()` and\n\
  \#   `function M:f()` are one kind, `function_declaration`, whose\n\
  \#   `name` is an identifier, a dot_index_expression or a\n\
  \#   method_index_expression; `local` is an anonymous child token\n\
  \# - every other function is an anonymous `function_definition` — a\n\
  \#   table field's value, an assignment's value (`local f = function`,\n\
  \#   `M.f = function`), an argument. Its name, when it has one, is the\n\
  \#   binding around it (scan/binding.rs), and an unbound one is a unit\n\
  \#   of its own named `(anonymous)` (register D3: Lua's function values\n\
  \#   are its declaration form, so they never fold into their host)\n\
  \# - `if_statement` hangs its `elseif_statement`s and its\n\
  \#   `else_statement` on one repeated `alternative` field; an elseif\n\
  \#   carries its own condition, which sits at the chain's level (the\n\
  \#   Python elif reading)\n\
  \# - `goto` carries its label as an unfielded identifier child; break\n\
  \#   never carries one (register D5)\n\
  \# - comments are one compound `comment` kind (start / content / end\n\
  \#   children), skipped whole; a string lexes as a start token, a\n\
  \#   string_content and an end token, and the long-bracket tokens are\n\
  \#   `[[` / `]]` at every level (`[==[` included)\n\
  \# - a call is `function_call{name, arguments}`: the callee is its\n\
  \#   `name` field, and a member callee (`M.f()`, `obj:m()`) is a\n\
  \#   dot_index_expression or method_index_expression holding the object\n\
  \#   and the member; `self` is the method's own table\n\
  \fn_kinds = ['function_declaration', 'function_definition']\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['parameters']\n"

scan2 :: String
scan2 =
  "# an elseif is a decision of its own (lizard counts the keyword)\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'elseif_statement',\n\
  \  'for_statement',\n\
  \  'while_statement',\n\
  \  'repeat_statement',\n\
  \]\n\
  \cc_operators = ['and', 'or']\n\
  \chain_kinds = []\n\
  \# a repeat's `until` condition is its header (register D31)\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'while_statement body',\n\
  \  'repeat_statement body',\n\
  \]\n\
  \# the if's alternatives are elseif / else nodes, scored flat\n\
  \if_kinds = ['if_statement']\n\
  \coc_flat_kinds = ['elseif_statement consequence', 'else_statement']\n\
  \# a function value is a unit of its own (register D3)\n\
  \coc_nest_only_kinds = []\n\
  \coc_operators = ['and', 'or']\n\
  \coc_jump_kinds = ['goto_statement']\n\
  \label_kinds = ['identifier']\n\
  \comment_kinds = ['comment']\n\
  \# no convention holds across Lua code bases (register D22)\n\
  \name_style = 'any'\n\
  \literal_delims = ['\"', \"'\", '[[', ']]']\n\
  \call_kinds = ['function_call']\n\
  \call_fields = ['name', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['dot_index_expression', 'method_index_expression']\n\
  \call_self_words = ['self']\n\
  \# a table constructor's fields answer to the table, never to a\n\
  \# bare name (`{ f = function() end }` is reached as `t.f`)\n\
  \call_member_scopes = ['table_constructor']\n\
  \# a member's owner is the table its name spells (`M` of `M.f`,\n\
  \# scan/binding.rs), not a type declaration around it\n\
  \owner_kinds = []\n\
  \overloads = null\n"

scan3 :: String
scan3 =
  "# a local binding is Lua's import: `local helper = other.helper`\n\
  \# inside a body shadows a same-named callable of the file for the\n\
  \# calls that body makes (register D21, closed in plan v2.30 step\n\
  \# 5b); one bound to a function value is a unit the index seats and\n\
  \# shadows nothing (scan/calls.rs shadowed)\n\
  \call_import_kinds = ['variable_declaration']\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# The Lua slot table (plan v2.31 step 7), beside the contract in\n\
  \# slot.rs: Lua has no type position.\n\
  \expr_kinds = [\n\
  \  'binary_expression', 'bracket_index_expression', 'dot_index_expression',\n\
  \  'method_index_expression', 'function_call', 'function_definition', 'identifier', 'number',\n\
  \  'string', 'true', 'false', 'nil', 'vararg_expression', 'table_constructor',\n\
  \  'parenthesized_expression', 'unary_expression',\n\
  \]\n\
  \name_fields = [['function_declaration', 'name']]\n\
  \target_fields = [['variable_list', 'name']]\n\
  \part_fields = [['dot_index_expression', 'field'], ['method_index_expression', 'method']]\n\
  \part_kinds = ['string_content']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'chunk', 'function_declaration', 'parameters', 'block', 'arguments', 'assignment_statement',\n\
  \  'variable_list', 'expression_list', 'attribute', 'field', 'for_generic_clause',\n\
  \  'for_numeric_clause', 'comment', 'comment_content',\n\
  \]\n"
