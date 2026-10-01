-- | The graph and four-class definition tables (plan v2.32 step 1),
-- transcribed from cli/src/fourclass/kinds.rs, cli/src/graph/keys.rs,
-- cli/src/graph/deadcode/flags.rs, cli/src/graph/compdb_flags.rs,
-- cli/src/graph/spec/calls.rs at a378e78c; from this commit on the core is
-- the authority.
module CE.Lang.Common.Graph where

fourclass :: String
fourclass =
  "[fourclass]\n\
  \# Wrapper kinds whose child declaration REDECLARES a name instead of\n\
  \# introducing one: tree-sitter-haskell wraps `data instance F Int =\n\
  \# …` / `newtype instance …` as a `data_instance` around a plain\n\
  \# `data_type` / `newtype` node carrying the family's own name, so\n\
  \# without this guard every instance minted a second row of the\n\
  \# family (the step-8 review's duplicate-row catch). `type instance`\n\
  \# is its own kind and never keyed.\n\
  \redeclaring = [\n\
  \  'data_instance',\n\
  \]\n\
  \# Kinds that declare only at package level — under the file root's\n\
  \# own `const_declaration` / `var_declaration` (grandparent = root):\n\
  \# the same node inside a function body declares a local, which is\n\
  \# never a cross-file identifier (plan v2.30 step 5b). A spec may name\n\
  \# several (`var a, b int`): each name is a unit of its own.\n\
  \package_level = [\n\
  \  'const_spec', 'var_spec',\n\
  \]\n\
  \# Java's field forms (plan v2.30 step 5b-6): a `field_declaration`\n\
  \# declares only when `static` is among its modifiers — JLS 8.3.1.1\n\
  \# makes a static field one variable of the class, named across files\n\
  \# as `Type.NAME`, where an instance field is every object's own; an\n\
  \# interface's or annotation type's `constant_declaration` is\n\
  \# implicitly `public static final` (JLS 9.3). Each declarator is a\n\
  \# unit of its own (declared.rs).\n\
  \java_fields = [\n\
  \  'field_declaration', 'constant_declaration',\n\
  \]\n\
  \# The C family's variable form (plan v2.30 step 5b-6): a `declaration`\n\
  \# declares only at file scope — the translation unit, a namespace or\n\
  \# linkage body, through preprocessor conditionals and a template head\n\
  \# — and only the variables it defines (C11 6.9.2: an `extern` without\n\
  \# an initializer is a reference, a prototype names a function); one\n\
  \# unit per declarator (declared.rs).\n\
  \c_variable = 'declaration'\n"

fourclass2 :: String
fourclass2 =
  "# TypeScript's lexical forms (plan v2.30 step 5b-6): `const` / `let`\n\
  \# (`lexical_declaration`) and `var` (`variable_declaration`) declare\n\
  \# only at module level — under the program, an `export` / `declare`\n\
  \# wrapper, or a namespace / module / `declare global` body; the same\n\
  \# kinds inside a function, a loop head or a bare block bind locals.\n\
  \# One unit per bound identifier (declared.rs).\n\
  \ts_lexical = [\n\
  \  'lexical_declaration', 'variable_declaration',\n\
  \]\n\
  \# Kinds that declare only with a `body`: `struct K { … }` declares K,\n\
  \# while `struct K x;`, a `struct K *` parameter type and a forward\n\
  \# `class Fwd;` reference or promise it by the SAME node kind and are\n\
  \# nothing a file can be judged on (plan v2.30 step 2).\n\
  \bodied = [\n\
  \  'struct_specifier', 'union_specifier', 'enum_specifier', 'class_specifier',\n\
  \]\n"

