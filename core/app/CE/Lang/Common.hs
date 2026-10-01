-- | The common definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/lang.rs, cli/src/scan/walk.rs,
-- cli/src/scan/outputs.rs at a378e78c; from this commit on the core is the
-- authority.
module CE.Lang.Common where

outputs :: String
outputs =
  "# Build-output directories — the one part of the built-in excludes\n\
  \# (plan §4.1, walk.rs) that is decided by what sits BESIDE a\n\
  \# directory rather than by its name alone (plan v2.30 step 5). A\n\
  \# build tool writes its default output directory next to its project\n\
  \# file, so `target/` beside a `Cargo.toml` is Cargo's and a source\n\
  \# directory of that name there would collide with the tool's own\n\
  \# output. Anywhere else the name is an ordinary directory: luarocks\n\
  \# keeps its build back ends in `src/luarocks/build/` (the module\n\
  \# `luarocks.build`), a KOReader plugin ships its modules in\n\
  \# `target/`, and a Java or Go package may well be called `build`. The\n\
  \# any-depth globs this replaces hid every one of them from every\n\
  \# measurement, the reference graph included.\n\
  \#\n\
  \# One row per directory name: the project files whose tool writes it\n\
  \# by default, per that tool's own documentation — Cargo, Maven, sbt\n\
  \# and Leiningen `target/`; Gradle, setuptools and Flutter `build/`;\n\
  \# webpack, Vite and Parcel (`package.json`), setuptools' sdist and\n\
  \# wheel, and Cabal's first build tree `dist/`; Cabal's current one\n\
  \# `dist-newstyle/`. A `*` row entry matches any file with that suffix\n\
  \# (a package's `<name>.cabal`). A tool whose output directory is a\n\
  \# convention rather than a default (CMake's `-B build`) is no row: a\n\
  \# build tree the table does not name is excluded the declarative way,\n\
  \# by the tree's `.gitignore` — where such trees live in practice — or\n\
  \# a ce.toml `exclude`.\n\
  \outputs = [\n\
  \  ['target', ['Cargo.toml', 'pom.xml', 'build.sbt', 'project.clj']],\n\
  \  ['build', [\n\
  \    'build.gradle', 'build.gradle.kts', 'settings.gradle', 'settings.gradle.kts',\n\
  \    'setup.py', 'setup.cfg', 'pyproject.toml', 'pubspec.yaml',\n\
  \  ]],\n\
  \  ['dist', ['package.json', 'setup.py', 'setup.cfg', 'pyproject.toml', '*.cabal']],\n\
  \  ['dist-newstyle', ['cabal.project', '*.cabal']],\n\
  \]\n"

languages :: String
languages =
  "[languages]\n\
  \# ONE row per language: variant, extensions, report name, scan-only\n\
  \# bit. This table drives from_path / name / scan_only — as separate\n\
  \# matches each was a cyclomatic-warn-sized copy of the same facts.\n\
  \#\n\
  \# judged: the language is in the judged set (the scan-only arm, the\n\
  \# prose-only arm and the wire sentinel are not); document: a document\n\
  \# language (Markdown, HTML) — the two columns the measuring side derived\n\
  \# from the arms and from its own matches. flow_judged: the language's\n\
  \# flow findings are verdicts, not advice — its blind-reviewed flow\n\
  \# precision exam passed (plan v2.31 step 4 commit G; the set moved here\n\
  \# from cli/src/flow/mod.rs at plan v2.32 step 2). Implies judged.\n\
  \rows = [\n\
  \  {code = 0, name = 'python', exts = ['py'], judged = true, flow_judged = true},\n\
  \  {code = 1, name = 'typescript', exts = ['ts', 'mts', 'cts'], judged = true, flow_judged = true},\n\
  \  {code = 2, name = 'tsx', exts = ['tsx'], judged = true, flow_judged = true},\n\
  \  {code = 3, name = 'rust', exts = ['rs'], judged = true, flow_judged = true},\n\
  \  {code = 4, name = 'go', exts = ['go'], judged = true, flow_judged = true},\n\
  \  {code = 5, name = 'markdown', exts = ['md', 'markdown'], judged = true, document = true},\n\
  \  {code = 6, name = 'haskell', exts = ['hs'], judged = true},\n\
  \  {code = 7, name = 'unknown', exts = []},\n\
  \  {code = 8, name = 'javascript', exts = ['js', 'mjs', 'cjs', 'jsx'], scan_only = true},\n\
  \  {code = 9, name = 'css', exts = ['css', 'scss', 'less'], scan_only = true},\n\
  \  {code = 10, name = 'html', exts = ['html', 'htm'], judged = true, document = true},\n\
  \  {code = 11, name = 'vue', exts = ['vue'], scan_only = true},\n\
  \  {code = 12, name = 'svelte', exts = ['svelte'], scan_only = true},\n\
  \  {code = 13, name = 'shell', exts = ['sh', 'bash'], scan_only = true},\n\
  \  {code = 14, name = 'yaml', exts = ['yml', 'yaml'], scan_only = true},\n\
  \  {code = 15, name = 'c', exts = ['c'], judged = true, flow_judged = true},\n\
  \  {code = 16, name = 'cpp', exts = ['cpp', 'cc', 'cxx', 'hpp', 'hh', 'hxx', 'h', 'inl'], judged = true, flow_judged = true},\n\
  \  {code = 17, name = 'lua', exts = ['lua'], judged = true, flow_judged = true},\n\
  \  {code = 18, name = 'java', exts = ['java'], judged = true, flow_judged = true},\n\
  \  {code = 19, name = 'ruby', exts = [], scan_only = true},\n\
  \  {code = 20, name = 'r', exts = ['R', 'r'], judged = true, flow_judged = true},\n\
  \  {code = 21, name = 'text', exts = ['txt'], prose_only = true},\n\
  \]\n"

