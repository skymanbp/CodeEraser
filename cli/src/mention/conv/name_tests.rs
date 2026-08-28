//! The name half's K37 witnesses: every path rule, every protocol
//! table row family, the Go receiver pipeline with its four empty
//! exits and the three bit-6/7 counterfactuals, and the file-level
//! allow claim. Table-driven as `path key ⇒ letters` lines in the
//! shared alphabet (`bit_of`, mod.rs): `m` Main, `T` Test, `P`
//! Protocol, `d` MemberDispatch, `a` MemberApi, `A` Ambient, `L`
//! Allow, `-` none. ONE text table, not an array of case strings: a
//! run of string literals is this repo's most-rhyming token shape,
//! and the clone gate paired the first draft's array with the
//! AST-half table and the self-mention table alike.

use super::{PathWords, name_bits, text_bits};
use crate::mention::conv::{Conv, bit_of};
use crate::mention::name::mention_name;
use crate::scan::lang::Lang;
use crate::testutil::{scratch, write_tree};
use std::path::Path;

/// `path key ⇒ letters`: the path bits, the name × key bits and — when
/// the key is a whole file marked `@allow` — the text bit, OR'd. A
/// `#` line is commentary; the tree holds one Cargo package at `pkg/`.
const CASES: &str = "\
# path half: components, package-root-qualified dirs, the equal
# group, the pattern group with `*` non-empty
src/test/a.rs f/0 ⇒ T
tests/a.py f/0 ⇒ T
x/spec/a.ts f/0 ⇒ T
x/__tests__/a.ts f/0 ⇒ T
x/testdata/a.go f/0 ⇒ T
src/mytests/a.rs f/0 ⇒ -
src/latestdata/a.rs f/0 ⇒ -
protest/a.rs f/0 ⇒ -
pkg/benches/b.rs f/0 ⇒ T
pkg/examples/e.rs f/0 ⇒ T
other/benches/b.rs f/0 ⇒ -
pkg/build.rs f/0 ⇒ T
pkg/src/build.rs f/0 ⇒ -
pkg/src/trend_rebuild.rs f/0 ⇒ -
conftest.py f/0 ⇒ T
x/Spec.hs f/0 ⇒ T
x/FooSpec.hs f/0 ⇒ T
x/a_test.go f/0 ⇒ T
x/_test.go f/0 ⇒ -
x/a.test.ts f/0 ⇒ T
x/.test.ts f/0 ⇒ -
x/.test.helper.test.ts f/0 ⇒ T
x/a.spec.tsx f/0 ⇒ T
x/test_a.py f/0 ⇒ T
x/test_.py f/0 ⇒ -
x/a_test.py f/0 ⇒ T
x/types.d.ts f/0 ⇒ A
x/types.d.mts f/0 ⇒ A
x/typesd.ts f/0 ⇒ -
# Main: Python and Haskell only (Rust `fn main` is out of the domain
# without `pub`; Go's lowercase is out)
a.py main/0 ⇒ m
a.hs main/0 ⇒ m
a.rs main/0 ⇒ -
a.go main/0 ⇒ -
# Python protocol: unittest, xunit, pluggy and reflection prefixes
# with a non-empty tail, Django loader targets
a.py tearDownModule/0 ⇒ P
a.py setup_method/1 ⇒ P
a.py load_tests/3 ⇒ P
a.py pytest_addoption/1 ⇒ P
a.py pytest_/1 ⇒ -
a.py clean_email/1 ⇒ P
a.py perform_create/1 ⇒ P
a.py Command ⇒ P
a.py setup_thing/0 ⇒ -
# TS protocol: filename form × export name, directory forms; a bare
# `GET` in a non-route file is not exempt
app/api/route.ts GET/1 ⇒ P
app/x/page.tsx generateMetadata/1 ⇒ P
app/x/page.tsx generateStaticParams/0 ⇒ P
src/routes/+server.ts fallback/1 ⇒ P
src/routes/+page.server.ts load/1 ⇒ P
src/hooks.server.ts handle/1 ⇒ P
middleware.ts middleware/1 ⇒ P
instrumentation.ts register/0 ⇒ P
pages/x.tsx getStaticProps/1 ⇒ P
src/pages/api.ts GET/1 ⇒ P
app/routes/x.tsx loader/1 ⇒ P
app/routes/x.tsx ErrorBoundary/0 ⇒ P
app/root.tsx links/0 ⇒ P
lib/util.ts GET/1 ⇒ -
app/x/page.tsx metadata ⇒ -
# Haskell: autogen modules, hspec `spec` in a `*Spec.hs` (the path
# half fires beside it)
Paths_ce_core.hs version/0 ⇒ P
PackageInfo_x.hs synopsis/0 ⇒ P
test/FooSpec.hs spec/0 ⇒ TP
FooSpec.hs spec/0 ⇒ TP
Foo.hs spec/0 ⇒ -
# Go receiver pipeline: value/pointer, generic, package-qualified,
# the exported-method-on-unexported-receiver counterfactual, and the
# four empty exits plus the anonymous key
a.go (T) add/1 ⇒ a
a.go (*Command) UsagePadding/0 ⇒ a
a.go (*pkg.Cache[K, V]) M/0 ⇒ a
a.go (commandSorterByName) Len/0 ⇒ d
a.go (*flagCompError) Error/0 ⇒ d
a.go (_t) M/0 ⇒ d
a.go () M/0 ⇒ -
a.go (*) M/0 ⇒ -
a.go ([K]) M/0 ⇒ -
a.go (Cache.) M/0 ⇒ -
a.go (anonymous)/0 ⇒ -
a.go free/1 ⇒ -
a.rs impl Foo for fn(u8) -> u8 ⇒ -
";

