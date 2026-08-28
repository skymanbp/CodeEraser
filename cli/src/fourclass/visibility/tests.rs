//! The word per language, measured at the extractor's own root so the
//! test sees what the producer sees. Haskell's legs live in
//! tests_hs.rs (the export-list lexer has a battery of its own).

use super::*;
use crate::fourclass::units;
use crate::scan::functions;
use crate::scan::spec;

/// (unit name, visibility word) for every function-shaped declaration.
pub(super) fn fns(src: &str, lang: Lang) -> Vec<(String, i64)> {
    let tree = ast::parse_lang(src, lang).expect("parses");
    let bytes = src.as_bytes();
    functions::extract(tree.root_node(), bytes, spec::spec(lang))
        .into_iter()
        .map(|f| (f.name.clone(), bits(f.node, bytes, lang)))
        .collect()
}

/// One case as one line (testutil::check_word_case): every unit as
/// `name:bits` — `E` exported, `S` scope-exported, `R` restricted,
/// `-` none.
pub(super) fn check(lang: Lang, case: &str) {
    crate::testutil::check_word_case(lang, case, fns, bit_of);
}

/// The same line shape over EVERY unit the register extracts (the
/// named type forms beside the functions), keyed as stored — `fns`
/// sees the functions alone.
pub(super) fn check_units(lang: Lang, case: &str) {
    let all = |src: &str, lang: Lang| {
        // source order: the register walks its extras off a stack
        let mut units = units::segments(src, lang);
        units.sort_by(|a, b| (a.start_line, &a.key).cmp(&(b.start_line, &b.key)));
        units.into_iter().map(|u| (u.key, u.vis)).collect()
    };
    crate::testutil::check_word_case(lang, case, all, bit_of);
}

/// L round step 8 (O55): Go's named type forms are units, read by the
/// same initial-capital rule as a function, and a type declared inside
/// a function body is still keyed (a local, like Rust's) — exported by
/// its case alone (bit 0), never by its scope (bit 1: the step-8
/// review found the Go arm storing both on a body-local `type`).
#[test]
fn go_type_forms_are_units_with_their_own_word() {
    check_units(
        Lang::Go,
        "package p\ntype Pub struct{}\ntype priv = Pub\nfunc F() { type inner int; type Inner int } \
         ⇒ Pub:ES priv:- F/0:ES Inner:E inner:-",
    );
}

fn bit_of(letter: char) -> i64 {
    match letter {
        'E' => VIS_EXPORTED,
        'S' => VIS_SCOPE_EXPORTED,
        'R' => VIS_RESTRICTED,
        '-' => 0,
        other => panic!("unknown bit letter {other:?}"),
    }
}

/// Each language's own rule on a top-level declaration: the export
/// mechanism names it or it does not, and at top level bit 1 follows.
#[test]
fn exported_bits_follow_each_language_declaration_rule() {
    check(
        Lang::Rust,
        "pub fn open() {}\nfn shut() {}\npub(crate) fn near() {}\npub(self) fn own() {}\n\
         pub(in crate::open) fn path() {} ⇒ open:ES shut:- near:ESR own:ESR path:ESR",
    );
    check(
        Lang::TypeScript,
        "export function open() {}\nfunction shut() {} ⇒ open:ES shut:-",
    );
    check(
        Lang::Python,
        "def open():\n    pass\ndef _shut():\n    pass ⇒ open:ES _shut:-",
    );
    check(
        Lang::Go,
        "package p\nfunc Open() {}\nfunc shut() {} ⇒ Open:ES shut:-",
    );
}

/// K21 producer half: a `pub fn` in a private `mod` is exported by
/// its own declaration (bit 0) and not by its scope (bit 1); the chain
/// must be plain `pub` at every level, and a function body closes it.
#[test]
fn rust_scope_bit_reads_the_inline_mod_chain() {
    check(
        Lang::Rust,
        "mod hidden { pub fn h() {} }\n\
         pub mod shown { pub fn s() {} pub(crate) mod cm { pub fn c() {} } }\n\
         pub(super) fn up() {}\n\
         fn outer() { pub fn inner() {} } ⇒ h:E s:ES c:E up:ESR outer:- inner:E",
    );
}

/// K28 producer half: a nested def and a `_Private` class's methods
/// keep their own bit 0 and lose bit 1; a public class's public
/// method keeps both.
#[test]
fn python_scope_bit_reads_defs_and_class_names() {
    check(
        Lang::Python,
        "def top():\n    def inner():\n        pass\n\
         class _Impl:\n    def method(self):\n        pass\n\
         class Pub:\n    def method(self):\n        pass\n    def _hidden(self):\n        pass\
         \n ⇒ top:ES inner:E method:E method:ES _hidden:-",
    );
}

