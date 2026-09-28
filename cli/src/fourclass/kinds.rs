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

/// Kinds that declare only at package level — under the file root's
/// own `const_declaration` / `var_declaration` (grandparent = root):
/// the same node inside a function body declares a local, which is
/// never a cross-file identifier (plan v2.30 step 5b). A spec may name
/// several (`var a, b int`): each name is a unit of its own.
pub const PACKAGE_LEVEL: [&str; 2] = ["const_spec", "var_spec"];

/// Java's field forms (plan v2.30 step 5b-6): a `field_declaration`
/// declares only when `static` is among its modifiers — JLS 8.3.1.1
/// makes a static field one variable of the class, named across files
/// as `Type.NAME`, where an instance field is every object's own; an
/// interface's or annotation type's `constant_declaration` is
/// implicitly `public static final` (JLS 9.3). Each declarator is a
/// unit of its own (declared.rs).
pub const JAVA_FIELDS: [&str; 2] = ["field_declaration", "constant_declaration"];

/// The C family's variable form (plan v2.30 step 5b-6): a `declaration`
/// declares only at file scope — the translation unit, a namespace or
/// linkage body, through preprocessor conditionals and a template head
/// — and only the variables it defines (C11 6.9.2: an `extern` without
/// an initializer is a reference, a prototype names a function); one
/// unit per declarator (declared.rs).
pub const C_VARIABLE: &str = "declaration";

/// TypeScript's lexical forms (plan v2.30 step 5b-6): `const` / `let`
/// (`lexical_declaration`) and `var` (`variable_declaration`) declare
/// only at module level — under the program, an `export` / `declare`
/// wrapper, or a namespace / module / `declare global` body; the same
/// kinds inside a function, a loop head or a bare block bind locals.
/// One unit per bound identifier (declared.rs).
pub const TS_LEXICAL: [&str; 2] = ["lexical_declaration", "variable_declaration"];

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
/// nothing. A `declaration` is a unit per variable it defines at file
/// scope (C_VARIABLE, declared.rs) — never per prototype: a header
/// spelling a name is itself the mention (booklet §6). The specifiers
/// declare only with a body (BODIED), and the typedef keys by its
/// declarator leaf (declared.rs).
const C_FAMILY: [&str; 10] = [
    "preproc_def",
    "preproc_function_def",
    "struct_specifier",
    "union_specifier",
    "enum_specifier",
    "type_definition",
    "declaration",
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
            "lexical_declaration",
            "variable_declaration",
        ],
        Lang::Go => &["type_spec", "type_alias", "const_spec", "var_spec"],
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
        Lang::Java => &[
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
            "record_declaration",
            "annotation_type_declaration",
            "field_declaration",
            "constant_declaration",
        ],
        // the sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never four-classified
        _ => &[],
    }
}
