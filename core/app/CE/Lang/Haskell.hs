-- | The haskell definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_hs.rs, cli/src/merge/slot_hs.rs,
-- cli/src/graph/spec.rs, cli/src/fourclass/kinds.rs at e877f389; from this
-- commit on the core is the authority.
module CE.Lang.Haskell where

top :: String
top =
  "name = 'haskell'\n\
  \extra = [\n\
  \  'data_type', 'newtype', 'type_synomym', 'class', 'type_family', 'data_family',\n\
  \]\n\
  \sites = [\n\
  \  # a `package` child (PackageImports) leads the spec when present\n\
  \  {node = 'import', label = 'import', via = {form = 'spanned', arg = {from = 'package', to = 'module'}}},\n\
  \]\n"

scan :: String
scan =
  "[scan]\n\
  \# The Haskell LangSpec table (M5-3k). Lives outside spec.rs per the\n\
  \# design's split plan (RM16: spec.rs is a 300-line throat).\n\
  \#\n\
  \# Every kind below is AST-probed against tree-sitter-haskell 0.23.1\n\
  \# (probe transcripts 2026-08-14, two rounds: constructs + fields),\n\
  \# never guessed. Haskell has NO external cognitive-complexity\n\
  \# oracle — S3776 names no Haskell constructs — so each mapping is a\n\
  \# recorded STANCE with its nearest whitepaper analogue; the full\n\
  \# divergence register lives in contracts/coc-haskell-divergences.md\n\
  \# and every entry is pinned by tests/sonar_whitepaper.rs.\n\
  \#\n\
  \# Key probe facts the table stands on:\n\
  \# - one `function` node PER EQUATION (same name, own patterns);\n\
  \#   with_nth keys the equations apart, same as Rust impl siblings\n\
  \# - `bind` is three things at once: a top-level/where/let value\n\
  \#   binding (HAS a `name` field), a do-statement `x <- act`, and a\n\
  \#   pattern bind (both name-less) — hence the fn_required_fields gate\n\
  \# - `infix` carries fields left_operand/operator/right_operand (the\n\
  \#   `operator` field is what the event emitter matches on; the\n\
  \#   operand alias is handled in metrics/events.rs::collect_in_order)\n\
  \# - guard alternatives are `guards` nodes; each condition inside is\n\
  \#   a `boolean` / `pattern_guard` / `let` qualifier\n\
  \# - if-then-else is the `conditional` EXPRESSION — the ternary\n\
  \#   analogue, so else-if chains pay nesting like nested ternaries\n\
  \#   (S3776's else-if exemption covers statement ifs)\n\
  \# Every named binding is a unit — equations (`function`) and\n\
  \# value binds (`bind`, name-gated). Local where/let binds are\n\
  \# standalone units, the Rust-closure precedent; a 60-line\n\
  \# point-free pipeline must not escape the E01 function gate.\n\
  \fn_kinds = ['function', 'bind']\n\
  \# BOTH unit kinds are name-gated: `bind` is also the do-statement\n\
  \# / pattern-bind kind, and `function` is ALSO the arrow TYPE\n\
  \# (`A -> B` inside a signature — battery-caught: a 3-arg\n\
  \# signature minted three spurious units). Value-level equations\n\
  \# and binds always carry the `name` field; the impostors never do.\n\
  \fn_required_fields = [['function', 'name'], ['bind', 'name']]\n\
  \param_list_kinds = ['patterns']\n"

scan2 :: String
scan2 =
  "cc_kinds = [\n\
  \  'conditional',\n\
  \  # every case/lambda_case arm, wildcard included (Rust\n\
  \  # match_arm precedent: case is total, `_` is a real path;\n\
  \  # divergence register: gocyclo's default-skip not adopted)\n\
  \  'alternative',\n\
  \  # one per guard CONDITION, not per `guards` group: `| a, b`\n\
  \  # is two decisions, matching `&&` counting; also the list-\n\
  \  # comprehension filter qualifier (Python if_clause precedent)\n\
  \  'boolean',\n\
  \  'pattern_guard',\n\
  \  # list-comprehension generator (Python for_in_clause\n\
  \  # precedent: a real branch path CC counts, CoC does not)\n\
  \  'generator',\n\
  \]\n\
  \# && and || arrive as `operator` leaves under `infix`; matched by\n\
  \# TEXT via the operator field, same as every other language.\n\
  \cc_operators = ['&&', '||']\n\
  \chain_kinds = []\n\
  \# `case e of`: the scrutinee is the header. `if` is an expression\n\
  \# (a ternary) and a multi-way if has no header: both nest whole\n\
  \coc_nesting_kinds = ['conditional', 'case alternatives', 'multi_way_if']\n\
  \# `if … then … else …` is the `conditional` EXPRESSION, whose else\n\
  \# is its own field and scores like a nested ternary (module doc):\n\
  \# no if statement carries an alternative-field else here\n\
  \if_kinds = []\n\
  \# each guarded alternative is +1 flat — the elif analogue\n\
  \# (whitepaper p.7 hybrid increments: no nesting penalty)\n\
  \coc_flat_kinds = ['guards']\n\
  \# lambdas absorb into their host and raise nesting only\n\
  \# (whitepaper p.13 nesting-level list; Go func_literal precedent)\n\
  \coc_nest_only_kinds = ['lambda', 'lambda_case']\n\
  \coc_operators = ['&&', '||']\n\
  \# Haskell has no labeled jumps.\n\
  \coc_jump_kinds = []\n\
  \label_kinds = []\n\
  \# haddock (`-- |`, `{-| -}`) is a distinct kind beside comment.\n\
  \comment_kinds = ['comment', 'haddock']\n\
  \name_style = 'mixed_caps'\n"

