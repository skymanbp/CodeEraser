//! Extra unit kinds for the relocation register, per language —
//! consumed ONLY by `fourclass::units`. `scan/spec.rs` must not grow
//! these: its `fn_kinds` drives the M1 function metrics and the
//! dogfood ratchet. A relocated `pub const` (the register's
//! CLASSES case) is a unit worth naming in a report even though it
//! is not a function.
//!
//! Only kinds whose node carries a `name` field are listed — the
//! extractor keys purely on that (every kind below is grammar-probed
//! for the field). Since plan v2.17 L round step 8 the register is
//! also the symbols table's declaration domain (the unmentioned
//! advisory and the export surface read it), so a language's named
//! type forms belong here whether or not a relocation case exists:
//! Go's `type_spec`/`type_alias` nest inside a `type_declaration`,
//! which the walker descends anyway, and Haskell's six type forms
//! carry `name` like a `bind`. Go `const_spec`/`var_spec` stay out —
//! they are as often a function-local as a package-level
//! declaration, and a local is never a cross-file identifier.

use crate::scan::lang::Lang;

/// Wrapper kinds whose child declaration REDECLARES a name instead of
/// introducing one: tree-sitter-haskell wraps `data instance F Int =
/// …` / `newtype instance …` as a `data_instance` around a plain
/// `data_type` / `newtype` node carrying the family's own name, so
/// without this guard every instance minted a second row of the
/// family (the step-8 review's duplicate-row catch). `type instance`
/// is its own kind and never keyed.
pub const REDECLARING: [&str; 1] = ["data_instance"];

pub fn extra(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Rust => &[
            "const_item",
            "static_item",
            "struct_item",
            "enum_item",
            "trait_item",
            "mod_item",
        ],
        Lang::Python => &["class_definition"],
        Lang::TypeScript | Lang::Tsx => &[
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
        ],
        Lang::Go => &["type_spec", "type_alias"],
        // tree-sitter-haskell 0.23.1 spells the synonym kind
        // `type_synomym` (sic) — the grammar's own name, probed
        Lang::Haskell => &[
            "data_type",
            "newtype",
            "type_synomym",
            "class",
            "type_family",
            "data_family",
        ],
        // the sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never four-classified
        _ => &[],
    }
}

/// Kinds whose unit name lives in a field OTHER than `name`, as
/// (kind, name field, qualifier field, display prefix). Rust impl
/// blocks carry only a `type` — without a unit for them, methods of
/// two different impls look top-level and their shared name/arity
/// key becomes false stacking evidence (attack review 2026-08-11
/// F7). The qualifier keeps `impl Advisor for Foo` distinct from
/// `impl Foo` — the two legitimately coexist (FPR replay caught the
/// unqualified key colliding on exactly that shape).
pub fn typed(lang: Lang) -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
    match lang {
        Lang::Rust => &[("impl_item", "type", "trait", "impl")],
        _ => &[],
    }
}