#[test]
fn every_name_half_row_fires_on_its_witness_and_nowhere_else() {
    let root = scratch("conv-name");
    write_tree(&root, &[("pkg/Cargo.toml", "[package]\nname = \"p\"\n")]);
    let mut words = PathWords::new(&root);
    for case in CASES.lines().filter(|l| !l.starts_with('#')) {
        let (input, letters) = case.rsplit_once(" ⇒ ").expect("case has ` ⇒ `");
        let (rel, key) = input.split_once(' ').expect("path then key");
        let lang = Lang::judged_path(Path::new(rel)).expect("a judged path");
        let name = mention_name(rel, key).unwrap_or_default();
        let got = words.bits(rel) | name_bits(lang, rel, key, &name);
        let want: i64 = letters.chars().map(bit_of).sum();
        assert_eq!(got, want, "{case:?}");
    }
    std::fs::remove_dir_all(&root).ok();
}

/// The package-root stat is one per directory: a second file in the
/// same directory answers off the memo (the count cannot be observed
/// through the filesystem, so the memo's key set is the witness).
#[test]
fn package_roots_are_stat_once_per_directory() {
    let root = scratch("conv-name-memo");
    write_tree(&root, &[("Cargo.toml", "[package]\nname = \"p\"\n")]);
    let mut words = PathWords::new(&root);
    assert_eq!(words.bits("benches/a.rs"), Conv::Test.bit());
    assert_eq!(words.bits("benches/b.rs"), Conv::Test.bit());
    assert_eq!(words.bits("build.rs"), Conv::Test.bit());
    assert_eq!(words.pkg_roots.len(), 1, "one directory, one stat");
    std::fs::remove_dir_all(&root).ok();
}

/// Only a why-bearing claim exempts, and it exempts by file.
#[test]
fn the_allow_claim_is_file_level_and_needs_its_why() {
    assert_eq!(
        text_bits("// ce:allow(unmentioned) -- reached by name\nfn x() {}\n"),
        Conv::Allow.bit()
    );
    assert_eq!(text_bits("// ce:allow(unmentioned)\nfn x() {}\n"), 0);
    assert_eq!(text_bits("// ce:allow(deadcode) -- why\n"), 0);
}
