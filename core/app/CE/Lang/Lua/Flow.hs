-- | The lua flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_lua.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.Lua.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The Lua flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs; kinds read off the round-3 probe transcript of flow.lua.\n\
  \# Lua spells every name `identifier` — a field, a method, a label, a\n\
  \# `<const>` attribute — so the name positions carry most of the table;\n\
  \# an if branch may hold no block at all (§5.3: the empty flag), and a\n\
  \# `repeat … until` condition exits the loop (§5.1 rule 5).\n\
  \block_kinds = ['block']\n\
  \wrapper_kinds = ['do_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \elif_kinds = ['elseif_statement']\n\
  \else_kinds = [['else_statement', 'body']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', header = '@for_numeric_clause', target = 'name', iter = '*', body = 'body'},\n\
  \  {kind = 'for_statement', header = '@for_generic_clause', target = '@variable_list', iter = '@expression_list', body = 'body'},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body'},\n\
  \  {kind = 'repeat_statement', cond = 'condition', body = 'body', body_first = true, until = true},\n\
  \]\n\
  \return_kinds = ['return_statement']\n\
  \break_kinds = ['break_statement']\n\
  \gotos = [['goto_statement', '@identifier']]\n\
  \labels = [['label_statement', '@identifier', '']]\n\
  \noreturn = ['error', 'os.exit']\n\
  \const_true = [['true', '']]\n\
  \const_false = [['false', ''], ['nil', '']]\n\
  \int_kinds = ['number']\n\
  \dynamic_names = ['load', 'loadstring', 'dofile', 'setfenv', 'getfenv', 'debug.*']\n\
  \decls = [{kind = 'variable_declaration', binder = '@variable_list|@assignment_statement/@variable_list', init = '@assignment_statement/@expression_list'}]\n\
  \pattern_kinds = ['variable_list']\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [\n\
  \  ['dot_index_expression', 'field'], ['method_index_expression', 'method'], ['field', 'name'],\n\
  \  ['variable_list', 'attribute'], ['goto_statement', '@identifier'], ['label_statement', '@identifier'],\n\
  \]\n\
  \member_write_bases = ['dot_index_expression', 'bracket_index_expression']\n\
  \assigns = [{kind = 'assignment_statement', left = '@variable_list', right = '@expression_list'}]\n\
  \conditional_ctx = [['binary_expression', 'right', ['and', 'or']]]\n"
