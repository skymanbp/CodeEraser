# Addendum 4 to BARS.md — rule 1 of section 3: the citations and the position rules

Written 2026-11-11 08:41 local by the main session. BARS.md (sha256 2934ab1c…a29fa65) and addenda 1–3 are not
edited. This file supplies the one input BARS §3 rule 1 asks for and relaxes no bar.

## What was known at this moment

- RESULT-2A's class counts per language (K1 primary 105 / 246 / 73) — no row's class.
- The first-reading totals (392 / 28 / 4), the agreement table (94 / 98), the seven rows whose two readings
  differ and their settlement (`SETTLEMENT.md`). Among them: `zod packages/docs-v3/CONTRIBUTING.md`, settled
  `unsure`; and, printed by the same search, the answer for `zod CONTRIBUTING.md` (`in_use`, forge convention).
- From addendum 1 C: one row under Maven's test-resources directory was answered `unused` at first reading.
- Nothing else about which audited rows carry which candidate, and no §3 or §4 table: none has been computed.

Because two of these facts touch candidates, every position rule below is taken from the tool's documentation
alone, quoted, and the consequence the main session can already see is stated next to it.

## How a position rule is derived (one reading, applied to all twelve)

BARS §3 rule 1: the row "encodes the position rule the tool uses (a name the tool reads only at one place is not
entered as a bare basename)". The documentation fixes a position in one of two ways:

- (F) relative to the repository or to a named directory — then the rule is exactly those places;
- (I) relative to the invocation (the directory the tool runs in, or a directory given on its command line) —
  then the documentation fixes no place inside a repository: any directory may be that project root, and the
  rule is "the name, in any directory". Rule 2 (every audited carrying row `in_use`) is the audit's check on it.

Where the documentation is silent or contradicts itself, the narrower reading is taken and the gap is named.

## The twelve candidates

