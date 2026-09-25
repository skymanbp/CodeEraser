//! The `Protocol` bit's name tables (sealed criterion §3.2, the NAME ×
//! language half of name.rs): the framework names a loader spells for
//! the author — Python unittest/xunit/pluggy/Django/reflection
//! prefixes, TS filename × export-name conventions, Haskell autogen
//! modules and hspec, the C-family entries a linker, a JVM or a
//! language runtime looks up by name, the Java methods the platform
//! calls, the Lua metamethods and host callbacks, the R app entry
//! points — so a declaration carrying one is reached though no file
//! names it. Split from name.rs when plan v2.30 step 4's Lua and R
//! tables took it past 300 lines; the path half (test files, the
//! runner table) stays there. A leaf: name.rs reads it, never the
//! reverse. Computed at wire time and stored nowhere, like every row
//! of name.rs.

use crate::scan::lang::Lang;

/// Python names a loader spells: unittest's discovered hooks, pytest's
/// xunit-style hooks, Django's loader targets. Prefixes follow.
const PY_NAMES: &str = "setUp tearDown setUpClass tearDownClass setUpModule tearDownModule \
                        asyncSetUp asyncTearDown load_tests runTest \
                        setup teardown setup_module teardown_module setup_function \
                        teardown_function setup_class teardown_class setup_method teardown_method \
                        Command Migration";
/// pluggy hooks and the fixed-prefix reflection Django/DRF perform
/// (`clean_<field>`, `validate_<field>`, `perform_<action>`).
const PY_PREFIXES: [&str; 4] = ["pytest_", "clean_", "validate_", "perform_"];

/// TS/TSX file form × export names, one line per form group: the
/// stem (basename minus extension) on the left, the function- or
/// class-capable exports a framework loads by name on the right —
/// Next App Router route handlers and segment hooks, SvelteKit
/// endpoints, page/layout loads and hooks, Next middleware and
/// instrumentation. Constant-form exports (`metadata`, `prerender`,
/// `actions`, `config`) are out of the domain by §3.1 and not rows.
const TS_BY_STEM: &str = "\
route : GET POST PUT PATCH DELETE HEAD OPTIONS
+server : GET POST PUT PATCH DELETE HEAD OPTIONS fallback
page layout template default loading error not-found global-error : generateStaticParams generateMetadata generateViewport
opengraph-image twitter-image icon apple-icon sitemap : generateImageMetadata generateSitemaps
middleware : middleware
instrumentation : register onRequestError
+page +page.server +layout +layout.server : load entries
hooks hooks.server hooks.client : handle handleError handleFetch init reroute";
/// Directory-scoped forms: Next/Astro `pages/**` (data fetchers and
/// endpoint verbs), Remix `routes/**` and its `root` module.
const TS_PAGES: &str =
    "getStaticProps getServerSideProps getStaticPaths GET POST PUT PATCH DELETE HEAD OPTIONS ALL";
const TS_ROUTES: &str = "loader action meta links headers ErrorBoundary HydrateFallback \
                         shouldRevalidate clientLoader clientAction";

/// Java methods the platform or a container calls for the author (plan
/// v2.30 step 3, booklet §9): the Object and Comparable contracts, the
/// functional interfaces, iteration, serialization's reflected hooks,
/// cloning and finalization, the enum's synthesized pair, and the
/// servlet lifecycle. `main` is `Main`.
const JAVA_NAMES: &str = "toString equals hashCode compareTo compare run call get accept apply test \
                          close iterator hasNext next readObject writeObject readResolve \
                          writeReplace finalize clone valueOf values doGet doPost doPut doDelete \
                          init destroy service";

/// C / C++ names a loader, the linker or a language runtime spells for
/// the author (plan v2.30 step 2): the Windows DLL and program entries,
/// the bare-metal entry, the JNI, Node-API and libFuzzer hooks. `main`
/// is `Main`, the category the criterion keeps for it. Prefixes follow:
/// CPython, Lua and JNI native modules are looked up by a prefixed name.
const C_NAMES: &str = "DllMain WinMain wWinMain wmain _start JNI_OnLoad JNI_OnUnload \
                       napi_register_module_v1 LLVMFuzzerTestOneInput LLVMFuzzerInitialize";
const C_PREFIXES: [&str; 3] = ["PyInit_", "luaopen_", "Java_"];

