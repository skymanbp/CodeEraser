-- | The go flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_go.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.Go.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The Go flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs. Every kind, field and token here is read off the round-3\n\
  \# probe transcript of scripts/tsprobe/snippets/flow.go and checked\n\
  \# against the pinned grammar by the unit legs.\n\
  \block_kinds = ['block']\n\
  \splice_kinds = ['statement_list']\n\
  \wrapper_kinds = ['expression_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative', init = 'initializer'}\n\
  \loops = [\n\
  \  {kind = 'for_statement', header = '@for_clause', init = 'initializer', cond = 'condition', update = 'update', body = 'body'},\n\
  \  {kind = 'for_statement', header = '@range_clause', target = 'left', iter = 'right', body = 'body', marker = [[':=', 'block']]},\n\
  \  {kind = 'for_statement', header = '*', cond = '.', body = 'body'},\n\
  \  {kind = 'for_statement', body = 'body'},\n\
  \]\n\
  \switches = [\n\
  \  {kind = 'expression_switch_statement', subject = 'value', arms = '.', init = 'initializer'},\n\
  \  {kind = 'type_switch_statement', subject = 'value', arms = '.', init = 'initializer', binder = 'alias'},\n\
  \  {kind = 'select_statement', arms = '.', always_default = true, empty_noreturn = true},\n\
  \]\n\
  \cases = [\n\
  \  {kind = 'expression_case', fallthrough = 'statement', value = 'value', body = '@statement_list'},\n\
  \  {kind = 'type_case', body = '@statement_list'},\n\
  \  {kind = 'communication_case', value = 'communication', body = '@statement_list'},\n\
  \  {kind = 'default_case', default = 'kind', fallthrough = 'statement', body = '@statement_list'},\n\
  \]\n\
  \return_kinds = ['return_statement']\n\
  \break_kinds = ['break_statement']\n\
  \continue_kinds = ['continue_statement']\n\
  \gotos = [['goto_statement', '@label_name']]\n\
  \labels = [['labeled_statement', 'label', '*']]\n\
  \fallthrough_kinds = ['fallthrough_statement']\n\
  \noreturn = [\n\
  \  'panic', 'os.Exit', 'log.Fatal', 'log.Fatalf', 'log.Fatalln',\n\
  \  'log.Panic', 'log.Panicf', 'log.Panicln', 'runtime.Goexit',\n\
  \]\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['int_literal']\n\
  \params = [['parameter_declaration', 'name'], ['variadic_parameter_declaration', 'name']]\n\
  \results = 'result'\n"

flow2 :: String
flow2 =
  "decls = [\n\
  \  {kind = 'short_var_declaration', binder = 'left', init = 'right', redeclare = true},\n\
  \  {kind = 'var_declaration', items = '@var_spec|@var_spec_list/@var_spec', binder = 'name', init = 'value'},\n\
  \  {kind = 'receive_statement', token = ':=', binder = 'left', init = 'right'},\n\
  \]\n\
  \pattern_kinds = ['expression_list']\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [['const_spec', 'name']]\n\
  \member_write_bases = ['selector_expression', 'index_expression', 'unary_expression']\n\
  \assigns = [\n\
  \  {kind = 'assignment_statement', op = '=', left = 'left', right = 'right'},\n\
  \  {kind = 'assignment_statement', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \  {kind = 'receive_statement', op = '=', left = 'left', right = 'right'},\n\
  \]\n\
  \update_kinds = [['inc_statement', '*'], ['dec_statement', '*']]\n\
  \conditional_ctx = [['binary_expression', 'right', ['&&', '||']]]\n\
  \capture_kinds = ['func_literal']\n\
  \address_ops = [['unary_expression', '&']]\n\
  \discard_names = ['_']\n"
