//! The AST half per language, measured through the real producer
//! (fourclass::units, so the wiring is under test, not a copy of it)
//! and shown in the domain's own view: each unit as `name:letters`
//! where the name is `mention_name` of its key — a Go method by its
//! receiver-stripped name — and a unit the extractor puts out of the
//! domain (`impl W`, an anonymous default export, a dunder) has no
//! row. So an empty right side is itself a witness: K24's
//! producer-measured domain exits live here, beside K25 and K28's.
//! Each case is one line (testutil::check_word_case) led by the
//! declaring file's path — the language and the tokenizer arm both
//! come from it, as in name.rs — then every in-domain unit in source
//! order, spelled in the shared alphabet (`bit_of`, mod.rs): `T`
//! Test, `F` Ffi, `G` Registration, `M` Member, `D` DefaultExport,
//! `A` Ambient, `L` Allow, `C` Cfg, `-` none. These are K37's
//! producer-half witnesses: every AST-half category fires on an
//! in-domain declaration. One table, not one per language: the clone
//! gate reads a run of same-shaped `check(Lang, &[…])` blocks as one
//! another's copies.

use super::*;
use crate::fourclass::units;
use crate::mention::name::mention_name;
use std::path::Path;

fn conv_of(rel: &str, src: &str, lang: Lang) -> Vec<(String, i64)> {
    let mut units: Vec<_> = units::segments(src, lang)
        .into_iter()
        .filter_map(|u| Some((u.start_line, mention_name(rel, &u.key)?, u.conv)))
        .collect();
    units.sort();
    units
        .into_iter()
        .map(|(_, name, conv)| (name, conv))
        .collect()
}

