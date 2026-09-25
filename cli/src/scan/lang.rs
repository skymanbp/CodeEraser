//! Language identification and per-language tree-sitter grammar lookup.
//! M1 launch set (plan §6 M1): TypeScript / Python / Rust / Go / Markdown
//! (Markdown is size-only: no grammar, no functions). M5-3k adds
//! Haskell (full grammar — spike pinned in tests/it/grammar_pins.rs).
//! Plan v2.5 adds the SCAN-ONLY arm: common front-end/script
//! extensions that enter the scan's size gates, the guard's hard
//! budget and the score ratchet — and nothing else (see scan_only).
//!
//! APPEND-ONLY: `Lang as i64` is a frozen wire position (graph node
//! rows) — inserting a variant would silently relabel every language
//! code downstream (RM15). Scan-only variants sit AFTER the sentinel
//! and never reach the wire anyway (they are never indexed).
//!
//! Plan v2.30 (docs/reference/language-expansion.md) reserves six
//! more codes after the arm — C / C++ / Lua / Java / Ruby / R — and
//! promotes HTML to a judged document language. Each row turns judged
//! in its own step, together with its measurement tables, so no file
//! is ever judged or fingerprinted under a placeholder spec; the core
//! reads the judged set off the wire (`judgedMask`, proto 7.2.0)
//! instead of a constant, so a row flipping here needs no core change.
//! Step 2 turned C and C++ (spec_c.rs; `.h` is C++ by the 2026-09-24
//! ruling — a header parsed as C loses every class body); step 3 turned
//! Java (spec_java.rs); step 4 turned Lua and R (spec_lua.rs, spec_r.rs;
//! R takes both `.R` and `.r`, the extension match being exact).

use std::path::Path;
use tree_sitter_language::LanguageFn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Python,     // 0
    TypeScript, // 1
    Tsx,        // 2
    Rust,       // 3
    Go,         // 4
    Markdown,   // 5
    Haskell,    // 6
    /// Wire sentinel: "extension not ours". NEVER produced by
    /// from_path (which returns None) — it exists so the graph wire's
    /// unknown-language code is 7, not Python's 0 (RM15: today an
    /// unknown extension was indistinguishable from Python on the
    /// wire; the core's contract only rejects negatives).
    LangUnknown, // 7
    // ---- the v2.5 scan-only arm (never on the wire) ----
    JavaScript, // 8
    Css,        // 9
    Html,       // 10
    Vue,        // 11
    Svelte,     // 12
    Shell,      // 13
    Yaml,       // 14
    // ---- plan v2.30 reserved codes (language-expansion.md §1):
    // each turns judged in its own step, with its tables ----
    C,    // 15
    Cpp,  // 16
    Lua,  // 17
    Java, // 18
    Ruby, // 19
    R,    // 20
}

/// ONE row per language: variant, extensions, report name, scan-only
/// bit. This table drives from_path / name / scan_only — as separate
/// matches each was a cyclomatic-warn-sized copy of the same facts.
const LANGS: &[(Lang, &[&str], &str, bool)] = &[
    (Lang::Python, &["py"], "python", false),
    (Lang::TypeScript, &["ts", "mts", "cts"], "typescript", false),
    (Lang::Tsx, &["tsx"], "tsx", false),
    (Lang::Rust, &["rs"], "rust", false),
    (Lang::Go, &["go"], "go", false),
    (Lang::Markdown, &["md", "markdown"], "markdown", false),
    (Lang::Haskell, &["hs"], "haskell", false),
    (Lang::LangUnknown, &[], "unknown", false),
    (
        Lang::JavaScript,
        &["js", "mjs", "cjs", "jsx"],
        "javascript",
        true,
    ),
    (Lang::Css, &["css", "scss", "less"], "css", true),
    (Lang::Html, &["html", "htm"], "html", true),
    (Lang::Vue, &["vue"], "vue", true),
    (Lang::Svelte, &["svelte"], "svelte", true),
    (Lang::Shell, &["sh", "bash"], "shell", true),
    (Lang::Yaml, &["yml", "yaml"], "yaml", true),
    // plan v2.30 rows: the wire code was frozen here first (RM15); the
    // extensions, the grammar and the tables arrive with the language's
    // own step (§13) — an extension-less row is what the sentinel
    // already is, and the judged mask never counts one
    (Lang::C, &["c"], "c", false),
    (
        Lang::Cpp,
        &["cpp", "cc", "cxx", "hpp", "hh", "hxx", "h", "inl"],
        "cpp",
        false,
    ),
    (Lang::Lua, &["lua"], "lua", false),
    (Lang::Java, &["java"], "java", false),
    (Lang::Ruby, &[], "ruby", true),
    (Lang::R, &["R", "r"], "r", false),
];

