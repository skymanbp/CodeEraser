//! Extra unit kinds for the relocation register, per language —
//! consumed ONLY by `fourclass::units`. `scan/spec.rs` must not grow
//! these: its `fn_kinds` drives the M1 function metrics and the
//! dogfood ratchet. A relocated `pub const` (the register's
//! CLASSES case) is a unit worth naming in a report even though it
//! is not a function.
//!
//! Every kind but one carries a `name` field (grammar-probed) and the
//! extractor keys on it; the exception is a C `type_definition`, which
//! spells the new name at the end of its declarator chain, and the four
//! C-family type specifiers declare only with a `body` (BODIED —
//! `struct K x;` references K by the same node kind that declares it).
//! Since plan v2.17 L round step 8 the register is
//! also the symbols table's declaration domain (the unmentioned
//! advisory and the export surface read it), so a language's named
//! type forms belong here whether or not a relocation case exists:
//! Go's `type_spec`/`type_alias` nest inside a `type_declaration`,
//! which the walker descends anyway, and Haskell's six type forms
//! carry `name` like a `bind`. Go `const_spec`/`var_spec` stay out —
//! they are as often a function-local as a package-level
//! declaration, and a local is never a cross-file identifier.

use crate::scan::lang::Lang;

/// Declaration forms carried beside the key hash in fourclass/2.
/// Kept here so units can measure kinds without importing decls,
/// which reads the unit table back.
pub const KIND_FN: i64 = 1;
pub const KIND_NAMED: i64 = 2;
pub const KIND_IMPL: i64 = 3;
pub const KIND_SECTION: i64 = 4;

/// Wrapper kinds whose child declaration REDECLARES a name instead of
/// introducing one: tree-sitter-haskell wraps `data instance F Int =
/// …` / `newtype instance …` as a `data_instance` around a plain
/// `data_type` / `newtype` node carrying the family's own name, so
/// without this guard every instance minted a second row of the
/// family (the step-8 review's duplicate-row catch). `type instance`
/// is its own kind and never keyed.
pub const REDECLARING: [&str; 1] = ["data_instance"];

/// Kinds that declare only with a `body`: `struct K { … }` declares K,
/// while `struct K x;`, a `struct K *` parameter type and a forward
/// `class Fwd;` reference or promise it by the SAME node kind and are
/// nothing a file can be judged on (plan v2.30 step 2).
pub const BODIED: [&str; 4] = [
    "struct_specifier",
    "union_specifier",
    "enum_specifier",
    "class_specifier",
];

/// The C family (plan v2.30 step 2), one table for both grammars: a
/// macro is a real declaration, the type specifiers and a typedef name
/// types, and the three C++ forms at the end never occur in a C parse
/// (tree-sitter-c has no such kinds), so sharing the table costs
/// nothing. A prototype `declaration` is not a unit — a header spelling
/// the name is itself the mention (booklet §6). The specifiers declare
/// only with a body (BODIED), and the typedef keys by its declarator
/// leaf (units.rs).
const C_FAMILY: [&str; 9] = [
    "preproc_def",
    "preproc_function_def",
    "struct_specifier",
    "union_specifier",
    "enum_specifier",
    "type_definition",
    "class_specifier",
    "namespace_definition",
    "alias_declaration",
];

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
        Lang::C | Lang::Cpp => &C_FAMILY,
        // the sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never four-classified
        _ => &[],
    }
}