/// L round step 8 (O56): a literal `__all__` is the module's export
/// list — an underscore name it lists is exported, a public name it
/// omits is not, `+=` unions, a tuple reads like a list; a method is
/// never a module name so the convention keeps speaking for it; a
/// dynamic `__all__` is unreadable and the convention answers. The
/// step-8 review's three shapes are the unreadable half's load-bearing
/// rows: a `.extend`, a guarded `+=`, an escaped or f-string entry
/// each build the list dynamically, and reading the literal part alone
/// had narrowed bit 0 on a name the module exports; a docstring
/// mention falls the same (wider) way by design.
#[test]
fn python_all_is_the_module_export_list_when_literal() {
    for case in [
        "__all__ = [\"_hid\", \"Shown\"]\ndef _hid():\n    pass\ndef omitted():\n    pass\n\
         class Shown:\n    def m(self):\n        pass\n    def _p(self):\n        pass\n\
         __all__ += (\"late\",)\ndef late():\n    pass ⇒ _hid:ES omitted:- m:ES _p:- late:ES",
        "__all__ = [n for n in dir()]\ndef open():\n    pass\ndef _shut():\n    pass ⇒ open:ES _shut:-",
        "__all__ = other.__all__\ndef open():\n    pass ⇒ open:ES",
        "__all__ = []\n__all__.extend([\"open\"])\ndef open():\n    pass\ndef _shut():\n    pass ⇒ open:ES _shut:-",
        "__all__ = [\"base\"]\nif X:\n    __all__ += [\"cond\"]\ndef base():\n    pass\ndef cond():\n    pass ⇒ base:ES cond:ES",
        "__all__ = [\"\\x66oo\"]\ndef foo():\n    pass\ndef _bar():\n    pass ⇒ foo:ES _bar:-",
        "__all__ = [f\"h_{S}\"]\ndef h_x():\n    pass\ndef h_y():\n    pass ⇒ h_x:ES h_y:ES",
        "\"\"\"see __all__\"\"\"\n__all__ = [\"a\"]\ndef a():\n    pass\ndef b():\n    pass ⇒ a:ES b:ES",
    ] {
        check(Lang::Python, case);
    }
}

/// K26, bit 0: the guarded climb and the two hops, leg by leg. Every
/// file here is a module (it exports), so bit 1 mirrors bit 0.
#[test]
fn typescript_export_climb_is_identity_guarded() {
    for case in [
        "export const f = () => {}; ⇒ f:ES",
        "export default { m() {} }; ⇒ m:ES",
        "export const f = function g() {}; ⇒ g:-",
        "export const dbnc = function dbnc() {}; ⇒ dbnc:ES",
        "export let lf = function lg() {}; ⇒ lg:-",
        "export var vf = function vk() {}; ⇒ vk:-",
        "export const h = function () {}; ⇒ h:ES",
        "export class K { m() {} } ⇒ m:-",
        "export default { handler: function hh() {} }; ⇒ hh:-",
        "export const o = { m() {} }; ⇒ m:-",
        "export function open() {}\nconst x = () => {}; ⇒ open:ES x:-",
        "export var vf = () => {}; ⇒ vf:ES",
    ] {
        check(Lang::TypeScript, case);
    }
}

/// K26: a destructuring export declares no unit at all — the producer
/// must not mint a symbol for `a`, `b` or `x` (and cannot, for there
/// is no declaration node to hang one on).
#[test]
fn typescript_destructuring_export_declares_no_symbol() {
    let src = "export const [a, b] = pair();\nexport const { x } = obj();\n";
    assert!(units::segments(src, Lang::TypeScript).is_empty());
}

/// K26, bit 1: the namespace chain under the module/script split —
/// a script's top-level namespace is global, a module's private
/// namespace is not, and `export namespace` opens it again. The last
/// case is the empty chain: `declare global` is no member, and an
/// ambient signature is no unit.
#[test]
fn typescript_scope_bit_reads_the_namespace_chain() {
    for case in [
        "namespace N { export function nf() {} } ⇒ nf:ES",
        "namespace N { function g() {} } ⇒ g:-",
        "import x from 'y';\nnamespace P { export function pf() {} } ⇒ pf:E",
        "export namespace P { export function pf() {} } ⇒ pf:ES",
        "export namespace A { namespace B { export function bf() {} } } ⇒ bf:E",
        "export namespace A { export namespace B { export function bf() {} } } ⇒ bf:ES",
        "export {};\ndeclare global { function gf(): void; } ⇒ ",
        "export {};\nmodule Foo { export function ff() {} } ⇒ ff:E",
        "module Foo { export function ff() {} } ⇒ ff:ES",
    ] {
        check(Lang::TypeScript, case);
    }
}

/// Markdown has no private heading: the constant units.rs stamps on
/// every section says so, and this pins it against a silent flip to 0
/// (which would make every section read private).
#[test]
fn markdown_sections_are_public_by_construction() {
    assert_eq!(MARKDOWN_VIS, VIS_EXPORTED | VIS_SCOPE_EXPORTED);
    let sections = units::segments("# Title\n\ntext\n", Lang::Markdown);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].vis, MARKDOWN_VIS);
}

/// The three bits are distinct positions and bit 1 never appears
/// without bit 0 — the property the wire mask and the advisory mask
/// both lean on.
#[test]
fn scope_bit_implies_the_export_bit() {
    assert_eq!(VIS_EXPORTED | VIS_SCOPE_EXPORTED | VIS_RESTRICTED, 7);
    for (e, s) in [(false, false), (false, true), (true, false), (true, true)] {
        let w = word(e, s);
        assert!(
            w & VIS_SCOPE_EXPORTED == 0 || w & VIS_EXPORTED != 0,
            "{e} {s}"
        );
    }
}
