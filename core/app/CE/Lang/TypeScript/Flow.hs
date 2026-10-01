-- | The typescript flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_ts.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.TypeScript.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The TypeScript flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs. Every kind, field and token here is read off the round-3\n\
  \# probe transcripts of scripts/tsprobe/snippets/flow.ts and flow.tsx\n\
  \# and checked against the pinned grammar by the unit legs; TypeScript\n\
  \# and TSX share it (the TSX grammar's kinds are a superset, less the\n\
  \# `<T>x` type_assertion no row names).\n\
  \block_kinds = ['statement_block']\n\
  \wrapper_kinds = ['expression_statement']\n\
  \empty_kinds = ['empty_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \else_kinds = [['else_clause', '*']]\n\
  \cond_wrappers = [['parenthesized_expression', '*', '']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', init = 'initializer', cond = 'condition', update = 'increment', body = 'body'},\n\
  \  {kind = 'for_in_statement', target = 'left', iter = 'right', body = 'body', marker = [['const', 'block'], ['let', 'block'], ['var', 'function']]},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body'},\n\
  \  {kind = 'do_statement', cond = 'condition', body = 'body', body_first = true},\n\
  \]\n\
  \switches = [{kind = 'switch_statement', subject = 'value', arms = 'body'}]\n\
  \cases = [\n\
  \  {kind = 'switch_case', fallthrough = 'always', value = 'value', body = 'body'},\n\
  \  {kind = 'switch_default', default = 'kind', fallthrough = 'always', body = 'body'},\n\
  \]\n\
  \tries = [{kind = 'try_statement', body = 'body'}]\n\
  \catches = [{kind = 'catch_clause', param = 'parameter', body = 'body'}]\n\
  \finally_kinds = [['finally_clause', 'body']]\n\
  \return_kinds = ['return_statement']\n\
  \throw_kinds = ['throw_statement']\n\
  \break_kinds = ['break_statement']\n\
  \continue_kinds = ['continue_statement']\n\
  \labels = [['labeled_statement', 'label', 'body']]\n\
  \noreturn = ['process.exit']\n\
  \call_forms = [['new_expression', 'constructor']]\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['number']\n\
  \dynamic_names = ['eval', 'Function']\n\
  \dynamic_kinds = ['with_statement']\n\
  \params = [['required_parameter', 'pattern'], ['optional_parameter', 'pattern']]\n"

flow2 :: String
flow2 =
  "decls = [\n\
  \  {kind = 'lexical_declaration', items = '@variable_declarator', binder = 'name', init = 'value'},\n\
  \  {kind = 'variable_declaration', scope = 'function', items = '@variable_declarator', binder = 'name', init = 'value'},\n\
  \]\n\
  \pattern_kinds = ['object_pattern', 'array_pattern', 'rest_pattern', 'pair_pattern']\n\
  \pattern_idents = ['shorthand_property_identifier_pattern']\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [['type_annotation', '.']]\n\
  \type_reads = ['type_query']\n\
  \field_params = [\n\
  \  ['required_parameter', '@accessibility_modifier'], ['required_parameter', 'readonly'],\n\
  \  ['optional_parameter', '@accessibility_modifier'], ['optional_parameter', 'readonly'],\n\
  \]\n\
  \shorthand_kinds = ['shorthand_property_identifier']\n\
  \member_write_bases = ['member_expression', 'subscript_expression']\n\
  \assigns = [\n\
  \  {kind = 'assignment_expression', left = 'left', right = 'right'},\n\
  \  {kind = 'augmented_assignment_expression', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \]\n\
  \update_kinds = [['update_expression', 'argument']]\n\
  \conditional_ctx = [\n\
  \  ['binary_expression', 'right', ['&&', '||', '??']],\n\
  \  ['ternary_expression', 'consequence', []], ['ternary_expression', 'alternative', []],\n\
  \]\n\
  \default_arg_fields = [['required_parameter', 'value']]\n\
  \capture_kinds = ['class_declaration']\n\
  \forward_captures = true\n"
