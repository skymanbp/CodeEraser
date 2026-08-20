//! Extra unit kinds for the relocation register, per language —
//! consumed ONLY by `fourclass::units`. `scan/spec.rs` must not grow
//! these: its `fn_kinds` drives the M1 function metrics and the
//! dogfood ratchet. A relocated `pub const` (the register's
//! CLASSES case) is a unit worth naming in a report even though it
//! is not a function.
//!
//! Only kinds whose node carries a `name` field are listed — the
//! extractor keys purely on that. Go's named forms nest inside
//! *_spec children and stay out until a Go relocation case exists.

use crate::scan::lang::Lang;

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
        // Haskell's named type forms (data_type/newtype/class) stay
        // out until a Haskell relocation case exists — the Go
        // precedent above; the sentinel is never walked, and the
        // scan-only arm (plan v2.5) is never four-classified.
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
