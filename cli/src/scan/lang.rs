//! Language identification and per-language tree-sitter grammar lookup.
//! M1 launch set (plan §6 M1): TypeScript / Python / Rust / Go / Markdown
//! (Markdown is size-only: no grammar, no functions). M5-3k adds
//! Haskell (full grammar — spike pinned in tests/hs_grammar_pin.rs).
//!
//! APPEND-ONLY: `Lang as i64` is a frozen wire position (graph node
//! rows) — inserting a variant would silently relabel every language
//! code downstream (RM15).

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
}

impl Lang {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "py" => Some(Self::Python),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "md" | "markdown" => Some(Self::Markdown),
            "hs" => Some(Self::Haskell),
            _ => None,
        }
    }

    /// Grammar for AST-backed languages; None = size-only (Markdown)
    /// or the wire sentinel (never walked).
    pub fn grammar(self) -> Option<tree_sitter::Language> {
        match self {
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Self::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::Go => Some(tree_sitter_go::LANGUAGE.into()),
            Self::Markdown => None,
            Self::Haskell => Some(tree_sitter_haskell::LANGUAGE.into()),
            Self::LangUnknown => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Markdown => "markdown",
            Self::Haskell => "haskell",
            Self::LangUnknown => "unknown",
        }
    }
}
