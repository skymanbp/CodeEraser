-- | The c definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_c.rs, cli/src/merge/slot_c.rs,
-- cli/src/graph/spec.rs, cli/src/fourclass/kinds.rs at e877f389; from this
-- commit on the core is the authority.
module CE.Lang.C where

name :: String
name =
  "name = 'c'\n"

top :: String
top =
  "extra = [\n\
  \  'preproc_def', 'preproc_function_def', 'struct_specifier', 'union_specifier',\n\
  \  'enum_specifier', 'type_definition', 'declaration', 'class_specifier',\n\
  \  'namespace_definition', 'alias_declaration',\n\
  \]\n\
  \sites = [\n\
  \  # the quoted form loses its quotes, the system form keeps its brackets\n\
  \  {node = 'preproc_include', label = 'include', via = {form = 'field', arg = 'path'}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \# The C / C++ LangSpec tables (plan v2.30 step 2). ONE set of kinds\n\
  \# for both grammars: tree-sitter-cpp is a superset of tree-sitter-c\n\
  \# and spells every construct the two share with the same node kind\n\
  \# (probe transcripts 2026-09-24, four rounds against the pinned\n\
  \# 0.24.2 / 0.23.4 — scripts/tsprobe), so the C++-only kinds below\n\
  \# simply never occur in a C parse. The two tables differ in one\n\
  \# LANGUAGE fact no kind can carry (plan v2.30 step 3): C++ overloads a\n\
  \# name by its parameters and C does not — two same-named C functions\n\
  \# of one scope are one function under two `#if` arms, two C++ ones\n\
  \# may be two functions (LangSpec::overloads). Lives beside\n\
  \# spec_hs.rs for the reason that file does: spec.rs is the contract\n\
  \# alone (RM16).\n\
  \#\n\
  \# The external oracle is lizard (CCN only — no C/C++ cognitive\n\
  \# oracle exists), and every place the table reads differently from it\n\
  \# is a numbered stance in contracts/fixtures/crosscheck/DIVERGENCES.md\n\
  \# (the C / C++ section), never a hidden choice.\n\
  \#\n\
  \# Key probe facts the table stands on:\n\
  \# - a function's NAME sits at the leaf of a declarator chain\n\
  \#   (`function_definition.declarator` → function_declarator →\n\
  \#   pointer_declarator / parenthesized_declarator / … → identifier |\n\
  \#   field_identifier | qualified_identifier | destructor_name |\n\
  \#   operator_name | template_function), and its parameter list hangs\n\
  \#   off the innermost function_declarator — functions.rs walks that\n\
  \#   chain, which is why param_list_kinds stays empty here\n\
  \# - `else` is an `else_clause` holding either a compound_statement or\n\
  \#   the next if_statement (the TypeScript shape, scored by the same\n\
  \#   flat-hybrid rule); `default:` is a case_statement like any case\n\
  \# - `switch_statement` nests; its case_statement rows do not\n\
  \# - C++ `and` / `or` arrive as the `operator` field's text exactly\n\
  \#   like `&&` / `||`; `not` is unary and counts nowhere\n\
  \# - `goto` carries `label: statement_identifier`; C's break /\n\
  \#   continue never carry a label, so they are not listed (an entry\n\
  \#   that can never fire is a dead entry — M1 attack review)\n\
  \# - a lambda_expression absorbs into its host (no fn_kinds entry)\n\
  \#   and raises nesting only — the Go func_literal precedent\n\
  \# - string_literal lexes as `\"` + string_content + `\"`, char_literal\n\
  \#   as `'` + character + `'`, a raw string as `R\"` + `(` +\n\
  \#   raw_string_content + `)` + `\"`: the anonymous quote tokens are\n\
  \#   the delimiter pieces under literal_delims, and the whole node is\n"

scan2 :: String
scan2 =
  "#   ONE token however it is spelled — prefixed, raw, delimited or\n\
  \#   user-defined (dedup/tokens.rs whole_literal, plan v2.30 step 5b;\n\
  \#   a raw literal used to weigh five where a plain one weighs one)\n\
  \# - the callee is the `function` field of call_expression; a member\n\
  \#   callee is a field_expression (`this->m`, `obj.m`) or a\n\
  \#   qualified_identifier (`K::m`); a class body is a\n\
  \#   field_declaration_list; `using ns::name;` binds a name locally\n\
  \# - the preprocessor is invisible to both metrics: `#ifdef` /\n\
  \#   `#elif` are neither branches nor nesting (register D1 — lizard\n\
  \#   counts them), and a macro body is one opaque `preproc_arg` token\n\
  \fn_kinds = ['function_definition']\n\
  \# every definition shape rides declarator::defined instead\n\
  \fn_required_fields = []\n\
  \# the list hangs off the declarator chain — functions.rs reads it\n\
  \# through the chain, nothing here to scan for\n\
  \param_list_kinds = []\n\
  \cc_kinds = [\n\
  \  'if_statement',\n\
  \  'for_statement',\n\
  \  'for_range_loop',\n\
  \  'while_statement',\n\
  \  'do_statement',\n\
  \  # every case incl. `default:` (register D2: lizard counts the\n\
  \  # `case` keyword only; the Rust match_arm / Haskell alternative\n\
  \  # precedent — a total switch's default is a real path)\n\
  \  'case_statement',\n\
  \  'conditional_expression',\n\
  \  'catch_clause',\n\
  \]\n\
  \# `and` / `or` are the C++ alternative tokens (register D19)\n\
  \cc_operators = ['&&', '||', 'and', 'or']\n\
  \chain_kinds = []\n"