const CASES: &[&str] = &[
    // Rust — one predicate-tree reading yields Test and Cfg as a
    // partition: `test` anywhere in the tree (under `all`/`not`,
    // inherited from an enclosing `mod`, as an inner attribute) is
    // Test; other atoms without it are Cfg; a `feature = "test"`
    // VALUE is not the atom.
    "a.rs #[cfg(test)]\nfn a() {}\nfn b() {} ⇒ a:T b:-",
    "a.rs #[cfg(all(test, feature = \"x\"))]\npub fn b() {}\n#[cfg(not(test))]\nfn k() {} ⇒ b:T k:T",
    "a.rs #[cfg(target_os = \"macos\")]\nfn c() {}\n#[cfg(feature = \"test\")]\nfn d() {} ⇒ c:C d:C",
    "a.rs #[cfg(unix)]\nmod m {\n    #[cfg(test)]\n    pub fn j() {}\n    pub fn l() {}\n} ⇒ m:C j:T l:C",
    "a.rs mod t {\n    #![cfg(test)]\n    pub fn j() {}\n}\nfn free() {} ⇒ t:T j:T free:-",
    // Rust — the export table by last path segment, the `unsafe(…)`
    // wrapper, `doc(hidden)`, `proc_macro*`, the `extern` modifier, a
    // doc comment between attribute and item (R9), and inheritance
    // from an `impl` (whose own `impl W` key is out of the domain, so
    // it has no row).
    "a.rs #[no_mangle]\n/// doc\npub fn e() {}\n#[unsafe(no_mangle)]\npub extern \"C\" fn d() {} ⇒ e:F d:F",
    "a.rs #[pyo3::pyfunction]\nfn f() {}\n#[doc(hidden)]\npub fn g() {}\n#[doc = \"x\"]\nfn h() {} ⇒ f:F g:F h:-",
    "a.rs #[proc_macro_derive(X)]\npub fn derive_x() {}\npub extern \"C\" fn c() {}\n#[used]\nstatic S: u8 = 0; ⇒ derive_x:F c:F S:F",
    "a.rs #[wasm_bindgen]\nimpl W {\n    pub fn m() {}\n}\nimpl V {\n    pub fn n() {}\n} ⇒ m:F n:-",
    // Rust — `allow`/`expect` naming `dead_code`: outer, inner at
    // file level, inner in a function body (reaching the items inside
    // it and not the function), alongside other lints — and never
    // for an unrelated lint.
    "a.rs #[allow(dead_code)]\nfn h() {}\n#[expect(dead_code)]\nfn i() {}\n#[allow(unused)]\nfn y() {} ⇒ h:L i:L y:-",
    "a.rs #![allow(unused, dead_code)]\nfn z() {}\n#[cfg(test)]\n#[allow(dead_code)]\nfn q() {} ⇒ z:L q:TL",
    "a.rs fn outer() {\n    #![allow(dead_code)]\n    struct Helper;\n} ⇒ outer:- Helper:L",
    // TypeScript — decorators in both export orders and with comments
    // between; the named default export in both declaration kinds;
    // the anonymous one is out of the domain by its key (K25 — no
    // row); ambient by ancestor presence alone: the container-less
    // `declare class`, `export declare`, a body in `declare module`.
    "a.ts @dec\nclass A {}\nexport @dec class B {}\n@dec\nexport class C {}\nclass P {} ⇒ A:G B:G C:G P:-",
    "a.ts // c\n@dec\n// c2\nclass F {} ⇒ F:G",
    "a.ts export default function Named() {}\nexport function f() {} ⇒ Named:D f:-",
    "a.ts @dec\nexport default class E {} ⇒ E:GD",
    "a.ts export default function () {} ⇒",
    "a.ts declare class D {}\nexport declare class E {}\ndeclare module \"m\" {\n  export class G {}\n}\nclass H {} ⇒ D:A E:A G:A H:-",
    // Python — the registrar table on `@a.b(...)`, `@a.b` and bare
    // `@b`; a plain decorator claims nothing; class members carry
    // Member at any depth; a dunder member is out of the domain (K28
    // — no row).
    "a.py @app.route(\"/x\")\ndef f():\n    pass\n@pytest.fixture\ndef g():\n    pass\n@fixture\ndef h():\n    pass\n@other\ndef o():\n    pass ⇒ f:G g:G h:G o:-",
    "a.py class A:\n    @staticmethod\n    def m(self):\n        pass\n    class B:\n        pass\ndef free():\n    pass ⇒ A:- m:M B:M free:-",
    "a.py @app.register\nclass R:\n    pass\nclass Q:\n    @cli.command()\n    def run(self):\n        pass ⇒ R:G Q:- run:MG",
    "a.py class A:\n    def __init__(self):\n        pass\n    def go(self):\n        pass ⇒ A:- go:M",
    // Python — the `if TYPE_CHECKING:` consequence is ambient (bare
    // or qualified flag), and so is an `elif TYPE_CHECKING:` arm (the
    // step-8 review's stub); the `else` arm, the arm before it and an
    // unrelated `if` are live code (L round step 8, user ruling
    // 2026-08-28).
    "a.py if TYPE_CHECKING:\n    class Only:\n        pass\n    def tc():\n        pass\nelse:\n    def live():\n        pass\n\
     if typing.TYPE_CHECKING:\n    def qual():\n        pass\nif DEBUG:\n    def dbg():\n        pass ⇒ Only:A tc:A live:- qual:A dbg:-",
    "a.py if sys.version_info >= (3, 12):\n    def newpath():\n        pass\nelif TYPE_CHECKING:\n    class Stub:\n        pass\n\
     else:\n    def old():\n        pass ⇒ newpath:- Stub:A old:-",
    // Haskell — the exported name hits its own binding, with or
    // without the C entity string, and a sibling binding is
    // untouched; infix definitions (``x `f` y``, `a --> b`) yield no
    // unit at all under the extractor (K24 — no row).
    "a.hs module M where\nimport Foreign.C\nforeign export ccall hsAdd :: CInt -> CInt\nhsAdd x = x\nforeign export ccall \"hs_mul\" hsMul :: CInt -> CInt\nhsMul x = x\nhelper y = y ⇒ hsAdd:F hsMul:F helper:-",
    "a.hs module M where\nx `f` y = x + y\na --> b = a * b\nplain z = z ⇒ plain:-",
    // Go — `//export` must name the function it precedes, with cgo's
    // blank tolerance; `//go:wasmexport` exports whatever follows; a
    // plain comment or a mismatched name does nothing; a method is in
    // the domain by its receiver-stripped name.
    "a.go package main\n\n//export Add\nfunc Add() {}\n\n//go:wasmexport add\nfunc add() {}\n\n// plain\nfunc plain() {}\n\n//export Other\nfunc Mismatch() {} ⇒ Add:F add:F plain:- Mismatch:-",
    "a.go package p\n\n//export  Two\nfunc Two() {}\n\ntype T struct{}\n\nfunc (T) n() {} ⇒ Two:F T:- n:-",
];

#[test]
fn every_ast_half_category_fires_on_an_in_domain_declaration() {
    for case in CASES {
        let (rel, case) = case.split_once(' ').expect("path then case");
        let lang = Lang::judged_path(Path::new(rel)).expect("a judged path");
        let conv_of = |src: &str, lang| conv_of(rel, src, lang);
        crate::testutil::check_word_case(lang, case, conv_of, bit_of);
    }
}