/// The grammar of every AST-backed language, as the `LanguageFn`
/// value its crate exports (`Language::from` converts at lookup). A
/// row is five distinct tokens, under the clone gate's diversity
/// floor; a match arm per grammar read as clones of itself from the
/// eighth arm on (tests/it/grammar_pins.rs stores its pins the same
/// way, for the same reason). The plan v2.30 reserved codes join as
/// their steps land.
const GRAMMARS: [(Lang, LanguageFn); 11] = [
    (Lang::Python, tree_sitter_python::LANGUAGE),
    (
        Lang::TypeScript,
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
    ),
    (Lang::Tsx, tree_sitter_typescript::LANGUAGE_TSX),
    (Lang::Rust, tree_sitter_rust::LANGUAGE),
    (Lang::Go, tree_sitter_go::LANGUAGE),
    (Lang::Haskell, tree_sitter_haskell::LANGUAGE),
    (Lang::C, tree_sitter_c::LANGUAGE),
    (Lang::Cpp, tree_sitter_cpp::LANGUAGE),
    (Lang::Java, tree_sitter_java::LANGUAGE),
    (Lang::Lua, tree_sitter_lua::LANGUAGE),
    (Lang::R, tree_sitter_r::LANGUAGE),
];

impl Lang {
    /// This language's LANGS row — total by construction (every
    /// variant is in the table; the count is pinned in tests).
    fn row(self) -> &'static (Lang, &'static [&'static str], &'static str, bool) {
        LANGS
            .iter()
            .find(|&&(l, ..)| l == self)
            .expect("every Lang variant has a LANGS row")
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        LANGS
            .iter()
            .find(|(_, exts, ..)| exts.contains(&ext))
            .map(|&(l, ..)| l)
    }

    /// from_path in the judgment surfaces' form: None for unknown
    /// extensions AND for the scan-only arm. fourclass, churn and
    /// structure inputs come through here; the scan and the guard's
    /// budget stay on from_path (the plan v2.5 boundary).
    pub fn judged_path(path: &Path) -> Option<Self> {
        Self::from_path(path).filter(|l| !l.scan_only())
    }

    /// The judged-language set as a wire bitmask (H1 slice 2,
    /// 2.29.0): bit = Lang wire code, set = judged. The boundary
    /// AUTHORITY stays this table's scan_only column (CLAUDE.md
    /// names it); the mask only makes the set core-visible and
    /// drift-detectable through the verdict knobs echo. The wire
    /// sentinel is excluded: from_path never produces it, so it is
    /// in no population S derives from.
    pub fn judged_mask() -> i64 {
        LANGS
            .iter()
            .filter(|&&(l, ..)| l != Lang::LangUnknown && !l.scan_only())
            .fold(0, |m, &(l, ..)| m | (1 << (l as i64)))
    }

    /// The plan v2.5 boundary predicate: a scan-only language enters
    /// the scan (size gates), the guard's hard budget and the score
    /// ratchet — and NEVER the dedup index, the graph, or any
    /// judgment family. Markdown is not in this class: it is
    /// grammar-less but fully judged (docdup corpus, graph ladder).
    pub fn scan_only(self) -> bool {
        self.row().3
    }

    /// Grammar for AST-backed languages; None = size-only (Markdown,
    /// the scan-only arm), the wire sentinel (never walked) or a plan
    /// v2.30 reserved code whose grammar lands with its own step.
    pub fn grammar(self) -> Option<tree_sitter::Language> {
        GRAMMARS
            .iter()
            .find(|(l, _)| *l == self)
            .map(|&(_, f)| f.into())
    }

    /// The T1/T2/T3 population (plan v2.30 §2): a grammar to tokenize
    /// with AND a code language. HTML parses — docdup segments, the
    /// reference ladder, section anchors — but never fingerprints: its
    /// leaves carry no identifiers, so any two pages would hash into
    /// one clone pair. The index refresh, the guard's clone probe and
    /// the unit cache read this; every other grammar consumer keeps
    /// reading `grammar()`.
    pub fn fingerprints(self) -> bool {
        self.grammar().is_some() && self != Self::Html
    }

    pub fn name(self) -> &'static str {
        self.row().2
    }
}

/// The mention tokenizer's `$` arm (sealed criterion §2, frozen beside
/// the boundary predicate above because it is an extension table of
/// the same kind): files of these extensions keep a `$`-carrying run
/// WHOLE — `$ZodString` and `ZodString` are distinct identifiers in
/// the JS family, and so are `Outer$Inner` and `Inner` in Java, whose
/// identifiers take `$` too; emitting the `$`-free piece would let each
/// hide the other's death. Every other extension, and no extension,
/// takes the union arm (shell `$name`, Haskell `f$g`). A `MENTION_REV`
/// input.
pub const MENTION_WHOLE_RUN_EXTS: [&str; 11] = [
    "ts", "tsx", "mts", "cts", "js", "mjs", "cjs", "jsx", "vue", "svelte", "java",
];

#[cfg(test)]
#[path = "../../tests/unit/scan/lang.rs"]
mod tests;