scan3 :: String
scan3 =
  "coc_nesting_kinds = [\n\
  \  'if_statement consequence alternative',\n\
  \  'for_statement body',\n\
  \  'for_range_loop body',\n\
  \  'while_statement body',\n\
  \  'do_statement body',\n\
  \  'switch_statement body',\n\
  \  # the ternary nests like TypeScript's (register D4)\n\
  \  'conditional_expression',\n\
  \  'catch_clause body',\n\
  \]\n\
  \# the if's `alternative` is an else_clause, scored by the flat rule\n\
  \if_kinds = ['if_statement']\n\
  \coc_flat_kinds = ['else_clause']\n\
  \coc_nest_only_kinds = ['lambda_expression']\n\
  \coc_operators = ['&&', '||', 'and', 'or']\n\
  \coc_jump_kinds = ['goto_statement']\n\
  \label_kinds = ['statement_identifier']\n\
  \comment_kinds = ['comment']\n\
  \# no single convention holds across C code bases (register D22)\n\
  \name_style = 'any'\n\
  \literal_delims = ['\"', \"'\", 'character', 'R\"']\n\
  \call_kinds = ['call_expression']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['identifier']\n\
  \call_member_kinds = ['field_expression', 'qualified_identifier']\n\
  \call_self_words = ['this']\n\
  \# a class / struct / union body says so itself (the TypeScript\n\
  \# class_body precedent); a namespace body is a declaration_list\n\
  \# and is NOT a member scope — bare names resolve inside it\n\
  \call_member_scopes = ['field_declaration_list']\n\
  \# the declarator spells a member's owner (scan/declarator.rs)\n\
  \owner_kinds = []\n\
  \# overloads: none in C; C++ states its own after this table\n\
  \call_import_kinds = ['using_declaration']\n\
  \# `#if defined(A) && B` / `#elif`: the condition is an expression\n\
  \# to the parser and compile-time text to every metric (register\n\
  \# D1 — fmt's is_big_endian read 2 for the `&&` in its `#elif`\n\
  \# before this line existed); `#ifdef` names an identifier, not an\n\
  \# expression, and needs no row\n\
  \opaque_fields = [['preproc_if', 'condition'], ['preproc_elif', 'condition']]\n"

shared :: String
shared =
  "[[slot]]\n\
  \# Slot tables for C and C++ (plan v2.31 step 7), beside the contract in\n\
  \# slot.rs: C++ reads the shared piece and its own, C reads the shared\n\
  \# piece and the one kind only its grammar has — so the shared piece\n\
  \# names no kind either grammar lacks.\n\
  \expr_kinds = [\n\
  \  'binary_expression', 'call_expression', 'comma_expression', 'conditional_expression',\n\
  \  'field_expression', 'identifier', 'number_literal', 'char_literal', 'parenthesized_expression',\n\
  \  'pointer_expression', 'string_literal', 'escape_sequence',\n\
  \  'concatenated_string', 'subscript_expression', 'true', 'false', 'null', 'sizeof_expression',\n\
  \  'cast_expression', 'unary_expression',\n\
  \]\n\
  \type_kinds = [\n\
  \  'primitive_type', 'type_identifier', 'sized_type_specifier', 'struct_specifier',\n\
  \  'type_descriptor',\n\
  \]\n\
  \name_fields = [\n\
  \  ['function_declarator', 'declarator'], ['init_declarator', 'declarator'],\n\
  \  ['parameter_declaration', 'declarator'], ['pointer_declarator', 'declarator'],\n\
  \  ['array_declarator', 'declarator'], ['field_declaration', 'declarator'],\n\
  \]\n\
  \target_fields = [['assignment_expression', 'left'], ['update_expression', 'argument']]\n\
  \part_fields = [['field_expression', 'field']]\n\
  \part_kinds = ['string_content']\n\
  \helper_lines = 2\n\
  \other_kinds = [\n\
  \  'translation_unit', 'function_definition', 'function_declarator', 'parameter_list',\n\
  \  'parameter_declaration', 'init_declarator', 'pointer_declarator',\n\
  \  'array_declarator', 'parenthesized_declarator', 'initializer_list', 'field_declaration',\n\
  \  'field_declaration_list', 'field_identifier', 'compound_statement', 'argument_list',\n\
  \  'assignment_expression', 'update_expression', 'storage_class_specifier', 'type_qualifier',\n\
  \  'attribute', 'attribute_declaration', 'attribute_specifier', 'gnu_asm_expression',\n\
  \  'gnu_asm_qualifier', 'statement_identifier', 'preproc_include', 'preproc_defined',\n\
  \  'preproc_elif', 'preproc_else', 'system_lib_string', 'comment',\n\
  \]\n"

only :: String
only =
  "[[slot]]\n\
  \other_kinds = ['variadic_parameter']\n"