Sources were fetched on 2026-11-11 by a read-only research agent (`convention_docs_research.md`) and every quote
below was re-checked by the main session against the page or the documentation source file itself (raw files of
the tools' own repositories at their default branch; Oracle and Apache pages as served).

1. **`package-info.java`** — javac / JLS SE 25 §7.4.1, https://docs.oracle.com/javase/specs/jls/se25/html/jls-7.html :
   "The sole annotated `package` declaration, if it exists, is placed in a source file called `package-info.java`
   in the directory containing the source files for the package." javac man page (`-Xpkginfo`): "Generates a
   `package-info.class` file for every `package-info.java` file."
   Position (F, a named kind of directory at any depth): **the basename, in any directory.**
2. **`module-info.java`** — javac man page, https://docs.oracle.com/en/java/javase/25/docs/specs/man/javac.html :
   "each module has a source file called `module-info.java`"; "the file named `module-info.java` in one of the
   locations on the source path or one the module source path for the module."
   Position (I, the source path is an argument): **the basename, in any directory.**
3. **`vitest.config.*`** — Vitest, https://vitest.dev/config/ (source `docs/config/index.md`): "When an explicit
   `--config` option is not provided, Vitest looks for `vitest.config.{ts,mts,cts,js,mjs,cjs}` first and
   `vite.config.{ts,mts,cts,js,mjs,cjs}` second in the project `root`."
   Position (I): **basename exactly one of `vitest.config.ts`, `.mts`, `.cts`, `.js`, `.mjs`, `.cjs`, in any
   directory.** Not the prefix `vitest.config.`: the documentation lists six names and says JSON is not read.
   `vite.config.*` is not a registered candidate and is not added.
4. **Next.js App Router `page.tsx` / `layout.tsx` / `route.ts`** — Next.js,
   https://nextjs.org/docs/app/getting-started/project-structure (source `02-project-structure.mdx`): the
   routing-files table gives `layout` and `page` the extensions `.js` `.jsx` `.tsx` and `route` `.js` `.ts`; "A
   route becomes public when a `page` or `route` file exists."; private folders: "prefixing a folder with an
   underscore … opting the folder and all its subfolders out of routing."; `src` folder page: "move the `app`
   Router folder … to `src/app`".
   Position (F, a named directory; the project root itself is (I)): **basename one of the three registered names,
   some directory segment is `app`, and no directory segment after that `app` begins with `_`.**
   Not encoded (named, narrower readings not needed by any registered name): `src/app` being ignored when a
   root `app` exists.
5. **`setup.py`** — pip, https://pip.pypa.io/en/stable/cli/pip_install/ ("Argument Handling"): "Local directory
   (which must contain a `pyproject.toml` or `setup.py`, otherwise pip will report an error)."
   Position (I): **the basename, in any directory.**
6. **Sphinx `conf.py`** — Sphinx, https://www.sphinx-doc.org/en/master/man/sphinx-build.html : "sphinx-build looks
   for `<sourcedir>/conf.py` for the configuration settings."; `-c path`: "Don't look for the `conf.py` in the
   source directory, but use the given configuration directory instead."
   Position (I): **the basename, in any directory.** The lane's first encoding (`docs/conf.py`, `doc/conf.py`) is a
   habit of projects, not a sentence of the documentation, and is withdrawn. Stated plainly: `conf.py` is a
   generic name, so this is the broadest of the twelve rules; if the candidate passes, the §3 table prints its
   carrying rows by name and the owner sees that line.
7. **`tlconfig.lua`** — Teal, https://github.com/teal-language/tl/blob/master/docs/src/compiler_options.md :
   "the compiler will try to read the compilation options from a file called `tlconfig.lua` inside the current
   working directory."
   Position (I): **the basename, in any directory.**
8. **busted `spec/`** — busted. The tool's own option help, as defined in
   https://github.com/lunarmodules/busted/blob/master/busted/modules/cli.lua : ROOT — "test script file/folder.
   Folders will be traversed for any file that matches the --pattern option.", default `'spec'`; `--pattern` —
   "only run test files matching the Lua pattern", default `'_spec'`; `--[no-]recursive` default true. The
   rendered site https://lunarmodules.github.io/busted/ prints the same two help sentences with the pattern
   default `_spec`; it prints the ROOT default as `nil` (the site is behind the source). The loader
   (`test_file_loader.lua`) keeps a file when `basename:find(patt)`.
   Position (F under a named directory; that directory's parent is (I)): **some directory segment is `spec` AND
   the basename contains `_spec`.** A file under `spec/` whose name does not contain `_spec` is not read by the
   runner by name (a spec may `require` it — then it is referenced, and is not in the audited universe at all).
   The lane's first encoding (any file under a `spec` segment) is withdrawn. Gap, named: the default ROOT is in
   the tool's help definition, not in a prose page.
9. **Maven `src/test/resources`** — Apache Maven Super POM,
   https://maven.apache.org/ref/current/maven-model-builder/super-pom.html :
   `<testResource><directory>${project.basedir}/src/test/resources</directory></testResource>`; resources plugin,
   https://maven.apache.org/plugins/maven-resources-plugin/testResources-mojo.html : "Copy resources for the test
   source code to the test output directory."
   Position (F under a module's base directory, which is any directory holding a `pom.xml`): **the path contains
   the directory sequence `src/test/resources/` at a segment boundary.**
   Already visible to the main session: one carrying row was answered `unused`, so rule 2 will refuse this
   candidate whatever its position rule; the rule is still written as the documentation gives it.
10. **`CONTRIBUTING.md`** — GitHub Docs, "Setting guidelines for repository contributors",
    https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors :
    "you can add a file with contribution guidelines to your project repository's root, `docs`, or `.github`
    folder. When someone opens a pull request or creates an issue, they will see a link to that file."; "Contributing
    guidelines filenames are not case sensitive."
    Position (F): **the repository-relative path is exactly `CONTRIBUTING.md`, `docs/CONTRIBUTING.md` or
    `.github/CONTRIBUTING.md`, compared without case.**
    Already visible: under the lane's first encoding (bare basename) the row `packages/docs-v3/CONTRIBUTING.md`
    (settled `unsure`) would carry the candidate and refuse it by rule 2; under the documented places it does not
    carry it. The documented places are rule 1's letter; the §3 table prints the bare-basename reading beside it
    for this candidate so the difference is on the page, and the documented reading is the one evaluated.
11. **`SECURITY.md`** — GitHub Docs, "Creating a default community health file",
    https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file :
    the supported-files table lists `SECURITY.md`; "For supported files that can be stored in more than one
    location, GitHub uses the following order of precedence: The `.github` folder / The root of the repository /
    The `docs` folder". (The page that used to say "add a `SECURITY.md` file to your repository's root, `docs`, or
    `.github` folder" was moved on 2026-01-13 and its current text no longer states the places; the generic
    sentence above is the current citation.)
    Position (F): **exactly `SECURITY.md`, `docs/SECURITY.md` or `.github/SECURITY.md`.**
12. **`CODE_OF_CONDUCT.md`** — GitHub Docs, "Adding a code of conduct to your project",
    https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/adding-a-code-of-conduct-to-your-project :
    "GitHub looks for a code of conduct in the `.github` directory, then the root of the repository, then the
    `docs` directory, and uses the first file it finds."
    Position (F): **exactly `CODE_OF_CONDUCT.md`, `docs/CODE_OF_CONDUCT.md` or `.github/CODE_OF_CONDUCT.md`.**

## The closed set of rule kinds the table needs

`anywhere <basename>…` (1, 2, 3, 5, 6, 7) · `places <path>…` with an optional no-case flag (10 no-case; 11, 12) ·
`sequence <dir/dir/dir/>` (9) · `segment-name <segment> <substring of the basename>` (8) ·
`app-router <basename>…` (4: a segment `app`, no later segment beginning with `_`).

## The machine form

`convention_docs.json`, written 2026-11-11 09:15 local from the text above and from nothing else, in the shape of
the lane's reader (`conventions.rs` at tests c7df2aa: `exact` / `basename` / `under`, `*` inside a name). Every
quote in it is a verbatim run of the cited page's source (markup kept: `{% data variables.product.github %}` is
the documentation's variable for "GitHub"); `check_convention_docs.py` finds all nineteen in the fetched copies.

| rule kind above | machine form | reader |
|---|---|---|
| `anywhere` (1, 2, 3, 5, 6, 7) | `basename` + `names` | as built |
| `places` (11, 12) | `exact` + `paths` | as built |
| `places` without case (10) | `exact` + `paths` + `"nocase": true` | **to add** |
| `segment-name` (8) | `under`, `segment: spec`, `names: ["*_spec*"]` | as built |
| `app-router` (4) | `under`, `segment: app`, three names, `"no_later_prefix": "_"` | **to add** |
| `sequence` (9) | `{"kind": "sequence", "dirs": ["src", "test", "resources"]}` | **to add** |

Three reader additions are therefore asked of the lane, each with its own unit rows; none widens a rule.

One precision made while writing the machine form, by the rule "the narrower reading is taken": candidate 4 says
"some directory segment is `app`". With two segments named `app` in one path the documentation does not say
which is the router folder; the machine form takes the FIRST, and no directory segment after it may begin with
`_`. Known at this moment about candidate 4's rows: only that zod has three dead files named `page.tsx` (the
lane's producer proof printed the count, not the paths).

Candidate 10 carries a second rule under the key `beside` (the bare basename): the table prints that reading next
to the documented one and evaluates only the documented one. Candidate 6 carries `print_rows`.

The three project conventions of BARS §3 stay out by rule 1 whatever this file says.

## What this addendum does not do

It does not add a candidate, drop one, or change rule 2 or rule 3. A candidate whose citation is called a gap
above (8: the default root from the help definition) is still evaluated; the gap is printed with it.
