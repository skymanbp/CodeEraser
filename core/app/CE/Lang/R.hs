-- | The r definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_r.rs, cli/src/merge/slot_r.rs,
-- cli/src/graph/spec.rs, cli/src/graph/spec/calls.rs at e877f389; from
-- this commit on the core is the authority.
module CE.Lang.R where

top :: String
top =
  "name = 'r'\n\
  \sites = [\n\
  \  # `pkg::name` names its package whatever the right side selects\n\
  \  {node = 'namespace_operator', label = 'library', via = {form = 'field', arg = 'lhs'}},\n\
  \]\n\
  \# The calls that open a site: a file by `source`, a package by the\n\
  \# `library` family; `library` and `require` read a bare name unless\n\
  \# the call passes `character.only`.\n\
  \calls = [\n\
  \  {label = 'source', callees = ['source', 'sys.source'], package = 'base', unquoted = null},\n\
  \  {label = 'library', callees = ['library', 'require'], package = 'base', unquoted = 'character.only'},\n\
  \  {label = 'library', callees = ['requireNamespace', 'loadNamespace'], package = 'base', unquoted = null},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \# The R LangSpec table (plan v2.30 step 4; design booklet §4 the R\n\
  \# column). Lives beside spec_lua.rs for the reason every table does:\n\
  \# spec.rs is the contract alone (RM16).\n\
  \#\n\
  \# The external oracle is lizard (CCN): its R reader,\n\
  \# `lizard_languages/r.py`, was missed when the booklet pinned the\n\
  \# oracles and found by the step-4 crosscheck (register D0 now names\n\
  \# CoC alone). No R cognitive oracle exists, so CoC's mapping is the\n\
  \# repository's own stance and the batteries its only executor. Every\n\
  \# place the table reads differently from lizard is attributed in\n\
  \# contracts/fixtures/crosscheck/DIVERGENCES.md (the R section), never\n\
  \# a hidden choice.\n\
  \#\n\
  \# Key probe facts the table stands on (tree-sitter-r 1.3.0, the\n\
  \# scripts/tsprobe transcripts):\n\
  \# - every function is an anonymous `function_definition` (`function`\n\
  \#   and `\\` alike); its `name` field holds the KEYWORD token, never a\n\
  \#   name, and the name is the assignment around it\n\
  \#   (`f <- function(x)`, `(function(x) x) -> f`, scan/binding.rs) — an\n\
  \#   unbound one is a unit of its own named `(anonymous)` (register D3)\n\
  \# - `else` has no node: `if_statement.alternative` IS the else\n\
  \#   expression or the next if (the Go and Java shape), so the if_kinds\n\
  \#   rule scores a plain else, and a braced one alike\n\
  \# - `&&` / `||` short-circuit; the vectorised `&` / `|` do not and\n\
  \#   count nowhere (register D9); `repeat` is a loop; `next` / `break`\n\
  \#   carry no label, and R has no goto (register D5)\n\
  \# - `switch()`, `ifelse()` and `tryCatch()` are calls: their control\n\
  \#   flow is invisible to the syntax and scores nothing (register D8)\n\
  \# - comments are one `comment` kind (roxygen `#'` included); a string\n\
  \#   lexes as NAMED `string_open` / `string_content` / `string_close`\n\
  \#   pieces, a raw string (`r\"(…)\"`) too; `1L` and `3i` are composite\n\
  \#   nodes whose only leaf is the suffix (register D10)\n\
  \# - a call is `call{function, arguments}`; a member callee (`x$f()`,\n\
  \#   `pkg::f()`) is an extract_operator or a namespace_operator holding\n\
  \#   the object and the member; R has no self word\n\
  \fn_kinds = ['function_definition']\n\
  \fn_required_fields = []\n\
  \param_list_kinds = ['parameters']\n"

scan2 :: String
scan2 =
  "cc_kinds = [\n\
  \  'if_statement',\n\
  \  'for_statement',\n\
  \  'while_statement',\n\
  \  'repeat_statement',\n\
  \]\n\
  \cc_operators = ['&&', '||']\n\
  \chain_kinds = []\n\
  \# a for's sequence and a while's condition are headers (register\n\
  \# D31); a repeat is all body\n\
  \coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'while_statement body',\n\
  \  'repeat_statement body',\n\
  \]\n\
  \if_kinds = ['if_statement']\n\
  \coc_flat_kinds = []\n\
  \# a function value is a unit of its own (register D3)\n\
  \coc_nest_only_kinds = []\n\
  \coc_operators = ['&&', '||']\n\
  \coc_jump_kinds = []\n\
  \label_kinds = []\n\
  \comment_kinds = ['comment']\n\
  \# base R, the tidyverse and Bioconductor each write their own\n\
  \# (register D22)\n\
  \name_style = 'any'\n\
  \# the quote pieces are named kinds here, not anonymous tokens\n\
  \literal_delims = ['string_open', 'string_close']\n\
  \call_kinds = ['call']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['extract_operator', 'namespace_operator']\n\
  \call_self_words = []\n\
  \call_member_scopes = []\n\
  \# a member's owner is the object its name spells (`x` of `x$f`,\n\
  \# scan/binding.rs)\n\
  \owner_kinds = []\n\
  \overloads = null\n\
  \call_import_kinds = []\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# The R slot table (plan v2.31 step 7), beside the contract in\n\
  \# slot.rs: R has no type position.\n\
  \expr_kinds = [\n\
  \  'binary_operator', 'unary_operator', 'call', 'extract_operator', 'namespace_operator', 'subset',\n\
  \  'subset2', 'float', 'integer', 'string', 'true', 'false', 'null', 'identifier',\n\
  \  'function_definition', 'parenthesized_expression',\n\
  \]\n\
  \name_fields = [['parameter', 'name']]\n\
  \target_ops = [\n\
  \  ['binary_operator', 'lhs', '<-'], ['binary_operator', 'lhs', '<<-'],\n\
  \  ['binary_operator', 'lhs', '='], ['binary_operator', 'rhs', '->'],\n\
  \  ['binary_operator', 'rhs', '->>'],\n\
  \]\n\
  \part_fields = [['extract_operator', 'rhs']]\n\
  \part_kinds = ['string_content']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'program', 'braced_expression', 'parameters', 'parameter', 'dots', 'arguments', 'argument',\n\
  \  'comma', 'string_open', 'string_close', 'comment',\n\
  \]\n"
