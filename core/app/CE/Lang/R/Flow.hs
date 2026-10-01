-- | The r flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_r.rs at a378e78c; from this commit on
-- the core is the authority.
module CE.Lang.R.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The R flow table (plan v2.31 step 4), beside the contract in spec.rs;\n\
  \# kinds read off the round-3 probe transcript of flow.R. R has no\n\
  \# return statement — `return(x)` is a call, read by name (§5.1 rule 3)\n\
  \# — and every assignment is one binary_operator told apart by its\n\
  \# operator token, `->` and `->>` writing to their right (rules 8, 9).\n\
  \block_kinds = ['braced_expression']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \cond_wrappers = [['parenthesized_expression', 'body', '']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', target = 'variable', iter = 'sequence', body = 'body'},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body'},\n\
  \  {kind = 'repeat_statement', body = 'body'},\n\
  \]\n\
  \break_kinds = ['break']\n\
  \continue_kinds = ['next']\n\
  \noreturn = ['stop', 'quit', 'q', 'abort', 'cli_abort', 'rlang::abort', 'cli::cli_abort']\n\
  \return_calls = ['return']\n\
  \dispatch_calls = ['UseMethod', 'NextMethod', 'standardGeneric', 'callNextMethod']\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['integer', 'float']\n\
  \dynamic_names = [\n\
  \  'eval', 'evalq', 'eval.parent', 'parse', 'assign', 'get', 'get0', 'mget', 'exists', 'rm',\n\
  \  'environment', 'sys.function', 'parent.frame', 'local', 'with', 'within', 'attach', 'source',\n\
  \]\n\
  \scoping = 'function'\n\
  \first_write_declares = true\n\
  \params = [['parameter', 'name']]\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [['extract_operator', 'rhs'], ['namespace_operator', '.'], ['argument', 'name']]\n\
  \interpolated_strings = ['string']\n\
  \member_write_bases = ['extract_operator', 'subset', 'subset2', 'call']\n\
  \assigns = [\n\
  \  {kind = 'binary_operator', op = '<-', left = 'lhs', right = 'rhs'},\n\
  \  {kind = 'binary_operator', op = '=', left = 'lhs', right = 'rhs'},\n\
  \  {kind = 'binary_operator', op = '->', left = 'rhs', right = 'lhs'},\n\
  \  {kind = 'binary_operator', op = '<<-', left = 'lhs', right = 'rhs', mode = 'outer'},\n\
  \  {kind = 'binary_operator', op = '->>', left = 'rhs', right = 'lhs', mode = 'outer'},\n\
  \]\n"

flow2 :: String
flow2 =
  "conditional_ctx = [\n\
  \  ['binary_operator', 'rhs', ['&&', '||']],\n\
  \  ['if_statement', 'consequence', []], ['if_statement', 'alternative', []],\n\
  \]\n\
  \default_arg_fields = [['parameter', 'default']]\n"
