-- | The protocol name tables (plan v2.32 step 1),
-- transcribed from cli/src/mention/conv/protocol.rs at a378e78c; from this
-- commit on the core is the authority.
module CE.Lang.Common.Protocol where

protocol :: String
protocol =
  "[protocol]\n\
  \# Python names a loader spells: unittest's discovered hooks, pytest's\n\
  \# xunit-style hooks, Django's loader targets. Prefixes follow.\n\
  \py_names = [\n\
  \  'setUp', 'tearDown', 'setUpClass', 'tearDownClass', 'setUpModule', 'tearDownModule',\n\
  \  'asyncSetUp', 'asyncTearDown', 'load_tests', 'runTest', 'setup', 'teardown',\n\
  \  'setup_module', 'teardown_module', 'setup_function', 'teardown_function', 'setup_class',\n\
  \  'teardown_class', 'setup_method', 'teardown_method', 'Command', 'Migration',\n\
  \]\n\
  \# pluggy hooks and the fixed-prefix reflection Django/DRF perform\n\
  \# (`clean_<field>`, `validate_<field>`, `perform_<action>`).\n\
  \py_prefixes = [\n\
  \  'pytest_', 'clean_', 'validate_', 'perform_',\n\
  \]\n\
  \# TS/TSX file form × export names, one line per form group: the\n\
  \# stem (basename minus extension) on the left, the function- or\n\
  \# class-capable exports a framework loads by name on the right —\n\
  \# Next App Router route handlers and segment hooks, SvelteKit\n\
  \# endpoints, page/layout loads and hooks, Next middleware and\n\
  \# instrumentation. Constant-form exports (`metadata`, `prerender`,\n\
  \# `actions`, `config`) are out of the domain by §3.1 and not rows.\n\
  \ts_by_stem = [\n\
  \  [['route'], ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']],\n\
  \  [['+server'], ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS', 'fallback']],\n\
  \  [['page', 'layout', 'template', 'default', 'loading', 'error', 'not-found', 'global-error'], ['generateStaticParams', 'generateMetadata', 'generateViewport']],\n\
  \  [['opengraph-image', 'twitter-image', 'icon', 'apple-icon', 'sitemap'], ['generateImageMetadata', 'generateSitemaps']],\n\
  \  [['middleware'], ['middleware']],\n\
  \  [['instrumentation'], ['register', 'onRequestError']],\n\
  \  [['+page', '+page.server', '+layout', '+layout.server'], ['load', 'entries']],\n\
  \  [['hooks', 'hooks.server', 'hooks.client'], ['handle', 'handleError', 'handleFetch', 'init', 'reroute']],\n\
  \]\n\
  \# Directory-scoped forms: Next/Astro `pages/**` (data fetchers and\n\
  \# endpoint verbs), Remix `routes/**` and its `root` module.\n\
  \ts_pages = [\n\
  \  'getStaticProps', 'getServerSideProps', 'getStaticPaths', 'GET', 'POST', 'PUT', 'PATCH',\n\
  \  'DELETE', 'HEAD', 'OPTIONS', 'ALL',\n\
  \]\n\
  \ts_routes = [\n\
  \  'loader', 'action', 'meta', 'links', 'headers', 'ErrorBoundary', 'HydrateFallback',\n\
  \  'shouldRevalidate', 'clientLoader', 'clientAction',\n\
  \]\n"

protocol2 :: String
protocol2 =
  "# Java methods the platform or a container calls for the author (plan\n\
  \# v2.30 step 3, booklet §9): the Object and Comparable contracts, the\n\
  \# functional interfaces, iteration, serialization's reflected hooks,\n\
  \# cloning and finalization, the enum's synthesized pair, and the\n\
  \# servlet lifecycle. `main` is `Main`.\n\
  \java_names = [\n\
  \  'toString', 'equals', 'hashCode', 'compareTo', 'compare', 'run', 'call', 'get',\n\
  \  'accept', 'apply', 'test', 'close', 'iterator', 'hasNext', 'next', 'readObject',\n\
  \  'writeObject', 'readResolve', 'writeReplace', 'finalize', 'clone', 'valueOf', 'values',\n\
  \  'doGet', 'doPost', 'doPut', 'doDelete', 'init', 'destroy', 'service',\n\
  \]\n\
  \# C / C++ names a loader, the linker or a language runtime spells for\n\
  \# the author (plan v2.30 step 2): the Windows DLL and program entries,\n\
  \# the bare-metal entry, the JNI, Node-API and libFuzzer hooks. `main`\n\
  \# is `Main`, the category the criterion keeps for it. Prefixes follow:\n\
  \# CPython, Lua and JNI native modules are looked up by a prefixed name.\n\
  \c_names = [\n\
  \  'DllMain', 'WinMain', 'wWinMain', 'wmain', '_start', 'JNI_OnLoad', 'JNI_OnUnload',\n\
  \  'napi_register_module_v1', 'LLVMFuzzerTestOneInput', 'LLVMFuzzerInitialize',\n\
  \]\n\
  \c_prefixes = [\n\
  \  'PyInit_', 'luaopen_', 'Java_',\n\
  \]\n\
  \# Lua names the runtime or a host calls for the author (plan v2.30\n\
  \# step 4): the metamethods the manual lists (§2.4) and the ones the\n\
  \# standard libraries read (`__name`, `__pairs`, `__metatable`,\n\
  \# `__mode`), and the entry points a Neovim plugin manager calls.\n\
  \# Reached as `M.setup` too: a member name is judged by its last\n\
  \# segment (mention/name.rs).\n\
  \lua_names = [\n\
  \  '__index', '__newindex', '__call', '__tostring', '__eq', '__lt', '__le', '__add',\n\
  \  '__sub', '__mul', '__div', '__mod', '__pow', '__unm', '__idiv', '__band', '__bor',\n\
  \  '__bxor', '__shl', '__shr', '__bnot', '__concat', '__len', '__gc', '__close', '__mode',\n\
  \  '__name', '__metatable', '__pairs', 'setup', 'config', 'on_attach',\n\
  \]\n\
  \# LOVE's callbacks, called by name for the `love` table that\n\
  \# `main.lua` and `conf.lua` fill in.\n\
  \love_names = [\n\
  \  'load', 'update', 'draw', 'keypressed', 'keyreleased', 'mousepressed', 'mousereleased',\n\
  \  'mousemoved', 'wheelmoved', 'textinput', 'resize', 'focus', 'quit', 'conf',\n\
  \]\n"

protocol3 :: String
protocol3 =
  "# R names a host calls: a Shiny app's `server` and `ui` (and the old\n\
  \# `shinyServer` / `shinyUI` spelling) and golem's `run_app`. The hooks\n\
  \# R itself calls (`.onLoad`, `.onAttach`, `.First`) start with a dot\n\
  \# and never enter the mention domain (mention/name.rs).\n\
  \r_names = [\n\
  \  'server', 'ui', 'shinyServer', 'shinyUI', 'run_app',\n\
  \]\n"
