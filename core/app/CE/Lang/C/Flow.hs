-- | The c flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_c.rs at a378e78c; from this commit on
-- the core is the authority.
module CE.Lang.C.Flow where

flow :: String
flow =
  "[flow]\n\
  \# Flow tables for the C family (plan v2.31 step 4), beside the contract\n\
  \# in spec.rs. C and C++ read one text, FAMILY, and part only on the\n\
  \# noreturn names (§5.1 rule 3: C++ adds `std::exit`, `std::abort` and\n\
  \# `std::terminate`) — the text form of scan::spec_c's `..FAMILY`.\n\
  \# FAMILY names C++-only constructs (try, lambdas, range-for, condition\n\
  \# clauses, references, default arguments, `and` / `or`): a C tree never\n\
  \# holds them, and the unit legs pin that exact list as absent from the\n\
  \# C grammar and present in the C++ one. Kinds are read off the round-3\n\
  \# probe transcripts of flow.c and flow.cpp.\n\
  \block_kinds = ['compound_statement']\n\
  \wrapper_kinds = ['expression_statement', 'attributed_statement', 'init_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \else_kinds = [['else_clause', '*']]\n\
  \cond_wrappers = [['parenthesized_expression', '*', ''], ['condition_clause', 'value', 'initializer']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', init = 'initializer', cond = 'condition', update = 'update', body = 'body'},\n\
  \  {kind = 'for_range_loop', target = 'declarator', iter = 'right', body = 'body'},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body'},\n\
  \  {kind = 'do_statement', cond = 'condition', body = 'body', body_first = true},\n\
  \]\n\
  \switches = [{kind = 'switch_statement', subject = 'condition', arms = 'body'}]\n\
  \cases = [{kind = 'case_statement', default = {missing = 'value'}, fallthrough = 'always', value = 'value', body = '*'}]\n\
  \tries = [{kind = 'try_statement', body = 'body'}]\n\
  \catches = [{kind = 'catch_clause', param = 'parameters/@parameter_declaration/declarator', body = 'body'}]\n\
  \return_kinds = ['return_statement']\n\
  \throw_kinds = ['throw_statement']\n\
  \break_kinds = ['break_statement']\n\
  \continue_kinds = ['continue_statement']\n\
  \gotos = [['goto_statement', 'label']]\n\
  \labels = [['labeled_statement', 'label', '*']]\n\
  \noreturn_attrs = [['type_qualifier', '_Noreturn'], ['attribute_specifier', 'noreturn'], ['attribute', 'noreturn']]\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['number_literal']\n\
  \dynamic_names = ['setjmp']\n\
  \dynamic_kinds = ['preproc_if', 'preproc_ifdef', 'preproc_elif', 'preproc_else', 'gnu_asm_expression']\n\
  \params = [['parameter_declaration', 'declarator'], ['optional_parameter_declaration', 'declarator']]\n\
  \decls = [\n\
  \  {kind = 'declaration', token = '=', binder = 'declarator', init = 'value'},\n\
  \  {kind = 'declaration', items = 'declarator', pair = 'init_declarator', binder = 'declarator', init = 'value', storage = '@storage_class_specifier'},\n\
  \]\n"

flow2 :: String
flow2 =
  "lasting_storage = ['static', 'extern']\n\
  \prototype_kinds = ['function_declarator']\n\
  \prototype_reads = ['type_identifier']\n\
  \pattern_kinds = ['structured_binding_declarator']\n\
  \binder_paths = [\n\
  \  ['pointer_declarator', 'declarator'], ['array_declarator', 'declarator'],\n\
  \  ['parenthesized_declarator', '*'], ['function_declarator', 'declarator'],\n\
  \  ['reference_declarator', '*'],\n\
  \]\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [['qualified_identifier', '.'], ['attribute', 'name'], ['attribute_specifier', '.']]\n\
  \member_write_bases = ['field_expression', 'subscript_expression', 'pointer_expression']\n\
  \assigns = [\n\
  \  {kind = 'assignment_expression', op = '=', left = 'left', right = 'right'},\n\
  \  {kind = 'assignment_expression', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \]\n\
  \update_kinds = [['update_expression', 'argument']]\n\
  \conditional_ctx = [\n\
  \  ['binary_expression', 'right', ['&&', '||', 'and', 'or']],\n\
  \  ['conditional_expression', 'consequence', []], ['conditional_expression', 'alternative', []],\n\
  \]\n\
  \default_arg_fields = [['optional_parameter_declaration', 'default_value']]\n\
  \head_reads = ['field_initializer_list']\n\
  \capture_kinds = ['lambda_expression', 'function_definition']\n\
  \address_ops = [['pointer_expression', '&']]\n\
  \ref_binding_kinds = ['reference_declarator']\n"

noreturn :: String
noreturn =
  "# Rule 3's C names, the list left open: C++ adds its three `std::`\n\
  \# spellings (STD) before END closes it, so no name is written twice.\n\
  \noreturn = [\n\
  \  'exit', '_exit', '_Exit', 'abort', 'quick_exit', 'longjmp', 'siglongjmp',\n\
  \  '__builtin_unreachable', '__builtin_trap',\n"

close :: String
close =
  "]\n"