keys :: String
keys =
  "[keys]\n\
  \# Resolver-relevant config files (design §4 ladder inputs). The\n\
  \# tsconfig arm is a basename pattern because `extends` targets\n\
  \# conventionally read tsconfig.<flavor>.json and participate in\n\
  \# resolution — leaving them out of the key would serve stale edges\n\
  \# (2f refinement); an extends target under any other name joins the\n\
  \# key through the chain walk instead (`ts_fs_facts`, plan v2.30 step\n\
  \# 5b). compile_commands.json and compile_flags.txt (step 2, completed\n\
  \# in step 5b item 14) join the key wherever the walk reads them; the\n\
  \# C-family ladder itself finds its databases by clangd's probe\n\
  \# (compdb_find.rs) — a gitignored build directory's database never\n\
  \# reaches the walk, so the probe's own facts join the key beside these\n\
  \# (dedup/walkidx.rs). An R package's DESCRIPTION (step 4) names the\n\
  \# package the R ladder's second rung reaches.\n\
  \config_names = [\n\
  \  'Cargo.toml', 'go.mod', 'package.json', 'pyproject.toml', 'compile_commands.json',\n\
  \  'compile_flags.txt', 'DESCRIPTION',\n\
  \]\n\
  \# Source extension → the twins R2 can stat for it. The .mts/.cts\n\
  \# rows mirror esm_rewrite's [(\"js\",\"ts\"),(\"mjs\",\"mts\"),(\"cjs\",\"cts\")]\n\
  \# table (ladder/ts.rs): R2 tests `!root.join(js_twin).is_file()` on\n\
  \# exactly the .mjs/.cjs twin of an in-scope .mts/.cts source, so\n\
  \# while those rows were missing, creating or deleting foo.mjs beside\n\
  \# foo.mts flipped R2's answer under an unchanged resolve key and the\n\
  \# edge stayed stale forever — phase-1.5 cannot repair it either,\n\
  \# since dirty holds only content-refreshed judged files and .mjs is\n\
  \# scan-only, hence never judged. The .ts/.tsx row stays a superset\n\
  \# of R2's single \"js\" probe: a surplus stat fact costs one spurious\n\
  \# sweep, a missing one costs a permanently wrong edge.\n\
  \twin_exts = [\n\
  \  ['.ts', ['js', 'mjs', 'cjs']],\n\
  \  ['.tsx', ['js', 'mjs', 'cjs']],\n\
  \  ['.mts', ['mjs']],\n\
  \  ['.cts', ['cjs']],\n\
  \]\n"

flags :: String
flags =
  "[flags]\n\
  \# Files a runtime or a build starts by their own name, which nothing\n\
  \# imports: Rust's `main.rs` and `build.rs`, Go's `main.go`, Python's\n\
  \# `__main__.py`, cabal's executable main-is `Main.hs`, the C family's\n\
  \# `main` file, the class a Java launcher names, what LÖVE runs\n\
  \# (`main.lua`, and `conf.lua` before it) and what Shiny's `runApp`\n\
  \# reads from an app directory (`app.R`, or `ui.R` and `server.R`, and\n\
  \# `global.R`), and the pages a web server serves by name — a\n\
  \# directory's `index.html`, the not-found page `404.html` (plan v2.30\n\
  \# step 5). Neovim's `init.lua` is one only at the root, where a\n\
  \# config keeps it: anywhere else the name is a module's own file\n\
  \# (`require \"a\"` reads `a/init.lua`), which the graph reaches.\n\
  \entry_names = [\n\
  \  'main.rs', 'build.rs', 'main.go', '__main__.py', 'Main.hs', 'main.c', 'main.cc',\n\
  \  'main.cpp', 'Main.java', 'main.lua', 'conf.lua', 'app.R', 'ui.R', 'server.R',\n\
  \  'global.R', 'index.html', '404.html',\n\
  \]\n\
  \# Directories whose files a runtime starts by where they sit, never\n\
  \# by an import — one row per language (`*` every judged one), each\n\
  \# under the tree root except R's, which sit under each package root\n\
  \# (a DESCRIPTION's directory, targets.rs): Cargo's `src/bin/`\n\
  \# `examples/` `benches/` and Go's\n\
  \# `cmd/`; the Neovim runtime directories a Lua file is sourced from by\n\
  \# path (`plugin/` at startup; `ftplugin/` `indent/` `syntax/`\n\
  \# `colors/` `compiler/` `ftdetect/` `lsp/` on demand; `after/`\n\
  \# holding the same — `autoload/` is Vim script's alone, `lua/` is\n\
  \# `require`'s); the directories of an R package whose scripts R and\n\
  \# its tools run by path (`inst/` installed as it is, `vignettes/`,\n\
  \# `data-raw/`, `exec/`, `demo/`).\n\
  \entry_dirs = [\n\
  \  ['*', ['src/bin/', 'examples/', 'benches/', 'cmd/']],\n\
  \  ['lua', [\n\
  \    'plugin/', 'ftplugin/', 'indent/', 'syntax/', 'colors/', 'compiler/', 'ftdetect/',\n\
  \    'lsp/', 'after/',\n\
  \  ]],\n\
  \  ['r', ['inst/', 'vignettes/', 'data-raw/', 'exec/', 'demo/']],\n\
  \]\n"

