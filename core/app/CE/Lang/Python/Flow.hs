-- | The python flow table (plan v2.32 step 1),
-- transcribed from cli/src/flow/spec_py.rs at a378e78c; from this commit
-- on the core is the authority.
module CE.Lang.Python.Flow where

flow :: String
flow =
  "[flow]\n\
  \# The Python flow table (plan v2.31 step 4), beside the contract in\n\
  \# spec.rs. Every kind, field and token here is read off the round-3\n\
  \# probe transcript of scripts/tsprobe/snippets/flow.py and checked\n\
  \# against the pinned grammar by the unit legs.\n\
  \block_kinds = ['block']\n\
  \wrapper_kinds = ['expression_statement']\n\
  \if = {cond = 'condition', then = 'consequence', else = 'alternative'}\n\
  \elif_kinds = ['elif_clause']\n\
  \else_kinds = [['else_clause', 'body']]\n\
  \cond_wrappers = [['parenthesized_expression', '*', '']]\n\
  \loops = [\n\
  \  {kind = 'for_statement', target = 'left', iter = 'right', body = 'body', else = 'alternative'},\n\
  \  {kind = 'while_statement', cond = 'condition', body = 'body', else = 'alternative'},\n\
  \]\n\
  \switches = [{kind = 'match_statement', subject = 'subject', arms = 'body', passes_break = true}]\n\
  \cases = [{kind = 'case_clause', default = {holds = ['@case_pattern', '_']}, pattern = '@case_pattern', guard = 'guard', body = 'consequence'}]\n\
  \tries = [{kind = 'try_statement', body = 'body', else = '@else_clause'}]\n\
  \catches = [{kind = 'except_clause', param = 'value/alias', value = 'value', body = '@block'}]\n\
  \finally_kinds = [['finally_clause', '@block']]\n\
  \withs = [{kind = 'with_statement', item = '@with_clause/@with_item', value = 'value', binder = 'value/alias', body = 'body'}]\n\
  \return_kinds = ['return_statement']\n\
  \throw_kinds = ['raise_statement']\n\
  \break_kinds = ['break_statement']\n\
  \continue_kinds = ['continue_statement']\n\
  \noreturn = ['sys.exit', 'exit', 'quit', 'os._exit', 'os.abort']\n\
  \const_true = [['true', '']]\n\
  \int_kinds = ['integer']\n\
  \dynamic_names = ['eval', 'exec', 'compile', 'locals', 'globals', 'vars', '__import__']\n\
  \scoping = 'function'\n\
  \first_write_declares = true\n\
  \params = [\n\
  \  ['default_parameter', 'name'],\n\
  \  ['typed_parameter', '@identifier|@list_splat_pattern|@dictionary_splat_pattern'],\n\
  \  ['typed_default_parameter', 'name'],\n\
  \]\n\
  \receiver_names = ['self', 'cls']\n"

flow2 :: String
flow2 =
  "pattern_kinds = [\n\
  \  'pattern_list', 'tuple_pattern', 'list_pattern', 'list_splat_pattern', 'dictionary_splat_pattern',\n\
  \  'case_pattern', 'dict_pattern', 'splat_pattern', 'class_pattern', 'keyword_pattern',\n\
  \  'union_pattern', 'as_pattern', 'as_pattern_target',\n\
  \]\n\
  \dotted_patterns = ['dotted_name']\n\
  \nonlocal_kinds = ['global_statement', 'nonlocal_statement']\n\
  \local_only_scopes = ['list_comprehension', 'set_comprehension', 'dictionary_comprehension', 'generator_expression', 'lambda']\n\
  \ident_kinds = ['identifier']\n\
  \name_positions = [\n\
  \  ['attribute', 'attribute'], ['keyword_argument', 'name'], ['keyword_pattern', '@identifier'],\n\
  \  ['class_pattern', '@dotted_name'], ['type', '.'],\n\
  \]\n\
  \member_write_bases = ['attribute', 'subscript']\n\
  \assigns = [\n\
  \  {kind = 'assignment', left = 'left', right = 'right'},\n\
  \  {kind = 'augmented_assignment', left = 'left', right = 'right', mode = 'readwrite'},\n\
  \  {kind = 'named_expression', left = 'name', right = 'value'},\n\
  \]\n\
  \conditional_ctx = [['boolean_operator', 'right', []], ['conditional_expression', '*', []]]\n\
  \default_arg_fields = [['default_parameter', 'value'], ['typed_default_parameter', 'value']]\n\
  \capture_kinds = ['lambda', 'class_definition', 'generator_expression']\n"
