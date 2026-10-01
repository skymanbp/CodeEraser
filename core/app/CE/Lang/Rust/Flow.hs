-- | The rust flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_rs.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.Rust.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The Rust flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs. Every kind, field and token here is read off the round-3\n\
  \# probe transcript of scripts/tsprobe/snippets/flow.rs and checked\n\
  \# against the pinned grammar by the unit legs.\n\
  \block_kinds = ['block']\n\
  \wrapper_kinds = ['expression_statement', 'unsafe_block']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \else_kinds = [['else_clause', '*']]\n\
  \loops = [\n\
  \  {kind = 'loop_expression', body = 'body'},\n\
  \  {kind = 'while_expression', cond = 'condition', body = 'body'},\n\
  \  {kind = 'for_expression', target = 'pattern', iter = 'value', body = 'body'},\n\
  \]\n\
  \switches = [{kind = 'match_expression', subject = 'value', arms = 'body', always_default = true, passes_break = true}]\n\
  \cases = [{kind = 'match_arm', pattern = 'pattern', guard = 'pattern/condition', body = 'value'}]\n\
  \return_kinds = ['return_expression']\n\
  \break_kinds = ['break_expression']\n\
  \continue_kinds = ['continue_expression']\n\
  \self_label = '@label'\n\
  \noreturn = ['panic!', 'unreachable!', 'todo!', 'unimplemented!', 'std::process::exit', 'process::exit']\n\
  \macros = [{kind = 'macro_invocation', name = 'macro', args = '@token_tree', strings = ['string_literal', 'raw_string_literal']}]\n\
  \const_true = [['boolean_literal', 'true']]\n\
  \int_kinds = ['integer_literal']\n\
  \params = [['parameter', 'pattern']]\n\
  \decls = [{kind = 'let_declaration', binder = 'pattern', init = 'value', alternative = 'alternative'}]\n\
  \pattern_kinds = [\n\
  \  'tuple_pattern', 'tuple_struct_pattern', 'struct_pattern', 'field_pattern',\n\
  \  'captured_pattern', 'reference_pattern', 'match_pattern',\n\
  \]\n\
  \pattern_idents = ['shorthand_field_identifier']\n\
  \upper_pattern_paths = true\n\
  \pattern_binders = [['let_condition', 'pattern']]\n\
  \branch_binders = ['let_condition']\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [\n\
  \  ['scoped_identifier', '.'], ['label', '.'], ['tuple_struct_pattern', 'type'],\n\
  \  ['macro_invocation', 'macro'], ['use_declaration', 'argument'],\n\
  \]\n\
  \member_write_bases = ['field_expression', 'index_expression', 'unary_expression']\n"

flow2 :: String
flow2 =
  "assigns = [\n\
  \  {kind = 'assignment_expression', left = 'left', right = 'right'},\n\
  \  {kind = 'compound_assignment_expr', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \]\n\
  \conditional_ctx = [\n\
  \  ['binary_expression', 'right', ['&&', '||']],\n\
  \  ['if_expression', 'consequence', []], ['if_expression', 'alternative', []],\n\
  \  ['match_expression', 'body', []],\n\
  \]\n\
  \capture_kinds = ['async_block']\n\
  \address_ops = [['reference_expression', '@mutable_specifier']]\n"