compdb :: String
compdb =
  "[compdb]\n\
  \# Recognized GNU joined spellings, longest conflicting spelling first,\n\
  \# as one space-separated literal like SKIP below: a slice of a dozen\n\
  \# `&str` rows is the clone gate's most-rhyming shape (it read these\n\
  \# two against the tombstone role's stems and the walk's built-in\n\
  \# excludes).\n\
  \gnu = [\n\
  \  '-include-pch', '-iwithprefixbefore', '-isystem-after', '-iwithprefix', '-iframework',\n\
  \  '-idirafter', '-isystem', '-isysroot', '-iquote', '-iprefix', '--include=', '--include',\n\
  \  '-include', '--imacros', '-imacros', '--sysroot=', '-I', '-F',\n\
  \]\n\
  \# GNU separate operands that cannot themselves open include options.\n\
  \skip = [\n\
  \  '-o', '-MF', '-MT', '-MQ', '-x', '-arch', '-target', '-mllvm', '-D', '-U', '-L', '-l',\n\
  \  '-z', '-u', '-e', '-T', '-B', '-b', '-V', '--sysroot',\n\
  \]\n"

calls :: String
calls =
  "[calls]\n\
  \# The formals before `...` of each R callee, one line per callee as\n\
  \# `name: formals`, read off R-devel 4.6.0's help pages (base `source`,\n\
  \# `sys.source`, `library`, `ns-load`; 2026-09-26). The table stops at\n\
  \# the dots: a formal after them matches only exactly (R Language\n\
  \# Definition §4.3.2) and is never a row's target. The first formal is\n\
  \# the target. One literal, not a tuple table — rows of one shape rhyme\n\
  \# under the clone gate.\n\
  \r_formals = [\n\
  \  ['source', [\n\
  \    'file', 'local', 'echo', 'print.eval', 'exprs', 'spaced', 'verbose', 'prompt.echo',\n\
  \    'max.deparse.length', 'width.cutoff', 'deparseCtrl', 'chdir', 'catch.aborts',\n\
  \    'encoding', 'continue.echo', 'skip.echo', 'keep.source',\n\
  \  ]],\n\
  \  ['sys.source', [\n\
  \    'file', 'envir', 'chdir', 'keep.source', 'keep.parse.data', 'toplevel.env',\n\
  \  ]],\n\
  \  ['library', [\n\
  \    'package', 'help', 'pos', 'lib.loc', 'character.only', 'logical.return',\n\
  \    'warn.conflicts', 'quietly', 'verbose', 'mask.ok', 'exclude', 'include.only',\n\
  \    'attach.required',\n\
  \  ]],\n\
  \  ['require', [\n\
  \    'package', 'lib.loc', 'quietly', 'warn.conflicts', 'character.only', 'mask.ok',\n\
  \    'exclude', 'include.only', 'attach.required',\n\
  \  ]],\n\
  \  ['requireNamespace', ['package']],\n\
  \  ['loadNamespace', [\n\
  \    'package', 'lib.loc', 'keep.source', 'partial', 'versionCheck', 'keep.parse.data',\n\
  \  ]],\n\
  \]\n"