/// Lua names the runtime or a host calls for the author (plan v2.30
/// step 4): the metamethods the manual lists (§2.4) and the ones the
/// standard libraries read (`__name`, `__pairs`, `__metatable`,
/// `__mode`), and the entry points a Neovim plugin manager calls.
/// Reached as `M.setup` too: a member name is judged by its last
/// segment (mention/name.rs).
const LUA_NAMES: &str = "__index __newindex __call __tostring __eq __lt __le __add __sub \
                         __mul __div __mod __pow __unm __idiv __band __bor __bxor __shl \
                         __shr __bnot __concat __len __gc __close __mode __name \
                         __metatable __pairs setup config on_attach";
/// LOVE's callbacks, called by name for the `love` table that
/// `main.lua` and `conf.lua` fill in.
const LOVE_NAMES: &str = "load update draw keypressed keyreleased mousepressed \
                          mousereleased mousemoved wheelmoved textinput resize focus \
                          quit conf";
/// R names a host calls: a Shiny app's `server` and `ui` (and the old
/// `shinyServer` / `shinyUI` spelling) and golem's `run_app`. The hooks
/// R itself calls (`.onLoad`, `.onAttach`, `.First`) start with a dot
/// and never enter the mention domain (mention/name.rs).
const R_NAMES: &str = "server ui shinyServer shinyUI run_app";

/// Whether `lang`'s loader, runtime or host calls `name` for the
/// author in the file at `rel`: a name in the language's table, a name
/// behind one of its prefixes, or a row the file's own name or place
/// decides.
pub(super) fn protocol(lang: Lang, rel: &str, name: &str) -> bool {
    let (names, prefixes) = tables(lang);
    listed(names, name) || prefixes.iter().any(|p| starred(name, p, "")) || by_file(lang, rel, name)
}

/// A language's name table and name prefixes (the consts above).
fn tables(lang: Lang) -> (&'static str, &'static [&'static str]) {
    match lang {
        Lang::Python => (PY_NAMES, &PY_PREFIXES),
        Lang::C | Lang::Cpp => (C_NAMES, &C_PREFIXES),
        Lang::Java => (JAVA_NAMES, &[]),
        Lang::Lua => (LUA_NAMES, &[]),
        Lang::R => (R_NAMES, &[]),
        _ => ("", &[]),
    }
}

/// The rows a file's own name or place decides: the TS file forms,
/// Haskell's generated modules and hspec's `spec`, and LÖVE's callbacks
/// in the two files it fills its `love` table from.
fn by_file(lang: Lang, rel: &str, name: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    match lang {
        Lang::TypeScript | Lang::Tsx => ts_protocol(rel, base, name),
        Lang::Haskell => {
            base.starts_with("Paths_")
                || base.starts_with("PackageInfo_")
                || (name == "spec" && starred(base, "", "Spec.hs"))
        }
        Lang::Lua => matches!(base, "main.lua" | "conf.lua") && listed(LOVE_NAMES, name),
        _ => false,
    }
}

/// Whether a whitespace-separated name table holds `name` — the shape
/// every name list here takes, and the dead-code entry names too
/// (graph/deadcode/flags.rs).
pub(crate) fn listed(table: &str, name: &str) -> bool {
    table.split_whitespace().any(|n| n == name)
}

/// `<prefix>*<suffix>` with `*` non-empty — the one pattern the name
/// tables and name.rs's path half both read, here on the leaf side.
pub(super) fn starred(base: &str, prefix: &str, suffix: &str) -> bool {
    base.len() > prefix.len() + suffix.len() && base.starts_with(prefix) && base.ends_with(suffix)
}

/// Filename form × export name: the stem rows, then the directory
/// rows (`pages/**`; Remix `routes/**` and `app/root.*`).
fn ts_protocol(rel: &str, base: &str, name: &str) -> bool {
    let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
    let by_stem = TS_BY_STEM.lines().any(|row| {
        let (stems, names) = row.split_once(" : ").expect("a `stems : names` row");
        listed(stems, stem) && listed(names, name)
    });
    let in_dir = |d: &str| rel.split('/').rev().skip(1).any(|c| c == d);
    by_stem
        || (in_dir("pages") && listed(TS_PAGES, name))
        || ((in_dir("routes") || (stem == "root" && in_dir("app"))) && listed(TS_ROUTES, name))
}
