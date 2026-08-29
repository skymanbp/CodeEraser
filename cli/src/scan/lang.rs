//! Language identification and per-language tree-sitter grammar lookup.
//! M1 launch set (plan §6 M1): TypeScript / Python / Rust / Go / Markdown
//! (Markdown is size-only: no grammar, no functions). M5-3k adds
//! Haskell (full grammar — spike pinned in tests/hs_grammar_pin.rs).
//! Plan v2.5 adds the SCAN-ONLY arm: common front-end/script
//! extensions that enter the scan's size gates, the guard's hard
//! budget and the score ratchet — and nothing else (see scan_only).
//!
//! APPEND-ONLY: `Lang as i64` is a frozen wire position (graph node
//! rows) — inserting a variant would silently relabel every language
//! code downstream (RM15). Scan-only variants sit AFTER the sentinel
//! and never reach the wire anyway (they are never indexed).

use std::path::Path;

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
    /// the scan-only arm) or the wire sentinel (never walked).
    pub fn grammar(self) -> Option<tree_sitter::Language> {
        match self {
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Self::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::Go => Some(tree_sitter_go::LANGUAGE.into()),
            Self::Haskell => Some(tree_sitter_haskell::LANGUAGE.into()),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        self.row().2
    }
}

/// The mention tokenizer's `$` arm (sealed criterion §2, frozen beside
/// the boundary predicate above because it is an extension table of
/// the same kind): files of these extensions keep a `$`-carrying run
/// WHOLE — `$ZodString` and `ZodString` are distinct identifiers in
/// the JS family, and emitting the `$`-free piece would let each hide
/// the other's death. Every other extension, and no extension, takes
/// the union arm (shell `$name`, Haskell `f$g`). A `MENTION_REV` input.
pub const MENTION_WHOLE_RUN_EXTS: [&str; 10] = [
    "ts", "tsx", "mts", "cts", "js", "mjs", "cjs", "jsx", "vue", "svelte",
];

#[cfg(test)]
#[path = "../../tests/unit/scan/lang.rs"]
mod tests;
