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
//! carry `name` like a `bind`, as Java's five type declarations do.
//! Go `const_spec`/`var_spec` enter at PACKAGE level only (plan v2.30
//! step 5b): a package-level constant is a cross-file identifier like
//! Rust's `const_item`, while the same node kinds inside a function
//! body declare locals no other file can name (PACKAGE_LEVEL). Step
//! 5b-6 widened the domain by the same rule to the other languages'
//! file-level variables — a Java `static` field (JAVA_FIELDS), a C /
//! C++ file-scope variable definition (C_VARIABLE) and a TypeScript
//! module-level `const` / `let` / `var` (TS_LEXICAL); which node
//! declares here, and which names, is fourclass/declared.rs's question.
//!
//! The kind tables are the core's since plan v2.32 step 2 — each
//! language's `extra` list in CE.Lang.<Language>, the shared ones in
//! CE.Lang.Common.Graph (`fourclass`) — read off `tables/1`
//! (crate::tables); the reasons each kind is there sit with the data.

use crate::scan::lang::Lang;
use crate::tables::Fourclass;

/// Declaration forms carried beside the key hash in fourclass/2.
/// Kept here so units can measure kinds without importing decls,
/// which reads the unit table back.
pub const KIND_FN: i64 = 1;
pub const KIND_NAMED: i64 = 2;
pub const KIND_IMPL: i64 = 3;
pub const KIND_SECTION: i64 = 4;

/// The shared kind tables: REDECLARING (a wrapper whose child
/// redeclares a family's name), PACKAGE_LEVEL (kinds declaring only at
/// package level), JAVA_FIELDS, C_VARIABLE, TS_LEXICAL (the file-level
/// variable forms of step 5b-6) and BODIED (kinds declaring only with a
/// `body`).
pub fn shared() -> &'static Fourclass {
    &crate::tables::get().fourclass
}

/// A language's extra unit kinds; a language the package does not judge
/// (the sentinel, the scan-only arm) has none.
pub fn extra(lang: Lang) -> &'static [&'static str] {
    shared().extra.get(lang).copied().unwrap_or(&[])
}
