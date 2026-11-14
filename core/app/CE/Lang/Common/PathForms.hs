-- | The path forms two readers of a file's path take from the package
-- (release 1.10.0): the dead-code liveness role
-- (cli/src/graph/deadcode/flags.rs) and the convention word's path half
-- (cli/src/mention/conv/name.rs). Until this release each carried its
-- own Rust literals, and the two disagreed on which basenames a test
-- runner finds. The rows continue the `[flags]` table of
-- CE.Lang.Common.Graph; how a row is matched against a path stays the
-- measuring side's.
module CE.Lang.Common.PathForms where

flags3 :: String
flags3 =
  "# The basenames a language's own test runner discovers: ONE table,\n\
  \# read by both readers. A row with a `*` is `prefix * suffix`, the star\n\
  \# non-empty; a row without one is the whole basename. Each form is its\n\
  \# runner's documented discovery rule, nothing wider.\n\
  \runner_tests = [\n\
  \  # Go, https://pkg.go.dev/cmd/go (Test packages): files matching\n\
  \  # \"*_test.go\"; files beginning with \"_\" (including \"_test.go\") or\n\
  \  # \".\" are ignored. The non-empty star keeps `_test.go` out; a longer\n\
  \  # name beginning with `_` or `.` still reads as a test here, a file\n\
  \  # the toolchain never builds.\n\
  \  '* _test.go',\n\
  \  # pytest, https://docs.pytest.org/en/stable/explanation/goodpractices.html\n\
  \  # (Conventions for Python test discovery): test_*.py or *_test.py;\n\
  \  # https://docs.pytest.org/en/stable/how-to/writing_plugins.html\n\
  \  # (conftest.py: local per-directory plugins).\n\
  \  'test_ * .py', '* _test.py', 'conftest.py',\n\
  \  # Jest, https://jestjs.io/docs/configuration (testMatch default\n\
  \  # **/?(*.)+(spec|test).?([mc])[jt]s?(x): a .test or .spec suffix, and\n\
  \  # files called test.js or spec.js) and Vitest,\n\
  \  # https://vitest.dev/config/include (include default\n\
  \  # **/*.{test,spec}.?(c|m)[jt]s?(x)), on the TypeScript family's\n\
  \  # extensions. Jest's __tests__/ directory is the liveness role's\n\
  \  # directory row below.\n\
  \  '* .test.ts', '* .test.tsx', '* .test.mts', '* .test.cts',\n\
  \  '* .spec.ts', '* .spec.tsx', '* .spec.mts', '* .spec.cts',\n\
  \  'test.ts', 'test.tsx', 'test.mts', 'test.cts',\n\
  \  'spec.ts', 'spec.tsx', 'spec.mts', 'spec.cts',\n\
  \  # hspec-discover, https://hspec.github.io/hspec-discover.html: the\n\
  \  # name of a spec file has to end in Spec.hs; the test driver is\n\
  \  # test/Spec.hs (the test suite's main-is).\n\
  \  '* Spec.hs', 'Spec.hs',\n\
  \  # The C family's `_test` convention (googletest's, Unity's); Maven\n\
  \  # Surefire's default includes (a bare Test.java earns nothing, the\n\
  \  # non-empty star); busted's `_spec` and the `_test` of luaunit and its\n\
  \  # kin; testthat's `test-` / `test_` files, under either extension case.\n\
  \  '* _test.c', '* _test.cc', '* _test.cpp',\n\
  \  'Test * .java', '* Test.java', '* Tests.java', '* TestCase.java',\n\
  \  '* _spec.lua', '* _test.lua',\n\
  \  'test- * .R', 'test_ * .R', 'test- * .r', 'test_ * .r',\n\
  \]\n"

-- | The rows each reader keeps for itself: the two test-directory rows,
-- the Cargo rows, the doc entries and the ambient suffixes.
flags4 :: String
flags4 =
  "# Directory names are a project's habit, not a runner's rule, and the\n\
  \# two readers judge them differently, so each has its own row; a\n\
  \# project whose runner reads another directory declares it in\n\
  \# `[graph] entry_globs`. The liveness role's: a path component named\n\
  \# here (testthat's tests/testthat/ sits under `tests`).\n\
  \test_dirs_role = ['tests', '__tests__']\n\
  \# The convention word's: a path component named here.\n\
  \test_dirs_word = ['test', 'tests', 'spec', '__tests__', 'testdata']\n\
  \# The convention word's Cargo rows: a component named in `cargo_dirs`\n\
  \# whose parent directory is a Cargo package root (it holds Cargo.toml),\n\
  \# and a file named in `cargo_files` that is a package root's own child.\n\
  \cargo_dirs = ['benches', 'examples']\n\
  \cargo_files = ['build.rs']\n\
  \# The documentation entries the liveness role keeps: a basename in\n\
  \# `doc_names` anywhere, and a row's basenames under its directory.\n\
  \doc_names = ['README.md', 'CLAUDE.md']\n\
  \doc_dirs = [\n\
  \  ['docs/', ['index.md', 'README.md']],\n\
  \]\n\
  \# TypeScript declaration files, read by suffix: the convention word's\n\
  \# ambient bit (a declaration file states types and runs nothing).\n\
  \ambient_suffixes = ['.d.ts', '.d.mts', '.d.cts']\n"