scan3 :: String
scan3 =
  "# string literals lex as ONE `string` leaf (quotes inside the\n\
  \# token) — no anonymous delimiter tokens exist to classify.\n\
  \literal_delims = []\n\
  \# `f a b` nests as apply(apply(f, a), b), so only the innermost\n\
  \# application carries a `variable` in callee position: one\n\
  \# application chain yields one edge, not one per argument.\n\
  \call_kinds = ['apply']\n\
  \call_fields = ['function', null]\n\
  \call_name_kinds = ['variable']\n\
  \# No receiver syntax: a Haskell equation is not a method.\n\
  \call_member_kinds = []\n\
  \call_self_words = []\n\
  \# a class method is a top-level name in Haskell, bare-callable\n\
  \# like any other; the probe finds no member scope to exclude\n\
  \call_member_scopes = []\n\
  \owner_kinds = []\n\
  \# every equation of `f` is its own `function` node (divergence D7):\n\
  \# same-named units of one scope are one callable, never overloads\n\
  \overloads = null\n\
  \call_import_kinds = []\n\
  \opaque_fields = []\n"

slot :: String
slot =
  "[[slot]]\n\
  \# The Haskell slot table (plan v2.31 step 7, step-7 ruling 8), beside\n\
  \# the contract in slot.rs. Haskell has no flow table, so this table\n\
  \# names its own statement forms and containers: the do block's\n\
  \# statements and the declarations (a declaration held by `declarations`,\n\
  \# `local_binds` or a class or instance body stands in a statement\n\
  \# position through its container). The grammar spells an expression,\n\
  \# a pattern and a type with the same kinds (`apply`, `variable`,\n\
  \# `tuple`…), so the type class holds only the kinds that are types\n\
  \# alone — a type constructor's `name`, the arrow `function` (a function\n\
  \# declaration reaches 0 through its container first), `forall`,\n\
  \# `context` and the like — and a shared kind reads as an expression.\n\
  \# Every kind and field here is read off the probe\n\
  \# (scripts/tsprobe/snippets/flow.hs) and tree-sitter-haskell 0.23.1's\n\
  \# node-types.json.\n\
  \stmt_kinds = [\n\
  \  'bind', 'exp', 'let', 'rec', 'signature', 'data_type', 'newtype', 'class', 'instance', 'import',\n\
  \  'type_synomym', 'fixity', 'deriving_instance', 'foreign_import', 'foreign_export', 'type_family',\n\
  \  'data_family', 'type_instance', 'data_instance', 'kind_signature', 'pattern_synonym',\n\
  \  'top_splice', 'default_types', 'role_annotation', 'default_signature',\n\
  \]\n\
  \container_kinds = [\n\
  \  'do', 'declarations', 'local_binds', 'class_declarations', 'instance_declarations', 'imports',\n\
  \]\n\
  \expr_kinds = [\n\
  \  'apply', 'infix', 'variable', 'literal', 'integer', 'float', 'char', 'string', 'parens',\n\
  \  'lambda', 'lambda_case', 'lambda_cases', 'case', 'do', 'conditional', 'let_in', 'list', 'tuple',\n\
  \  'unit', 'constructor', 'operator', 'qualified', 'left_section', 'right_section', 'negation',\n\
  \  'record', 'projection', 'list_comprehension', 'arithmetic_sequence', 'multi_way_if', 'prefix_id',\n\
  \  'infix_id', 'wildcard', 'label', 'quasiquote', 'implicit_variable', 'empty_list',\n\
  \]\n\
  \type_kinds = [\n\
  \  'name', 'function', 'star', 'forall', 'forall_required', 'context', 'promoted', 'prefix_list',\n\
  \  'linear_function', 'kind_application', 'type_application',\n\
  \]\n\
  \name_fields = [['function', 'name'], ['bind', 'name'], ['signature', 'name']]\n"

slot2 :: String
slot2 =
  "part_fields = [['projection', 'field']]\n\
  \helper_lines = 1\n\
  \other_kinds = [\n\
  \  'haskell', 'header', 'module', 'module_id', 'exports', 'export', 'children', 'all_names',\n\
  \  'imports', 'import_list', 'import_name', 'declarations', 'class_declarations',\n\
  \  'instance_declarations', 'local_binds', 'match', 'guards', 'boolean', 'pattern_guard',\n\
  \  'alternatives', 'alternative',\n\
  \  'patterns', 'data_constructors', 'data_constructor', 'prefix', 'deriving', 'fields', 'field',\n\
  \  'field_name', 'field_update', 'newtype_constructor', 'type_params', 'type_patterns', 'generator',\n\
  \  'qualifiers', 'comment', 'haddock', 'pragma', 'cpp',\n\
  \]\n"
