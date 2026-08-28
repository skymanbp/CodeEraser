//! Site detection pinned per language (split from sites.rs at the
//! 300-line dogfood gate when the TS star export joined the table).

use super::detect;
use crate::scan::lang::Lang;

/// One table drives both checks per language: the expected
/// (kind, spec) sequence, and the anti-invention rule that every
/// spec is a substring of its STATEMENT WINDOW — the site line is
/// the statement head, and a multi-line TS import carries its
/// full specifier on a later line of the same statement (2c/2d
/// review F1: 14 frozen zod sites; rust only holds the per-line
/// form by accident of first-line truncation).
/// Pinned shapes: `mod foo { … }` is not a site, a plain export
/// is not a site, one site per Python import target, a
/// multi-line use keeps ONE site whose spec is the first-line
/// fragment (module header), a qualified/aliased Haskell import
/// keeps the bare module name, `foreign import` is NOT a
/// site — its anon `import` token shares the kind name (the 3k
/// D11 collision class) but carries no module field — and a TS
/// star export (bare or namespaced) is ONE `export_star` site
/// while the clause form stays `export_from`.
/// (language, source, expected (kind, spec) sequence).
type Case = (Lang, &'static str, &'static [(&'static str, &'static str)]);

/// The per-language table, split from its assertion loop at the
/// E01 fn-length line.
fn cases() -> [Case; 5] {
    [
        // `from __future__` is an `import_from` site on the literal
        // module name (step 8, O27)
        (
            Lang::Python,
            "from __future__ import annotations\nimport a.b, c as d\nfrom .pkg import thing\n",
            &[
                ("import_from", "__future__"),
                ("import", "a.b"),
                ("import", "c"),
                ("import_from", ".pkg"),
            ],
        ),
        // `import fs = require("./b")` is an `import` site off the
        // require clause; `import X = A.B.C` names a namespace and
        // opens none (step 8, O26)
        (
            Lang::TypeScript,
            "import { x } from \"./util\";\nimport {\n  a,\n  b,\n} from \"./multi\";\nexport { y } from './other';\nexport const z = 1;\nexport * from './all';\nexport * as ns from './space';\nimport fs = require(\"./req\");\nimport X = A.B.C;\nexport import Y = A.B;\n",
            &[
                ("import", "./util"),
                ("import", "./multi"),
                ("export_from", "./other"),
                ("export_star", "./all"),
                ("export_star", "./space"),
                ("import", "./req"),
            ],
        ),
        (
            Lang::Rust,
            // the #[path] attribute emits NO site of its own — the
            // remap is ladder-side (rs.rs path_attr), which is what
            // keeps the frozen site universe standing across REV 5
            "mod alpha;\n#[path = \"x.rs\"]\nmod beta { fn x() {} }\nuse crate::a::{b, c};\nuse crate::{\n    d,\n    e,\n};\n",
            &[
                ("mod_decl", "alpha"),
                ("use", "crate::a::{b, c}"),
                ("use", "crate::{"),
            ],
        ),
        (
            Lang::Go,
            "package main\n\nimport (\n\t\"fmt\"\n\t\"github.com/x/y\"\n)\n",
            &[("import", "fmt"), ("import", "github.com/x/y")],
        ),
        // a `{-# SOURCE #-}` import keeps the bare module name — the
        // ladder answers M.hs for it (step 8, O28)
        (
            Lang::Haskell,
            "module Main where\n\nimport CE.Alpha\nimport qualified Data.Map as M\nimport Data.List (sort)\nimport {-# SOURCE #-} CE.Boot (x)\n\nforeign import ccall \"math.h sin\" c_sin :: Double -> Double\n",
            &[
                ("import", "CE.Alpha"),
                ("import", "Data.Map"),
                ("import", "Data.List"),
                ("import", "CE.Boot"),
            ],
        ),
    ]
}

#[test]
fn per_language_kinds_specs_and_line_substrings() {
    for (lang, text, want) in cases() {
        let found = detect(text, lang);
        let got: Vec<(&str, &str)> = found.iter().map(|s| (s.kind, s.spec.as_str())).collect();
        assert_eq!(got, *want, "{lang:?}");
        let lines: Vec<&str> = text.lines().collect();
        let stray: Vec<&str> = found
            .iter()
            .filter(|s| {
                let end = (s.line + 15).min(lines.len());
                !lines[s.line - 1..end].iter().any(|l| l.contains(&s.spec))
            })
            .map(|s| s.spec.as_str())
            .collect();
        assert!(
            stray.is_empty(),
            "{lang:?}: specs not within their statement window: {stray:?}"
        );
    }
}

/// Self-ownership is filtered (Opus review): a `mod foo;` site's
/// one-line unit is the site itself, so its owner is None, while
/// a use inside a real function keeps its owner. Same-line sites
/// get distinct nth ordinals (unique identity for 2c sampling).
#[test]
fn owner_not_self_and_nth_ordinals() {
    let text = "mod alpha;\nfn holder() {\n    use crate::x;\n}\n";
    let found = detect(text, Lang::Rust);
    assert_eq!(found[0].kind, "mod_decl");
    assert_eq!(found[0].owner, None, "self-ownership must filter");
    assert_eq!(found[1].kind, "use");
    assert_eq!(found[1].owner.as_deref(), Some("holder/0"));
    let two = detect("[a](./x.md) [b](./y.md)\n", Lang::Markdown);
    assert_eq!(
        two.iter().map(|s| s.nth).collect::<Vec<_>>(),
        vec![0, 1],
        "same-line sites need distinct ordinals"
    );
}
