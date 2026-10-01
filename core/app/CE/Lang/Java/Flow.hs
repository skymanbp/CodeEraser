-- | The java flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_java.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.Java.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The Java flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs; kinds read off the round-3 probe transcript of flow.java.\n\
  \# A `switch` is one kind in both positions (switch_expression), its arms\n\
  \# either colon groups that fall through or arrow rules that do not\n\
  \# (§5.1 rule 6); `yield` is a break to it (rule 4).\n\
  \block_kinds = ['block', 'constructor_body']\n\
  \wrapper_kinds = ['expression_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \cond_wrappers = [['parenthesized_expression', '*', '']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', init = 'init', cond = 'condition', update = 'update', body = 'body'},\n\
  \  {kind = 'enhanced_for_statement', target = 'name', iter = 'value', body = 'body'},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body'},\n\
  \  {kind = 'do_statement', cond = 'condition', body = 'body', body_first = true},\n\
  \]\n\
  \switches = [{kind = 'switch_expression', subject = 'condition', arms = 'body'}]\n\
  \cases = [\n\
  \  {kind = 'switch_block_statement_group', default = {holds = ['@switch_label', 'default']}, fallthrough = 'always', pattern = '@switch_label/@pattern', value = '@switch_label', guard = '@switch_label/@guard', body = '*'},\n\
  \  {kind = 'switch_rule', default = {holds = ['@switch_label', 'default']}, pattern = '@switch_label/@pattern', value = '@switch_label', guard = '@switch_label/@guard', body = '*'},\n\
  \]\n\
  \tries = [\n\
  \  {kind = 'try_statement', body = 'body'},\n\
  \  {kind = 'try_with_resources_statement', body = 'body', resources = 'resources/@resource'},\n\
  \]\n\
  \catches = [{kind = 'catch_clause', param = '@catch_formal_parameter/name', body = 'body'}]\n\
  \finally_kinds = [['finally_clause', '@block']]\n\
  \return_kinds = ['return_statement']\n\
  \throw_kinds = ['throw_statement']\n\
  \break_kinds = ['break_statement']\n\
  \continue_kinds = ['continue_statement']\n\
  \labels = [['labeled_statement', '@identifier', '*']]\n\
  \yield_kinds = ['yield_statement']\n\
  \noreturn = ['System.exit']\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['decimal_integer_literal']\n\
  \params = [['formal_parameter', 'name'], ['spread_parameter', '@variable_declarator/name']]\n\
  \decls = [\n\
  \  {kind = 'local_variable_declaration', items = 'declarator', binder = 'name', init = 'value'},\n\
  \  {kind = 'resource', binder = 'name', init = 'value'},\n\
  \]\n"

flow2 :: String
flow2 =
  "pattern_kinds = ['pattern', 'record_pattern', 'record_pattern_body', 'record_pattern_component', 'type_pattern']\n\
  \pattern_binders = [['instanceof_expression', 'name']]\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [\n\
  \  ['field_access', 'field'], ['method_invocation', 'name'], ['record_pattern', '@identifier'],\n\
  \  ['scoped_identifier', '.'], ['labeled_statement', '@identifier'], ['break_statement', '@identifier'],\n\
  \  ['continue_statement', '@identifier'], ['marker_annotation', 'name'],\n\
  \  ['class_declaration', 'name'], ['record_declaration', 'name'],\n\
  \]\n\
  \member_write_bases = ['field_access', 'array_access']\n\
  \assigns = [\n\
  \  {kind = 'assignment_expression', op = '=', left = 'left', right = 'right'},\n\
  \  {kind = 'assignment_expression', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \]\n\
  \update_kinds = [['update_expression', '*']]\n\
  \conditional_ctx = [\n\
  \  ['binary_expression', 'right', ['&&', '||']],\n\
  \  ['ternary_expression', 'consequence', []], ['ternary_expression', 'alternative', []],\n\
  \  ['switch_expression', 'body', []],\n\
  \]\n\
  \capture_kinds = ['lambda_expression', 'class_body']\n"