languages2 :: String
languages2 =
  "# The `.txt` names a specification reserves for a machine format\n\
  \# (plan v2.30 step 5b-8) — no prose and no language of ours,\n\
  \# outside every arm like a Makefile: CMake's list file (the CMake\n\
  \# language; cmake-language(7) fixes the name), clang's flat\n\
  \# compilation database (one argument per line; the\n\
  \# JSONCompilationDatabase page fixes the name, graph/compdb_find.rs\n\
  \# reads it) and the robots exclusion file (RFC 9309 §2.3 fixes the\n\
  \# path). A name only a convention suggests — `requirements.txt`,\n\
  \# which pip reads under any name handed to `-r` — stays prose.\n\
  \machine_txt = [\n\
  \  'CMakeLists.txt', 'compile_flags.txt', 'robots.txt',\n\
  \]\n\
  \# The mention tokenizer's `$` arm (sealed criterion §2, frozen beside\n\
  \# the boundary predicate above because it is an extension table of\n\
  \# the same kind): files of these extensions keep a `$`-carrying run\n\
  \# WHOLE — `$ZodString` and `ZodString` are distinct identifiers in\n\
  \# the JS family, and so are `Outer$Inner` and `Inner` in Java, whose\n\
  \# identifiers take `$` too; emitting the `$`-free piece would let each\n\
  \# hide the other's death. Every other extension, and no extension,\n\
  \# takes the union arm (shell `$name`, Haskell `f$g`). A `MENTION_REV`\n\
  \# input.\n\
  \mention_whole_run_exts = [\n\
  \  'ts', 'tsx', 'mts', 'cts', 'js', 'mjs', 'cjs', 'jsx', 'vue', 'svelte', 'java',\n\
  \]\n"

walk :: String
walk =
  "[walk]\n\
  \# The secret-file globs BOTH walks refuse — this measurement walk and\n\
  \# the mention universe (mention/walk.rs) — one table of NAMES, so the\n\
  \# privacy invariant plan §5.9-2 promises is spelled once (widened at\n\
  \# plan v2.17 L round, S-A9). The reach differs by design: here the\n\
  \# override set also prunes a matching DIRECTORY (a `.env/`\n\
  \# virtualenv), while the mention walk tests file basenames only.\n\
  \# Privacy fails safe: `id_*` or `*credentials*` over-matching a code\n\
  \# file costs coverage, never leaks a key into the index.\n\
  \secret_globs = [\n\
  \  '.env*', '*.pem', '*.key', 'id_*', '.npmrc', '.pypirc', '.netrc', '*credentials*',\n\
  \]\n\
  \# Built-in excludes: lockfiles, minified/generated, vendored (Lua's\n\
  \# and R's project trees since plan v2.30 step 4), snapshots, migrations\n\
  \# (plan §4.1); the secret globs above join them in build_overrides.\n\
  \# Build outputs are not globs: a `target/` or `build/` is one only\n\
  \# beside its tool's project file (outputs.rs), asked at each\n\
  \# directory's door by the walk and by `Scope` alike.\n\
  \builtin_excludes = [\n\
  \  '!package-lock.json', '!yarn.lock', '!pnpm-lock.yaml', '!Cargo.lock', '!*.min.js',\n\
  \  '!*.min.css', '!*.pb.go', '!*_pb2.py', '!*.generated.*', '!vendor/', '!node_modules/',\n\
  \  '!lua_modules/', '!renv/', '!packrat/', '!__snapshots__/', '!*.snap', '!migrations/',\n\
  \]\n"
